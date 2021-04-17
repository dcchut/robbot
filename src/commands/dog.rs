use reqwest::Client as ReqClient;
use serde::Deserialize;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;

#[derive(Debug, Deserialize)]
struct RandomDog {
    message: String,
    status: String,
}

#[command]
async fn dog(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // Returns a random dog picture
    let client = ReqClient::new();

    let res = client
        .get("https://dog.ceo/api/breeds/image/random")
        .send()
        .await?;

    let dog: RandomDog = res.json().await?;

    msg.reply(ctx, &dog.message).await?;

    Ok(())
}
