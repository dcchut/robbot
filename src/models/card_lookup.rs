use diesel::{Insertable, Queryable};

use crate::schema::card_lookups;

#[derive(Queryable, Debug, Clone)]
pub struct CardLookup {
    pub id: i32,
    pub search_term: String,
    pub card_id: i32,
    pub last_updated: i32,
}

#[derive(Insertable)]
#[table_name = "card_lookups"]
pub(crate) struct RawNewCardLookup<'a> {
    pub search_term: &'a str,
    pub card_id: i32,
    pub last_updated: i32,
}
