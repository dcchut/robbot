use crate::containers::{SfClientContainer, SqliteConnectionContainer};
use crate::models::card::{get_card, insert_card, Card, NewCard};
use crate::models::card_lookup::{insert_card_lookup, search_card_lookups, NewCardLookup};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    utils::MessageBuilder,
};

async fn mtg_help(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg.reply(ctx, "help").await;

    Ok(())
}

async fn load_card(ctx: &mut Context, query: &str) -> Result<Card, ()> {
    let data = ctx.data.read().await;

    if let Some(conn) = data.get::<SqliteConnectionContainer>() {
        // First attempt to search the sqlite cache for this card
        if let Some(cached_card) = search_card_lookups(&query, conn).await {
            return Ok(cached_card);
        } else {
            // Load the card using the scryfall API
            if let Some(sfclient) = data.get::<SfClientContainer>() {
                let sfclient = sfclient.read().await;
                // We have to use ? here to avoid holding a Box<dyn Error> across an await point
                let card = sfclient.card_named(true, &query).await.map_err(|_e| ())?;

                // Is this card in the DB?
                if let Some(db_card) = get_card(&card.gameplay.name, conn).await {
                    let card_lookup = NewCardLookup {
                        search_term: String::from(query),
                        card_id: db_card.id,
                    };

                    insert_card_lookup(&card_lookup, conn).await;

                    return Ok(db_card);
                } else {
                    // Otherwise insert the card into the DB
                    let new_card = NewCard {
                        name: String::from(&card.gameplay.name),
                        type_line: String::from(&card.gameplay.type_line),
                        mana_cost: card.gameplay.mana_cost.clone(),
                        oracle_text: card.gameplay.oracle_text.clone(),
                        flavor_text: card.print.flavor_text.clone(),
                        image_uri: {
                            if let Some(uris) = &card.print.image_uris {
                                Some(uris.border_crop.clone())
                            } else {
                                None
                            }
                        },
                    };

                    if let Some(db_card) = insert_card(&new_card, conn).await {
                        let card_lookup = NewCardLookup {
                            search_term: String::from(query),
                            card_id: db_card.id,
                        };

                        insert_card_lookup(&card_lookup, conn).await;

                        return Ok(db_card);
                    }
                }
            }
        }
    }

    Err(())
}

async fn card_search(ctx: &mut Context, msg: &Message, query: &str) -> bool {
    if let Ok(card) = load_card(ctx, &query).await {
        // Create an embed for this card
        let _ = msg
            .channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title(&card.name);

                    // If we have card art URI's, include them.
                    if let Some(uri) = &card.image_uri {
                        e.image(&uri);
                    }

                    e.field("Type:", &card.type_line, true);

                    if let Some(mana_cost) = &card.mana_cost {
                        if !mana_cost.is_empty() {
                            e.field("Mana cost:", mana_cost, true);
                        }
                    }

                    if let Some(oracle_text) = &card.oracle_text {
                        // Double space the oracle text so it appears correctly
                        let spaced_oracle_text = oracle_text.replace("\n", "\n\n");
                        e.field("Oracle text:", spaced_oracle_text, false);
                    }

                    if let Some(flavor_text) = &card.flavor_text {
                        e.footer(|ef| {
                            ef.text(flavor_text);
                            ef
                        });
                    }

                    e
                });

                m
            })
            .await;
        true
    } else {
        false
    }
}

async fn mtg_card(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // our search query
    let query = args.rest().to_lowercase();

    if !card_search(ctx, msg, &query).await {
        let mut suggestions = None;

        {
            // Otherwise attempt to get an autocomplete for the search term
            // TODO: factor this into a load suggestions fn
            let data = ctx.data.read().await;

            if let Some(sfclient) = data.get::<SfClientContainer>() {
                let sfclient = sfclient.read().await;
                let _suggestions = sfclient.card_autocomplete(&query).await?;

                if !_suggestions.data.is_empty() {
                    suggestions = Some(_suggestions.data);
                }
            }
        }

        if let Some(suggestions) = suggestions {
            if suggestions.len() == 1 {
                // Special case when we have a single suggestions
                if card_search(ctx, msg, &suggestions[0]).await {
                    return Ok(());
                }
            }

            let _ = msg
                .channel_id
                .send_message(ctx, |m| {
                    let mut builder = MessageBuilder::new();
                    builder.push_bold_line(
                        "I can't let you do that Dave - perhaps you meant one of the following:",
                    );

                    let mut ix = false;

                    for suggestion in suggestions {
                        if ix {
                            builder.push(", ");
                        } else {
                            ix = true;
                        }
                        builder.push_italic_safe(suggestion);
                    }

                    m.content(builder.build());

                    m
                })
                .await;
        }
    }

    Ok(())
}

async fn mtg_set(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let _ = msg.reply(ctx, "set").await;

    Ok(())
}

#[command]
async fn mtg(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        // Return some help!
        return mtg_help(ctx, msg).await;
    }

    let _ = match args.current() {
        Some("card") => {
            args.advance();

            mtg_card(ctx, msg, args).await
        }
        Some("set") => {
            args.advance();

            mtg_set(ctx, msg, args).await
        }
        _ => mtg_card(ctx, msg, args).await,
    };

    Ok(())
}
