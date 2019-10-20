use crate::schema::cards;
use diesel::prelude::*;
use diesel::{RunQueryDsl, SqliteConnection};
use serenity::utils::Mutex;
use std::sync::Arc;

#[derive(Queryable, Debug, Clone)]
pub struct Card {
    pub id: i32,
    pub name: String,
    pub type_line: String,
    pub mana_cost: Option<String>,
    pub oracle_text: Option<String>,
    pub flavor_text: Option<String>,
    pub image_uri: Option<String>,
}

impl From<dcc_scryfall::Card> for Card {
    fn from(c: dcc_scryfall::Card) -> Self {
        Self {
            id: 0, // We use a dummy ID here (FIXME)
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

#[derive(Insertable)]
#[table_name = "cards"]
pub struct NewCard {
    pub name: String,
    pub type_line: String,
    pub mana_cost: Option<String>,
    pub oracle_text: Option<String>,
    pub flavor_text: Option<String>,
    pub image_uri: Option<String>,
}

/// Inserts a new card into the DB
pub async fn insert_card(new_card: &NewCard, conn: &Arc<Mutex<SqliteConnection>>) -> Option<Card> {
    use crate::schema::cards::dsl::*;

    let conn = conn.lock().await;

    let inserted_card = conn
        .transaction(|| {
            let inserted_count = diesel::insert_into(cards)
                .values(new_card)
                .execute(&*conn)?;

            if inserted_count != 1 {
                return Ok(None);
            }

            cards.order(id.desc()).first(&*conn).optional()
        })
        .unwrap_or_else(|_| None);

    inserted_card
}

/// Returns a card by its name
pub async fn get_card(query: &str, conn: &Arc<Mutex<SqliteConnection>>) -> Option<Card> {
    use crate::schema::cards::dsl::*;

    if query.is_empty() {
        return None;
    }

    cards
        .filter(name.eq(query))
        .first::<Card>(&*conn.lock().await)
        .optional()
        .unwrap_or_else(|_| None)
}
