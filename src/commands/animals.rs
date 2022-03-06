use crate::AnimalGatewayContainer;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;

#[command]
async fn dog(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let animal_gateway = data
        .get::<AnimalGatewayContainer>()
        .expect("failed to obtain animal gateway");

    let url = match args.current() {
        Some("golden") => animal_gateway.get_golden().await?,
        _ => animal_gateway.get_dog().await?,
    };

    msg.reply(ctx, url).await?;
    Ok(())
}

#[command]
async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let animal_gateway = data
        .get::<AnimalGatewayContainer>()
        .expect("failed to obtain animal gateway");

    let url = animal_gateway.get_cat().await?;
    msg.reply(ctx, url).await?;
    Ok(())
}
