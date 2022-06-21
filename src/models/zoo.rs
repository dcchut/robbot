use crate::client::make_client;
use anyhow::{Context, Result};
use reqwest::Client as ReqClient;
use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Dog {
    message: String,
}

#[derive(Debug, Deserialize)]
struct Cat {
    url: String,
}

pub struct AnimalGateway {
    client: ReqClient,
}

impl AnimalGateway {
    pub fn new() -> Self {
        Self {
            client: make_client(),
        }
    }

    async fn _get<T: DeserializeOwned>(&self, url: &'static str) -> Result<T> {
        self.client
            .get(url)
            .send()
            .await?
            .json()
            .await
            .with_context(|| "failed to get dog")
    }

    /// Returns the URL of an image of a golden retriever
    pub async fn get_golden(&self) -> Result<String> {
        self._get::<Dog>("https://dog.ceo/api/breed/retriever/golden/images/random")
            .await
            .map(|dog| dog.message)
    }

    /// Returns the URL of an image of a dog
    pub async fn get_dog(&self) -> Result<String> {
        self._get::<Dog>("https://dog.ceo/api/breeds/image/random")
            .await
            .map(|dog| dog.message)
    }

    /// Returns the URL of an image of a cat
    pub async fn get_cat(&self) -> Result<String> {
        self._get::<Cat>("https://cataas.com/cat?json=True")
            .await
            .map(|cat| format!("https://cataas.com{}", cat.url))
    }
}
