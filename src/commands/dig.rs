use crate::containers::SqliteConnectionContainer;
use crate::models::rocks::update_rocks;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::error;

#[command]
async fn dig(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(conn) = data.get::<SqliteConnectionContainer>() {
        let count = update_rocks(msg.author.id.0, conn).await;
        msg.reply(&*ctx, format!("You have dug {} times", count))
            .await?;
    } else {
        error!("Could not get SqliteConnection");
    }

    Ok(())
}
