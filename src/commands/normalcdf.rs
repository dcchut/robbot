use probability::distribution::{Distribution, Gaussian};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;

#[command]
async fn normalcdf(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Some(Ok(level)) = args.current().map(str::parse::<f64>) {
        let cumulative_prob = {
            let gaussian = Gaussian::new(0.0, 1.0);
            gaussian.distribution(level)
        };
        let reply = format!("P(Z <= {}) = {}", level, cumulative_prob);

        msg.reply(ctx, reply).await?;
    };

    Ok(())
}
