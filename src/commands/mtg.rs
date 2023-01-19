use crate::models::cards::Card;
use crate::CardStoreContainer;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

pub fn embed_card(e: &mut CreateEmbed, card: &Card) {
    e.title(&card.name);

    // If we have card art URI's, include them.
    if let Some(uri) = &card.image_uri {
        e.image(uri);
    }

    e.field("Type:", &card.type_line, true);

    if let Some(mana_cost) = &card.mana_cost {
        if !mana_cost.is_empty() {
            e.field("Mana cost:", mana_cost, true);
        }
    }

    if let Some(oracle_text) = &card.oracle_text {
        // Double space the oracle text so it appears correctly
        let spaced_oracle_text = oracle_text.replace('\n', "\n\n");
        e.field("Oracle text:", spaced_oracle_text, false);
    }

    if let Some(flavor_text) = &card.flavor_text {
        e.footer(|ef| {
            ef.text(flavor_text);
            ef
        });
    }
}

async fn display_card(ctx: &Context, msg: &Message, card: &Card) -> CommandResult {
    // Create an embed for this card
    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                embed_card(e, card);
                e
            });

            m
        })
        .await;

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

#[command]
async fn mtg(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;

    let card_store = data
        .get::<CardStoreContainer>()
        .expect("failed to obtain card store");

    match args.current() {
        Some("random") => {
            display_card(ctx, msg, &card_store.random().await?).await?;
        }
        _ => {
            if args.current() == Some("card") {
                args.advance();
            }
            let query = args.rest().to_lowercase();

            if let Some(card) = card_store.search(&query).await? {
                display_card(ctx, msg, &card).await?;
            } else {
                let suggestions = card_store.suggestions(&query).await?;

                // If there's a single suggestion then just show it as per normal.
                if suggestions.len() == 1 {
                    if let Some(card) = card_store.search(&suggestions[0]).await? {
                        display_card(ctx, msg, &card).await?;
                    }
                } else if !suggestions.is_empty() {
                    display_suggestions(ctx, msg, suggestions).await?;
                }
            }
        }
    };

    Ok(())
}
