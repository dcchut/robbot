#[macro_use]
extern crate diesel;

use std::collections::HashSet;
use std::{env, sync::Arc};

use dcc_scryfall::SfClient;
use diesel::{Connection, SqliteConnection};
use serenity::{
    async_trait,
    client::Client,
    framework::standard::{macros::group, StandardFramework},
    prelude::*,
};

use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use commands::{
    countdown::*, dig::*, dog::*, help::*, mtg::*, normalcdf::*, python::*, quit::*, rust::*,
};

use crate::containers::{
    ApplicationInfoContainer, GatewayContainer, ShardManagerContainer, SqliteConnectionContainer,
};
use crate::gateway::{ScryfallGateway, SqliteCardCache};
use serenity::http::Http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;

mod commands;
mod containers;
mod gateway;
mod models;
mod schema;
mod utils;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(quit, countdown, normalcdf, rust, rust_raw, py, py_raw, dig, dog, cat)]
struct General;

#[group]
#[commands(mtg)]
struct Mtg;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");

    // Establish a DB connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = Arc::new(std::sync::Mutex::new(
        SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url)),
    ));
    let cache = SqliteCardCache::new(&conn);

    let sfclient = SfClient::new();

    // Construct our scryfall gateway object
    let gateway = ScryfallGateway::new(sfclient, cache);

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, current_application_info) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("~"))
        .group(&GENERAL_GROUP)
        .group(&MTG_GROUP)
        .help(&MY_HELP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<SqliteConnectionContainer>(conn);
        data.insert::<ApplicationInfoContainer>(current_application_info);
        data.insert::<GatewayContainer>(gateway);
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    // Start listening for events
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
