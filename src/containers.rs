use crate::{AnimalGateway, CardStore, CountdownStore, RockCounter};
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::application::CurrentApplicationInfo;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::models::weather::{NominatimClient, OpenWeatherMapClient};

pub(crate) struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct AppInfoContainer;

impl TypeMapKey for AppInfoContainer {
    type Value = CurrentApplicationInfo;
}

pub struct CardStoreContainer;

impl TypeMapKey for CardStoreContainer {
    type Value = CardStore;
}

pub struct RockCounterContainer;

impl TypeMapKey for RockCounterContainer {
    type Value = RockCounter<'static>;
}

pub struct CountdownStoreContainer;

impl TypeMapKey for CountdownStoreContainer {
    type Value = CountdownStore<'static>;
}

pub struct AnimalGatewayContainer;

impl TypeMapKey for AnimalGatewayContainer {
    type Value = AnimalGateway;
}

pub struct NominatimClientContainer;

impl TypeMapKey for NominatimClientContainer {
    type Value = NominatimClient;
}

pub struct OpenWeatherMapClientContainer;

impl TypeMapKey for OpenWeatherMapClientContainer {
    type Value = OpenWeatherMapClient;
}