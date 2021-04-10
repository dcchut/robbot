use log::info;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::ShardManagerContainer;

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    {
        info!("Received shutdown command from {}", msg.author.name);

        let data = ctx.data.read().await;

        if let Some(manager) = data.get::<ShardManagerContainer>() {
            manager.lock().await.shutdown_all().await;
            let _ = msg.reply(&ctx, "Shutting down!").await;
        } else {
            let _ = msg
                .reply(&ctx, "There was a problem getting the shard manager")
                .await;
        }
    }

    Ok(())
}
