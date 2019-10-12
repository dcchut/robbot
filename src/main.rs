#![feature(async_closure)]

#[macro_use]
extern crate diesel;

use std::{collections::HashSet, env, sync::Arc};

use dotenv;
use log::info;
use serenity::{
    client::bridge::gateway::ShardManager,
    client::Client,
    framework::standard::{macros::group, StandardFramework},
    prelude::{EventHandler, TypeMapKey},
    utils::Mutex,
};

use commands::{countdown::*, help::*, quit::*};
use diesel::{Connection, SqliteConnection};
use serenity::model::id::UserId;

mod commands;
mod models;
mod schema;

// Our custom event handler
struct Handler;

impl EventHandler for Handler {}

// Keep a handle to our shard manager
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

// Keep a handle to our sqlite connection
struct SqliteConnectionContainer;

impl TypeMapKey for SqliteConnectionContainer {
    type Value = Arc<Mutex<SqliteConnection>>;
}

struct OwnersContainer;

impl TypeMapKey for OwnersContainer {
    type Value = UserId;
}

#[group]
#[commands(quit, countdown)]
struct General;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    // Establish a DB connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .await
        .expect("Error creating client");

    // Get the current owners of the bot
    let (owners, owner_id) = match client
        .cache_and_http
        .http
        .get_current_application_info()
        .await
        {
            Ok(info) => {
                let mut set = HashSet::new();
                info!("{} set as bot owner", info.owner.name);
                set.insert(info.owner.id);

                (set, info.owner.id)
            }
            Err(why) => panic!("Couldn't get application info: {:?}", why),
        };

    // Store a reference to the shard manager in our client data
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<SqliteConnectionContainer>(Arc::new(Mutex::new(conn)));
        data.insert::<OwnersContainer>(owner_id);
    }



    client
        .with_framework(
            StandardFramework::new()
                .configure(|c| c.owners(owners).prefix("~"))
                .group(&GENERAL_GROUP)
                .help(&MY_HELP),
        )
        .await;

    // Start listening for events
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
