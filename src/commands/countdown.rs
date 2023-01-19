use crate::commands::invalid_command;
use crate::containers::AppInfoContainer;
use crate::CountdownStoreContainer;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use tracing::info;

async fn ensure_in_guild(ctx: &Context, msg: &Message) -> Option<i64> {
    // FIXME: this is an unnecessary restriction, just a little annoying to work around it.
    if msg.guild_id.is_none() {
        let _ = msg
            .reply(ctx, "You can't run this command in this context")
            .await;
        None
    } else {
        Some(msg.guild_id.unwrap().0 as i64)
    }
}

/// Returns the next countdown that will go off.
async fn next_countdown(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let guild_id = ensure_in_guild(ctx, msg)
        .await
        .ok_or_else(|| anyhow!("not in guild"))?;
    let countdown_store = data
        .get::<CountdownStoreContainer>()
        .expect("failed to obtain countdown store");

    let now = Utc::now();
    match countdown_store
        .get_first_after(now.timestamp(), guild_id)
        .await?
    {
        Some(countdown) => {
            let _ = msg.reply(ctx, countdown.as_pretty_string(&now)).await;
        }
        None => {
            let _ = msg
                .reply(ctx, "There are no currently active countdowns!")
                .await;
        }
    }

    Ok(())
}

async fn list_countdowns(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let guild_id = ensure_in_guild(ctx, msg)
        .await
        .ok_or_else(|| anyhow!("not in guild"))?;
    let countdown_store = data
        .get::<CountdownStoreContainer>()
        .expect("failed to obtain countdown store");

    let now = Utc::now();

    let most_recent_countdowns: Vec<_> = countdown_store
        .get_after(now.timestamp(), guild_id, 5)
        .await?
        .into_iter()
        .map(|countdown| format!("  - {}", countdown.as_pretty_string(&now)))
        .collect();

    let response = if most_recent_countdowns.is_empty() {
        String::from("There are no currently active countdowns!")
    } else {
        format!(
            "Currently active countdowns:\n{}",
            most_recent_countdowns.join("\n")
        )
    };

    let _ = msg.reply(ctx, response).await;
    Ok(())
}

async fn add_countdown(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;

    // For now only the app owner can add countdowns
    let app_info = data
        .get::<AppInfoContainer>()
        .expect("failed to obtain app info");
    if app_info.owner.id != msg.author.id {
        return invalid_command(ctx, msg).await;
    }

    let guild_id = ensure_in_guild(ctx, msg)
        .await
        .ok_or_else(|| anyhow!("not in guild"))?;

    match args.single_quoted::<DateTime<Utc>>() {
        Ok(dt) => {
            let countdown_store = data
                .get::<CountdownStoreContainer>()
                .expect("failed to obtain countdown store");

            countdown_store
                .insert(dt.timestamp(), guild_id)
                .await
                .expect("failed to insert countdown");

            info!(
                "inserted countdown ending at {} in {}",
                dt.timestamp(),
                guild_id
            );
            let _ = msg.react(ctx, '👍').await;
        }
        Err(_) => {
            let _ = msg
                .reply(
                    ctx,
                    "Invalid date format!\nexample format: _2014-11-28T21:00:09-07:00_",
                )
                .await;
        }
    }

    Ok(())
}

#[command]
async fn countdown(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(mode) = args.single::<String>() {
        match mode.as_str() {
            "add" => add_countdown(ctx, msg, args).await,
            "list" => list_countdowns(ctx, msg).await,
            _ => invalid_command(ctx, msg).await,
        }
    } else {
        next_countdown(ctx, msg).await
    }
}
