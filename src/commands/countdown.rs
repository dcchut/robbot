use chrono::{DateTime, Utc};
use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use tracing::error;

use crate::containers::{ApplicationInfoContainer, SqliteConnectionContainer};
use crate::models::countdown::{get_countdowns, get_first_countdown, insert_countdown};
use crate::utils::invalid_command;

async fn add_countdown(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;

    // For now, only the application owner can add a countdown
    if let Some(application_info) = data.get::<ApplicationInfoContainer>() {
        if application_info.owner.id != msg.author.id {
            // Yikes, borrow checker.
            std::mem::drop(data);

            return invalid_command(ctx, msg).await;
        }
    }

    if let Ok(dt) = args.single_quoted::<DateTime<Utc>>() {
        // Get the DB connection
        match data.get::<SqliteConnectionContainer>() {
            Some(conn) => {
                if insert_countdown(dt.timestamp() as i32, conn).await {
                    let _ = msg.react(&*ctx, '👍').await;
                } else {
                    error!("Unable to insert countdown with dt {}", dt);
                }
            }
            _ => {
                error!("Unable to get SqliteConnectionContainer");
            }
        };
    } else {
        // User specified an invalid date format
        let _ = msg
            .reply(
                &*ctx,
                "Invalid date format!\nExample format: _2014-11-28T21:00:09-07:00_.",
            )
            .await;
    }

    Ok(())
}

async fn get_next_countdown(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(conn) = data.get::<SqliteConnectionContainer>() {
        let current_dt = Utc::now();

        match get_first_countdown(&current_dt, conn).await {
            Some(cd) => {
                let _ = msg.reply(&*ctx, cd.as_pretty_string(&current_dt)).await;
            }
            None => {
                let _ = msg
                    .reply(&*ctx, "There are no currently active countdowns!")
                    .await;
            }
        }
    } else {
        error!("Could not get SqliteConnection");
    }

    Ok(())
}

async fn get_countdown_list(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(conn) = data.get::<SqliteConnectionContainer>() {
        let current_dt = Utc::now();

        // Get the 5 most recent countdowns
        let human_countdowns = get_countdowns(5, &current_dt, conn)
            .await
            .into_iter()
            .map(|cd| format!("  - {}", cd.as_pretty_string(&current_dt)))
            .collect::<Vec<_>>();

        let response = if human_countdowns.is_empty() {
            String::from("There are no currently active countdowns!")
        } else {
            format!(
                "Currently active countdowns:\n{}",
                human_countdowns.join("\n")
            )
        };

        let _ = msg.reply(&*ctx, response).await;
    } else {
        error!("Could not get SqliteConnection");
    }

    Ok(())
}

#[command]
async fn countdown(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(mode) = args.single::<String>() {
        match mode.as_str() {
            "add" => add_countdown(ctx, msg, args).await,
            "list" => get_countdown_list(ctx, msg).await,
            _ => invalid_command(ctx, msg).await,
        }
    } else {
        get_next_countdown(ctx, msg).await
    }
}
