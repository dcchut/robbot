use std::sync::Arc;
use std::sync::Mutex;

use diesel::SqliteConnection;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::prelude::CurrentApplicationInfo;
use serenity::prelude::TypeMapKey;

use crate::gateway::ScryfallGateway;

pub(crate) struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<serenity::utils::Mutex<ShardManager>>;
}

pub(crate) struct SqliteConnectionContainer;

impl TypeMapKey for SqliteConnectionContainer {
    type Value = Arc<Mutex<SqliteConnection>>;
}

pub(crate) struct ApplicationInfoContainer;

impl TypeMapKey for ApplicationInfoContainer {
    type Value = CurrentApplicationInfo;
}

pub(crate) struct GatewayContainer;

impl TypeMapKey for GatewayContainer {
    type Value = ScryfallGateway;
}
