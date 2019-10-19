use crate::containers::SfClientContainer;
use crate::utils::invalid_command;
use lazy_static::lazy_static;
use regex::Regex;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use log::warn;
use serenity::model::channel::Embed;

async fn mtg_help(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg.reply(ctx, "help").await;

    Ok(())
}

async fn mtg_card(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\[(.*?)\]").unwrap();
    }

    let search_term = args.rest();
    let mut cards = Vec::new();

    // Does the search have things of the form [...] in it?
    if RE.is_match(search_term) {
        // Go over every capture
        for cap in RE.captures_iter(search_term) {
            cards.push(String::from(&cap[1]));
        }
    } else {
        cards.push(String::from(search_term));
    }

    // Invalid search somehow
    if cards.is_empty() {
        return invalid_command(ctx, msg).await;
    }

    // Is there a webhook for us to use? (TODO: cache this?)
    let webhooks = msg.channel_id.webhooks(&ctx.http).await?;
    let mut robbot_webhook = None;

    for webhook in webhooks {
        if let Some(name) = &webhook.name {
            if name == "robbot" {
                robbot_webhook = Some(webhook);

                break;
            }
        }
    }

    if robbot_webhook.is_none() {
        warn!("robbot webhook not found!");
    }

    // Build up result
    let mut results = Vec::new();

    {
        // Go an API request for each of the cards
        let data = ctx.data.read().await;

        if let Some(sfclient) = data.get::<SfClientContainer>() {
            let sfclient = sfclient.read().await;

            for card in cards {
                if let Ok(card) = sfclient.card_named(true, &card).await {
                    results.push(card);
                }
            }
        }
    }

    if !results.is_empty() {
        let webhook = robbot_webhook.unwrap();

        // Create an embed for each card
        let mut embeds = Vec::new();

        for card in &results {
            if let Some(image_uris) = &card.print.image_uris {
                embeds.push(Embed::fake(|e| {
                    e.title(&card.gameplay.name)
                        .image(&image_uris.small)
                }));
            }
        }

        let _ = webhook.execute(&ctx.http, false, |w| {
            w.content("Card(s):").embeds(embeds)
        }).await;
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
        _ => {
            mtg_card(ctx, msg, args).await
        }
    };

    Ok(())
}
