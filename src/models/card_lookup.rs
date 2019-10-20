use crate::schema::card_lookups;
use chrono::Utc;
use diesel::prelude::*;
use diesel::{RunQueryDsl, SqliteConnection};
use serenity::utils::Mutex;
use std::sync::Arc;

#[derive(Queryable, Debug, Clone)]
pub struct CardLookup {
    pub id: i32,
    pub search_term: String,
    pub card_id: i32,
    pub last_updated: i32,
}

#[derive(Insertable)]
#[table_name = "card_lookups"]
struct RawNewCardLookup<'a> {
    pub search_term: &'a str,
    pub card_id: i32,
    pub last_updated: i32,
}

pub struct NewCardLookup {
    pub search_term: String,
    pub card_id: i32,
}

/// Inserts a new card lookup into the DB
pub async fn insert_card_lookup(
    new_card_lookup: &NewCardLookup,
    conn: &Arc<Mutex<SqliteConnection>>,
) -> bool {
    let current_dt = Utc::now();

    let raw = RawNewCardLookup {
        search_term: &new_card_lookup.search_term,
        card_id: new_card_lookup.card_id,
        last_updated: current_dt.timestamp() as i32,
    };

    let conn = conn.lock().await;

    // In practice this _should_ never error
    diesel::insert_into(crate::schema::card_lookups::table)
        .values(&raw)
        .execute(&*conn)
        .is_ok()
}

/// Attempts to locate a card based on the given search term
pub async fn search_card_lookups(
    query: &str,
    conn: &Arc<Mutex<SqliteConnection>>,
) -> Option<crate::models::card::Card> {
    use crate::schema::card_lookups::dsl::*;

    if query.is_empty() {
        return None;
    }

    card_lookups
        .inner_join(crate::schema::cards::table)
        .filter(search_term.eq(query))
        .first::<(
            crate::models::card_lookup::CardLookup,
            crate::models::card::Card,
        )>(&*conn.lock().await)
        .map(|i| i.1)
        .optional()
        .unwrap_or_else(|_| None)
}
