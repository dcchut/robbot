use std::sync::Arc;
use std::sync::Mutex;

use chrono::Utc;
use dcc_scryfall::SfClient;
use diesel::prelude::*;
use diesel::{QueryDsl, SqliteConnection};
use serenity::AsyncRwLock;

use crate::models::card::{Card, NewCard};
use crate::models::card_lookup::RawNewCardLookup;

pub(crate) trait CardCache {
    /// Given a query string, return the corresponding card (if it exists in the cache)
    fn get_card_from_query(&self, query: &str) -> Option<Card>;

    /// Insert a new query lookup
    fn insert_query(&self, query: &str, card: &Card) -> bool;

    /// Given the name of a card, return it (if it exists in the cache)
    fn get_card(&self, card_name: &str) -> Option<Card>;

    /// Insert a card into the cache
    fn insert_card(&self, new_card: NewCard) -> Option<Card>;
}

pub(crate) struct SqliteCardCache {
    db: Arc<Mutex<SqliteConnection>>,
}

impl SqliteCardCache {
    pub fn new(db: &Arc<Mutex<SqliteConnection>>) -> Self {
        Self { db: Arc::clone(db) }
    }
}

impl CardCache for SqliteCardCache {
    fn get_card_from_query(&self, query: &str) -> Option<Card> {
        use crate::schema::card_lookups::dsl::*;

        if query.is_empty() {
            return None;
        }

        let conn = self.db.lock().expect("Unable to acquire mutex");

        card_lookups
            .inner_join(crate::schema::cards::table)
            .filter(search_term.eq(query))
            .first::<(
                crate::models::card_lookup::CardLookup,
                crate::models::card::Card,
            )>(&*conn)
            .map(|i| i.1)
            .optional()
            .unwrap_or_else(|_| None)
    }

    fn insert_query(&self, query: &str, card: &Card) -> bool {
        let current_dt = Utc::now();

        let raw = RawNewCardLookup {
            search_term: query,
            card_id: card.id,
            last_updated: current_dt.timestamp() as i32,
        };

        let conn = self.db.lock().expect("Unable to acquire mutex");

        // In practice this _should_ never error
        diesel::insert_into(crate::schema::card_lookups::table)
            .values(&raw)
            .execute(&*conn)
            .is_ok()
    }

    fn get_card(&self, card_name: &str) -> Option<Card> {
        use crate::schema::cards::dsl::*;

        if card_name.is_empty() {
            return None;
        }

        let conn = self.db.lock().expect("Unable to acquire mutex");

        cards
            .filter(name.eq(card_name))
            .first::<crate::models::card::Card>(&*conn)
            .optional()
            .unwrap_or_else(|_| None)
    }

    fn insert_card(&self, new_card: NewCard) -> Option<Card> {
        use crate::schema::cards::dsl::*;

        let conn = self.db.lock().expect("Unable to acquire mutex");

        conn.transaction(|| {
            let inserted_count = diesel::insert_into(cards)
                .values(new_card)
                .execute(&*conn)?;

            if inserted_count != 1 {
                return Ok(None);
            }

            cards.order(id.desc()).first(&*conn).optional()
        })
        .unwrap_or_else(|_| None)
    }
}

pub(crate) struct ScryfallGateway {
    client: Arc<AsyncRwLock<SfClient>>,
    cache: Box<dyn CardCache + Send + Sync>,
}

impl ScryfallGateway {
    pub fn new<C: CardCache + Send + Sync + 'static>(client: SfClient, cache: C) -> Self {
        Self {
            client: Arc::new(AsyncRwLock::new(client)),
            cache: Box::new(cache),
        }
    }

    pub async fn random(&self) -> Option<Card> {
        self.client
            .read()
            .await
            .card_random()
            .await
            .map(Card::from)
            .ok()
    }

    pub async fn suggestions(&self, query: &str) -> Vec<String> {
        if !query.is_empty() {
            let suggestions = self.client.read().await.card_autocomplete(query).await;

            if let Ok(suggestions) = suggestions {
                if !suggestions.data.is_empty() {
                    return suggestions.data;
                }
            }
        }

        Vec::new()
    }

    pub async fn search(&self, query: &str) -> Option<Card> {
        // Check the cache first
        if let card @ Some(_) = self.cache.get_card_from_query(query) {
            return card;
        }

        // Otherwise, fire off a web request
        let lock = self.client.read().await;
        let card = lock.card_named(true, query).await.map_err(|_| ()).ok();

        if let Some(card_model) = card {
            let mut cached_card = self.cache.get_card(&card_model.gameplay.name);

            if cached_card.is_none() {
                // If this card doesn't exist in the cache, then insert it
                let new_card = NewCard {
                    name: String::from(&card_model.gameplay.name),
                    type_line: String::from(&card_model.gameplay.type_line),
                    mana_cost: card_model.gameplay.mana_cost.clone(),
                    oracle_text: card_model.gameplay.oracle_text.clone(),
                    flavor_text: card_model.print.flavor_text.clone(),
                    image_uri: {
                        if let Some(uris) = &card_model.print.image_uris {
                            Some(uris.border_crop.clone())
                        } else {
                            None
                        }
                    },
                };

                cached_card = self.cache.insert_card(new_card);
            }

            // If we still don't have a card by now, theres nothing to be done
            cached_card.as_ref()?;

            let cached_card = cached_card.unwrap();

            // Insert the card lookup
            self.cache.insert_query(query, &cached_card);

            return Some(cached_card);
        }

        None
    }
}
