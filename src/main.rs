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
use serenity::model::id::GuildId;
use serenity::model::interactions::application_command::{
    ApplicationCommandInteractionData, ApplicationCommandOptionType,
};
use serenity::model::interactions::Interaction;
use serenity::model::prelude::application_command::ApplicationCommandInteractionDataOption;
use serenity::utils::MessageBuilder;

mod commands;
mod containers;
mod gateway;
mod models;
mod schema;
mod utils;

struct Handler;

fn is_subcommand<'data>(
    data: &'data ApplicationCommandInteractionData,
    command: &'static str,
    sub_command: &'static str,
) -> Option<&'data ApplicationCommandInteractionDataOption> {
    if data.name == command {
        if let Some(option) = data.options.get(0) {
            if option.name == sub_command {
                return Some(option);
            }
        }
    }
    None
}

fn get_subcommand_value<'data>(
    data: &'data ApplicationCommandInteractionData,
    command: &'static str,
    sub_command: &'static str,
) -> Option<&'data serde_json::Value> {
    if let Some(option) = is_subcommand(data, command, sub_command) {
        if let Some(sub_options) = option.options.get(0) {
            return sub_options.value.as_ref();
        }
    }
    None
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);

        // let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
        //     commands
        //
        // })
        // .await;
        //info!("Set the following global slash commands: {:?}", commands);

        let guild = GuildId(627275384832000007);

        guild
            .create_application_command(&ctx.http, |command| {
                command
                    .name("mtg")
                    .description("MTG card lookup")
                    .create_option(|option| {
                        option
                            .name("random")
                            .description("display a random card")
                            .kind(ApplicationCommandOptionType::SubCommand)
                    })
                    .create_option(|option| {
                        option
                            .name("search")
                            .description("search for a card")
                            .kind(ApplicationCommandOptionType::SubCommand)
                            .create_sub_option(|sub_option| {
                                sub_option
                                    .name("card")
                                    .description("name of the card to search for")
                                    .kind(ApplicationCommandOptionType::String)
                                    .required(true)
                                    .set_autocomplete(true)
                            })
                    })
            })
            .await
            .expect("failed to create guild command");

        guild
            .create_application_command(&ctx.http, |command| {
                command
                    .name("welcome")
                    .description("Welcome a user")
                    .create_option(|option| {
                        option
                            .name("user")
                            .description("The user to welcome")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
            })
            .await
            .expect("failed to create guild command");
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Autocomplete(interaction) => {
                if let Some(value) = get_subcommand_value(&interaction.data, "mtg", "search") {
                    if let Some(txt) = value.as_str() {
                        let mut suggestions = Vec::new();

                        if !txt.is_empty() {
                            let data = ctx.data.read().await;
                            if let Some(gateway) = data.get::<GatewayContainer>() {
                                suggestions.extend(gateway.suggestions(txt).await);
                            }
                        }

                        // Discord only allows us 25 suggestions, so ensure we abide by that.
                        suggestions.truncate(25);

                        let _ = interaction
                            .create_autocomplete_response(&ctx.http, |response| {
                                for suggestion in suggestions {
                                    response.add_string_choice(&suggestion, &suggestion);
                                }
                                response
                            })
                            .await;
                    }
                }
            }
            Interaction::ApplicationCommand(interaction) => {
                if let Some(value) = get_subcommand_value(&interaction.data, "mtg", "search") {
                    if let Some(txt) = value.as_str() {
                        let card = get_single_card(&ctx, txt).await;

                        // Handle the hard case - the card they're looking for isn't here.
                        if let Some(card) = card {
                            let _ = interaction
                                .create_interaction_response(&ctx.http, |response| {
                                    response.interaction_response_data(|data| {
                                        data.create_embed(|e| {
                                            embed_card(e, &card);
                                            e
                                        })
                                    })
                                })
                                .await;
                        } else {
                            let mut suggestions = Vec::new();

                            if !txt.is_empty() {
                                let data = ctx.data.read().await;
                                if let Some(gateway) = data.get::<GatewayContainer>() {
                                    suggestions.extend(gateway.suggestions(txt).await);
                                }
                            }

                            let _ = interaction.create_interaction_response(&ctx.http, |response| {
                                response.interaction_response_data(|data| {
                                    data.content(
                                        {
                                            let mut builder = MessageBuilder::new();
                                            builder.push_bold_line(
                                                "I can't let you do that Dave - perhaps you meant one of the following:",
                                            );

                                            let mut ix = false;

                                            for suggestion in suggestions {
                                                if ix {
                                                    builder.push(", ");
                                                } else {
                                                    ix = true;
                                                }
                                                builder.push_italic_safe(suggestion);
                                            }

                                            builder.build()
                                        }
                                    )
                                })
                            }).await;
                        }
                    }
                }

                if is_subcommand(&interaction.data, "mtg", "random").is_some() {
                    let data = ctx.data.read().await;

                    if let Some(gateway) = data.get::<GatewayContainer>() {
                        if let Some(card) = gateway.random().await {
                            let _ = interaction
                                .create_interaction_response(&ctx.http, |response| {
                                    response.interaction_response_data(|f| {
                                        f.create_embed(|e| {
                                            embed_card(e, &card);
                                            e
                                        })
                                    })
                                })
                                .await;
                        }
                    }
                }
            }
            _ => {}
        }
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

    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

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
        .application_id(application_id)
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
