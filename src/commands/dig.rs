use crate::RockCounterContainer;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;

#[command]
async fn dig(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;

    let rock_counter = data
        .get::<RockCounterContainer>()
        .expect("failed to obtain rock counter");

    let count = rock_counter.increment(msg.author.id.0 as i64).await?;
    let _ = msg
        .reply(ctx, format!("You have dug {} times", count))
        .await;

    Ok(())
}
