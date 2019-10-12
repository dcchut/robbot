use diesel::SqliteConnection;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::prelude::CurrentApplicationInfo;
use serenity::prelude::TypeMapKey;
use serenity::utils::Mutex;
use std::sync::Arc;

// Keep a handle to our shard manager
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

// Keep a handle to our sqlite connection
pub struct SqliteConnectionContainer;

impl TypeMapKey for SqliteConnectionContainer {
    type Value = Arc<Mutex<SqliteConnection>>;
}

pub struct ApplicationInfoContainer;

impl TypeMapKey for ApplicationInfoContainer {
    type Value = CurrentApplicationInfo;
}
