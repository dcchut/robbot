use crate::containers::{SfClientContainer, SqliteConnectionContainer};
use crate::models::card::{get_card, insert_card, NewCard};
use crate::models::card_lookup::{insert_card_lookup, search_card_lookups, NewCardLookup};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

async fn mtg_help(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg.reply(ctx, "help").await;

    Ok(())
}

async fn mtg_card(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // our search query
    let query = args.rest().to_lowercase();

    let mut retrieved_card = None;

    {
        let data = ctx.data.read().await;

        if let Some(conn) = data.get::<SqliteConnectionContainer>() {
            // First attempt to search the sqlite cache for this card
            if let Some(cached_card) = search_card_lookups(&query, conn).await {
                retrieved_card = Some(cached_card);
            } else {
                // Load the card using the scryfall API
                if let Some(sfclient) = data.get::<SfClientContainer>() {
                    let sfclient = sfclient.read().await;
                    // We have to use ? here to avoid holding a Box<dyn Error> across an await point
                    let card = sfclient.card_named(true, &query).await?;

                    // Is this card in the DB?
                    if let Some(db_card) = get_card(&card.gameplay.name, conn).await {
                        let card_lookup = NewCardLookup {
                            search_term: query,
                            card_id: db_card.id,
                        };

                        insert_card_lookup(&card_lookup, conn).await;

                        retrieved_card = Some(db_card);
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
                                search_term: query,
                                card_id: db_card.id,
                            };

                            insert_card_lookup(&card_lookup, conn).await;

                            retrieved_card = Some(db_card);
                        }
                    }
                }
            }
        }
    }

    fn unwrap_or_empty(v: &Option<String>) -> &str {
        if let Some(inner) = v {
            inner.as_str()
        } else {
            ""
        }
    }

    if let Some(card) = retrieved_card {
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
                    e.field("Mana cost:", unwrap_or_empty(&card.mana_cost), true);

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
