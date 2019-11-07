use serenity::{
    client::Context,
    framework::standard::{Args, CommandResult, macros::command},
    model::channel::Message,
    utils::MessageBuilder,
};

use crate::containers::GatewayContainer;
use crate::models::card::Card;

async fn mtg_help(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg.reply(ctx, "help").await;

    Ok(())
}

async fn display_suggestions(
    ctx: &Context,
    msg: &Message,
    suggestions: Vec<String>,
) -> CommandResult {
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

    Ok(())
}

async fn display_card(ctx: &Context, msg: &Message, card: &Card) -> CommandResult {
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

    Ok(())
}

async fn mtg_random_card(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(gateway) = data.get::<GatewayContainer>() {
        if let Some(card) = gateway.random().await {
            let _ = display_card(ctx, msg, &card).await;
        }
    }

    Ok(())
}

async fn mtg_card(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // our search query
    let query = args.rest().to_lowercase();
    let data = ctx.data.read().await;

    if let Some(gateway) = data.get::<GatewayContainer>() {
        // First, does a plain ol' search work?
        if let Some(card) = gateway.search(&query).await {
            return display_card(ctx, msg, &card).await;
        } else {
            let suggestions = gateway.suggestions(&query).await;

            // If there is a single suggestion, we look that card up
            if suggestions.len() == 1 {
                if let Some(card) = gateway.search(&suggestions[0]).await {
                    return display_card(ctx, msg, &card).await;
                }
            } else if !suggestions.is_empty() {
                return display_suggestions(ctx, msg, suggestions).await;
            }
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
        Some("random") => mtg_random_card(ctx, msg).await,
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
