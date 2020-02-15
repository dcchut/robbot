use probability::distribution::{Distribution, Gaussian};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;

#[command]
async fn normalcdf(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(level) = args.single::<f64>() {
        let desc = {
            let gaussian = Gaussian::new(0.0, 1.0);

            if let Ok(end) = args.single::<f64>() {
                format!("P({} <= Z <= {}) = {}", level, end, gaussian.distribution(end) - gaussian.distribution(level))
            } else {
                format!("P(Z <= {}) = {}", level, gaussian.distribution(level))
            }
        };

        msg.reply(ctx, desc).await?;
    };

    Ok(())
}
