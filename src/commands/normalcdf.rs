use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::command;
use probability::distribution::{Gaussian, Distribution};

#[command]
async fn normalcdf(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.current() {
        Some(x) => {
            // try and parse the argument as a float
            let arg = x.parse::<f64>()?;

            let cumulative_prob = {
                let gaussian = Gaussian::new(0.0, 1.0);
                gaussian.distribution(arg)
            };
            let reply = format!("P(Z <= {}) = {}", arg, cumulative_prob);

            msg.reply(ctx, reply).await?;
        }
        _ => {},
    };

    Ok(())
}
