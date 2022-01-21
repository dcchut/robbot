use crate::models::cards::local::LocalCardStorage;
use crate::models::cards::lookup::{LocalCardLookup, NewCardLookup};
use crate::models::cards::remote::RemoteCardGateway;
use anyhow::Result;
use sqlx::{Pool, Sqlite};

mod local;
mod lookup;
mod remote;

#[derive(Debug, Clone)]
pub struct Card {
    pub id: i64,
    pub name: String,
    pub type_line: String,
    pub mana_cost: Option<String>,
    pub oracle_text: Option<String>,
    pub flavor_text: Option<String>,
    pub image_uri: Option<String>,
}

#[derive(Debug, Clone)]
struct RawCard {
    pub name: String,
    pub type_line: String,
    pub mana_cost: Option<String>,
    pub oracle_text: Option<String>,
    pub flavor_text: Option<String>,
    pub image_uri: Option<String>,
}

impl From<dcc_scryfall::Card> for RawCard {
    fn from(c: dcc_scryfall::Card) -> Self {
        Self {
            name: c.gameplay.name,
            type_line: c.gameplay.type_line,
            mana_cost: c.gameplay.mana_cost,
            oracle_text: c.gameplay.oracle_text,
            flavor_text: c.print.flavor_text,
            image_uri: {
                if let Some(img) = c.print.image_uris {
                    Some(img.border_crop)
                } else {
                    None
                }
            },
        }
    }
}

pub struct CardStore {
    lookups: LocalCardLookup<'static>,
    local: LocalCardStorage<'static>,
    remote: RemoteCardGateway,
}

impl CardStore {
    pub fn new(pool: &'static Pool<Sqlite>) -> Self {
        Self {
            lookups: LocalCardLookup::new(pool),
            local: LocalCardStorage::new(pool),
            remote: RemoteCardGateway::new(),
        }
    }

    /// Fetches a random MTG card.
    pub async fn random(&self) -> Result<Card> {
        let card = self.remote.random().await?;
        self.local.get_or_insert(card).await
    }

    /// Returns a list of suggestions based on the given query.
    pub async fn suggestions<S: AsRef<str>>(&self, query: S) -> Result<Vec<String>> {
        self.remote.suggestions(query.as_ref()).await
    }

    async fn _search(&self, query: &str) -> Result<Option<Card>> {
        if let card @ Some(_) = self.lookups.query(query).await? {
            return Ok(card);
        }

        // Otherwise perform a remote lookup
        if let Some(raw_card) = self.remote.get_by_name(query).await? {
            let card = self.local.get_or_insert(raw_card).await?;
            self.lookups
                .insert(NewCardLookup::new(query.to_string(), card.id))
                .await?;
            return Ok(Some(card));
        }

        Ok(None)
    }

    pub async fn search<S: AsRef<str>>(&self, query: S) -> Result<Option<Card>> {
        self._search(query.as_ref()).await
    }
}
