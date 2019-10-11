#![feature(async_closure)]

use dotenv;
use log::info;
use serenity::{
    client::{bridge::gateway::ShardManager, Client},
    framework::standard::{macros::group, StandardFramework},
    prelude::{EventHandler, TypeMapKey},
    utils::Mutex,
};
use std::{collections::HashSet, env, sync::Arc};

use commands::quit::*;

mod commands;

// Our custom event handler
struct Handler;
impl EventHandler for Handler {}

// Keep a handle to our shard manager
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group]
#[commands(quit)]
struct General;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .await
        .expect("Error creating client");

    // Store a reference to the shard manager in our client data
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    // Get the current owners of the bot
    let owners = match client
        .cache_and_http
        .http
        .get_current_application_info()
        .await
    {
        Ok(info) => {
            let mut set = HashSet::new();
            info!("{} set as bot owner", info.owner.name);
            set.insert(info.owner.id);

            set
        }
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    client
        .with_framework(
            StandardFramework::new()
                .configure(|c| c.owners(owners).prefix("~"))
                .group(&GENERAL_GROUP),
        )
        .await;

    // Start listening for events
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
