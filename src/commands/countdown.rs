use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
async fn countdown(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg
        .reply(&ctx, "31700 minutes until the next session!")
        .await;

    Ok(())
}
