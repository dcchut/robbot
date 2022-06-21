use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::invalid_command;
use crate::{NominatimClientContainer, OpenWeatherMapClientContainer};

#[command]
async fn weather(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let nominatim_client = data
        .get::<NominatimClientContainer>()
        .expect("failed to obtain nominatim client");

    if let Ok(Some((display_name, lat, lon))) = nominatim_client.search(args.rest()).await {
        // Now look up the weather at this lat/lon.
        let owm_client = data
            .get::<OpenWeatherMapClientContainer>()
            .expect("failed to open OpenWeatherMap client");

        if let Ok(Some((response, emoji))) = owm_client.get(&lat, &lon).await {
            msg.reply(
                ctx,
                format!(
                    "Currently in {}: {} **{:.2}°C / {:.2}°F**",
                    display_name,
                    emoji,
                    response.main.temp,
                    c_to_f(response.main.temp)
                ),
            )
            .await?;
        }
    } else {
        return invalid_command(ctx, msg).await;
    }

    Ok(())
}

fn c_to_f(temp: f32) -> f32 {
    (temp * (9. / 5.)) + 32.
}
