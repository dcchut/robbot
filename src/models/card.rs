use diesel::{Insertable, Queryable};

use crate::schema::cards;

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
