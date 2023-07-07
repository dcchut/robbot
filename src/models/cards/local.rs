use crate::models::cards::{Card, RawCard};
use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

pub(super) struct LocalCardStorage<'pool> {
    pool: &'pool Pool<Sqlite>,
}

impl<'pool> LocalCardStorage<'pool> {
    pub fn new(pool: &'pool Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Returns (if it exists) the card with the given ID from the store.
    pub async fn get(&self, id: i64) -> Result<Card> {
        sqlx::query_as!(
            Card,
            "
        SELECT id, name, type_line, mana_cost, oracle_text, flavor_text, image_uri
        FROM cards
        WHERE id = ?
            ",
            id
        )
        .fetch_one(self.pool)
        .await
        .with_context(|| format!("failed to get card {id}"))
    }

    /// Gets or inserts the given card into the store.
    pub async fn get_or_insert(&self, card: RawCard) -> Result<Card> {
        let mut tx = self.pool.begin().await?;

        // Attempt to get this card by name
        let card_name = card.name.as_str();
        if let Some(card) = sqlx::query_as!(
            Card,
            "
        SELECT id, name, type_line, mana_cost, oracle_text, flavor_text, image_uri
        FROM cards
        WHERE name = ?
            ",
            card_name
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            return Ok(card);
        }

        // Failing that, insert a new row for this card
        let row_id = sqlx::query!(
            "
        INSERT INTO cards ( name, type_line, mana_cost, oracle_text, flavor_text, image_uri)
        VALUES ( ?, ?, ?, ?, ?, ? )
            ",
            card.name,
            card.type_line,
            card.mana_cost,
            card.oracle_text,
            card.flavor_text,
            card.image_uri
        )
        .execute(&mut tx)
        .await
        .map(|result| result.last_insert_rowid())
        .with_context(|| "failed to insert card")?;

        tx.commit().await?;
        self.get(row_id).await
    }
}
