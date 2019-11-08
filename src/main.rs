#[macro_use]
extern crate diesel;

use std::collections::HashSet;
use std::sync::Mutex;
use std::{env, sync::Arc};

use dcc_scryfall::SfClient;
use diesel::{Connection, SqliteConnection};
use dotenv;
use serenity::{
    client::Client,
    framework::standard::{macros::group, StandardFramework},
    prelude::EventHandler,
};

use commands::{countdown::*, help::*, mtg::*, quit::*};

use crate::containers::{
    ApplicationInfoContainer, GatewayContainer, ShardManagerContainer, SqliteConnectionContainer,
};
use crate::gateway::{ScryfallGateway, SqliteCardCache};

mod commands;
mod containers;
mod gateway;
mod models;
mod schema;
mod utils;

// Our custom event handler
struct Handler;

impl EventHandler for Handler {}

#[group]
#[commands(quit, countdown)]
struct General;

#[group]
#[commands(mtg)]
struct Mtg;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    // Establish a DB connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = Arc::new(Mutex::new(
        SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url)),
    ));
    let cache = SqliteCardCache::new(&conn);

    let sfclient = SfClient::new();

    // Construct our gateway object
    let gateway = ScryfallGateway::new(sfclient, cache);

    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .await
        .expect("Error creating client");

    // Get the current owners of the bot
    let (owners, current_application_info) = match client
        .cache_and_http
        .http
        .get_current_application_info()
        .await
    {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info)
        }
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<SqliteConnectionContainer>(conn);
        data.insert::<ApplicationInfoContainer>(current_application_info);
        data.insert::<GatewayContainer>(gateway);
    }

    client
        .with_framework(
            StandardFramework::new()
                .configure(|c| c.prefix("~").owners(owners))
                .group(&GENERAL_GROUP)
                .group(&MTG_GROUP)
                .help(&MY_HELP),
        )
        .await;

    // Start listening for events
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
