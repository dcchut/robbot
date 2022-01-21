use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;

pub(crate) mod animals;
pub(crate) mod countdown;
pub(crate) mod dig;
pub(crate) mod help;
pub(crate) mod mtg;
pub(crate) mod probability;
pub(crate) mod quit;
pub(crate) mod sandboxes;
pub(crate) mod weather;

/// Used to react to user commands which are invalid in a fundamental way.
pub(crate) async fn invalid_command(ctx: &Context, msg: &Message) -> CommandResult {
    let _ = msg.react(ctx, '❌').await;

    Ok(())
}
