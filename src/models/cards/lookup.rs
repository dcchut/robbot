use crate::models::cards::Card;
use crate::{Pool, Sqlite};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(super) struct CardLookup {
    search_term: String,
    card_id: i64,
    last_updated: i64,
}

#[derive(Clone, Debug)]
pub(super) struct NewCardLookup {
    query: String,
    card_id: i64,
}

impl NewCardLookup {
    pub fn new(query: String, card_id: i64) -> Self {
        Self { query, card_id }
    }
}

pub(super) struct LocalCardLookup<'pool> {
    pool: &'pool Pool<Sqlite>,
}

impl<'pool> LocalCardLookup<'pool> {
    pub fn new(pool: &'pool Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, lookup: NewCardLookup) -> Result<()> {
        let last_updated = Utc::now().timestamp();
        sqlx::query!(
            "
        INSERT OR IGNORE INTO card_lookups ( search_term, card_id, last_updated )
        VALUES ( ?, ?, ? )
            ",
            lookup.query,
            lookup.card_id,
            last_updated,
        )
        .execute(self.pool)
        .await
        .map(|_| ())
        .with_context(|| format!("failed to insert card lookup with card {}", lookup.card_id))
    }

    /// Clears any old card lookups out of the database.
    async fn clear(&self) -> Result<()> {
        let threshold = (Utc::now() - Duration::days(30)).timestamp();
        sqlx::query!(
            "DELETE FROM card_lookups WHERE last_updated <= ?",
            threshold
        )
        .execute(self.pool)
        .await
        .map(|_| ())
        .with_context(|| "failed to clear stale card lookups")
    }

    async fn _query(&self, query: &str) -> Result<Option<Card>> {
        self.clear().await?;

        sqlx::query_as!(
            Card,
            "
        SELECT cards.id, cards.name, cards.type_line, cards.mana_cost, cards.oracle_text, cards.flavor_text, cards.image_uri
        FROM cards
        INNER JOIN card_lookups
        ON card_lookups.card_id = cards.id
        AND card_lookups.search_term = ?
            ",
            query
        )
            .fetch_optional(self.pool)
            .await
            .with_context(|| format!("failed to get card matching query {}", query))
    }

    pub async fn query<S: AsRef<str>>(&self, query: S) -> Result<Option<Card>> {
        self._query(query.as_ref()).await
    }
}
