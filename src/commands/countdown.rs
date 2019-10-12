use crate::models::{Countdown, NewCountdown};
use crate::{SqliteConnectionContainer, OwnersContainer};
use chrono::{TimeZone, Utc, DateTime};
use chrono_humanize::HumanTime;
use diesel::prelude::*;
use log::error;
use serenity::framework::standard::{macros::command, CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

async fn add_countdown(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;

    // is this user an owner?
    if let Some(owner_id) = data.get::<OwnersContainer>() {
        if &msg.author.id == owner_id {
            if let Ok(target) = args.single_quoted::<String>() {
                // Attempt to parse the date argument
                if let Ok(dt) = target.parse::<DateTime<Utc>>() {
                    // get the db connection
                    if let Some(conn) = data.get::<SqliteConnectionContainer>() {
                        let new_countdown = NewCountdown {
                            end: dt.timestamp() as i32,
                            active: true
                        };

                        let conn = conn.lock().await;

                        if let Err(_) = diesel::insert_into(crate::schema::countdowns::table)
                            .values(&new_countdown)
                            .execute(&*conn) {
                            error!("Unable to insert new countdown!");
                        }
                    } else {
                        error!("Unable to get SqliteConnectionContainer");
                    }
                } else {
                    let mut builder = MessageBuilder::new();
                    let response = builder
                        .push_line("Invalid date format!")
                        .push("Example format: ")
                        .push_italic("2014-11-28T21:00:09-07:00")
                        .push(".")
                        .build();


                    let _ = msg.reply(&ctx, response).await;
                }
            }
        }
    }

    Ok(())
}

async fn get_latest_countdown(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(conn) = data.get::<SqliteConnectionContainer>() {
        use crate::schema::countdowns::dsl::*;

        let conn = conn.lock().await;

        let curr_dt = Utc::now();
        let curr_timestamp = curr_dt.timestamp() as i32;

        let results = countdowns
            .filter(active.eq(true))
            .filter(end.ge(curr_timestamp))
            .order(id.desc())
            .first::<Countdown>(&*conn)
            .optional()?;

        // Find the most recently created countdown
        if let Some(countdown) = results {
            let end_dt = Utc.timestamp(countdown.end as i64, 0);

            // How long until the countdown activates?
            let human_difference = format!("{:#}", HumanTime::from(end_dt - curr_dt));

            let mut builder = MessageBuilder::new();
            let response = builder
                .push("The next session is ")
                .push(human_difference)
                .push(".")
                .build();

            let _ = msg.reply(&ctx, response).await;
        } else {
            let _ = msg
                .reply(&ctx, "There are no currently active countdowns!")
                .await;
        }
    } else {
        error!("Could not get SqliteConnection");
    }

    Ok(())
}

#[command]
async fn countdown(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(mode) = args.single::<String>() {
        // add mode?
        if &mode == "add" {
            add_countdown(ctx, msg, args).await
        } else {
            let _ = msg.reply(&ctx, "Invalid mode specified").await;
            Ok(())
        }
    } else {
        get_latest_countdown(ctx, msg).await
    }
}
