use crate::commands::{
    animals::*, countdown::*, dig::*, help::*, mtg::*, probability::*, quit::*, sandboxes::*,
    weather::*,
};
use crate::containers::{
    AnimalGatewayContainer, AppInfoContainer, CardStoreContainer, CountdownStoreContainer,
    NominatimClientContainer, OpenWeatherMapClientContainer, RockCounterContainer,
    ShardManagerContainer,
};
use crate::models::cards::CardStore;
use crate::models::countdowns::CountdownStore;
use crate::models::rocks::RockCounter;
use crate::models::zoo::AnimalGateway;

use crate::models::weather::{NominatimClient, OpenWeatherMapClient};
use anyhow::Result;
use serde::Deserialize;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::application::CurrentApplicationInfo;
use serenity::model::id::UserId;
use serenity::prelude::GatewayIntents;
use serenity::Client;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};
use std::collections::HashSet;

mod client;
mod commands;
mod containers;
mod models;

#[group]
#[commands(
    countdown, dig, dog, cat, normalcdf, py, py_raw, rust, rust_raw, quit, weather
)]
struct General;

#[group]
#[commands(mtg)]
struct Mtg;

#[derive(Deserialize, Debug)]
struct Config {
    database_url: String,
    discord_token: String,
    discord_application_id: u64,
    openweather_api_key: String,
}

fn setup_app() -> Result<()> {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();

    Ok(())
}

async fn setup_db_pool(config: &Config) -> Result<&'static Pool<Sqlite>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    Ok(Box::leak(Box::new(pool)))
}

async fn get_bot_info(token: &str) -> (HashSet<UserId>, CurrentApplicationInfo) {
    let http = Http::new(token);

    match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info)
        }
        Err(why) => panic!("Could not access application info: {why:?}"),
    }
}

async fn build_client(config: &Config, pool: &'static Pool<Sqlite>) -> Client {
    let (owners, current_app_info) = get_bot_info(&config.discord_token).await;

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("~"))
        .group(&GENERAL_GROUP)
        .group(&MTG_GROUP)
        .help(&MY_HELP);

    let client = Client::builder(
        &config.discord_token,
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
    .framework(framework)
    .application_id(config.discord_application_id)
    .await
    .expect("error creating serenity client");

    // Serenity could be using TypeID's for this purpose but instead requires a wrapper
    // struct implementing TypeMapKey.  Unclear what problem motivated this choice.
    {
        let mut data = client.data.write().await;
        data.insert::<AppInfoContainer>(current_app_info);
        data.insert::<CardStoreContainer>(CardStore::new(pool));
        data.insert::<RockCounterContainer>(RockCounter::new(pool));
        data.insert::<CountdownStoreContainer>(CountdownStore::new(pool));
        data.insert::<AnimalGatewayContainer>(AnimalGateway::new());
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<NominatimClientContainer>(NominatimClient::new());
        data.insert::<OpenWeatherMapClientContainer>(OpenWeatherMapClient::new(
            &config.openweather_api_key,
        ));
    }

    client
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_app()?;

    let config: Config = envy::from_env()?;
    let pool = setup_db_pool(&config).await?;
    let mut client = build_client(&config, pool).await;

    // Set ctrl+c handler so we can shut down the running bot
    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        eprintln!("client error:  {why:?}")
    }

    Ok(())
}
