use reqwest::Client as ReqClient;
use crate::containers::SqliteConnectionContainer;
use crate::models::rocks::update_rocks;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RandomDog {
    message: String,
    status: String,
}

#[command]
async fn dog(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // Returns a random dog picture
    let client = ReqClient::new();

    let res = client.get("https://dog.ceo/api/breeds/image/random")
        .send().await?;

    let dog: RandomDog = res.json().await?;

    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.image(&dog.message);

                e
            });

            m
        }).await?;

    Ok(())
}
