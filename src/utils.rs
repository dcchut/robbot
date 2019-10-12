use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;

/// Used to react to user commands which are invalid in a fundamental way.
pub async fn invalid_command(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg.react(&ctx, "❌").await;

    Ok(())
}
