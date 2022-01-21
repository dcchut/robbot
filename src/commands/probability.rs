use crate::commands::invalid_command;
use crate::models::probability::GaussianDist;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;

#[command]
async fn normalcdf(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let dist = GaussianDist::new();

    if let Ok(level) = args.single::<f64>() {
        let response = if let Ok(upper) = args.single::<f64>() {
            dist.normal_cdf(level, upper)
        } else {
            dist.normal_cdf_one_sided(level)
        };
        msg.reply(ctx, response).await?;
    } else {
        invalid_command(ctx, msg).await?;
    }

    Ok(())
}
