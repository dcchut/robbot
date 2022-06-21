use crate::client::make_client;
use crate::models::cards::RawCard;
use anyhow::{anyhow, Result};
use dcc_scryfall::SfClient;

pub(super) struct RemoteCardGateway {
    client: SfClient,
}

impl RemoteCardGateway {
    pub fn new() -> Self {
        Self {
            client: SfClient::from_client(make_client()),
        }
    }

    pub async fn random(&self) -> Result<RawCard> {
        self.client
            .card_random()
            .await
            .map(RawCard::from)
            .map_err(|_| anyhow!("failed to get random card"))
    }

    /// Returns a list of suggestions based on the given query.
    pub async fn suggestions(&self, query: &str) -> Result<Vec<String>> {
        if query.is_empty() {
            return Ok(Vec::new());
        }

        self.client
            .card_autocomplete(query)
            .await
            .map(|li| li.data)
            .map_err(|_| anyhow!("failed to get suggestions for {}", query))
    }

    /// Returns a card with the given name (if it exists).
    pub async fn get_by_name(&self, name: &str) -> Result<Option<RawCard>> {
        // FIXME: scryfall client API returns Result<Card> when it should probably
        //        return Result<Option<RawCard>>.
        if let Ok(card) = self.client.card_named(true, name).await {
            Ok(Some(RawCard::from(card)))
        } else {
            Ok(None)
        }
    }
}
