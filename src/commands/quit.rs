use crate::ShardManagerContainer;
use log::info;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
async fn quit(ctx: &mut Context, msg: &Message) -> CommandResult {
    {
        info!("Received shutdown command from {}", msg.author.name);

        let data = ctx.data.read().await;

        if let Some(manager) = data.get::<ShardManagerContainer>() {
            manager.lock().await.shutdown_all();
            let _ = msg.reply(&ctx, "Shutting down!").await;
        } else {
            let _ = msg
                .reply(&ctx, "There was a problem getting the shard manager")
                .await;
        }
    }

    Ok(())
}
