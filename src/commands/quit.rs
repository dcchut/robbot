use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::info;

use crate::ShardManagerContainer;

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Received shutdown command from {}", msg.author.name);
    let data = ctx.data.read().await;

    let manager = data
        .get::<ShardManagerContainer>()
        .expect("failed to obtain shard manager");

    let _ = msg.reply(&ctx, "Shutting down!").await;
    manager.lock().await.shutdown_all().await;

    Ok(())
}
