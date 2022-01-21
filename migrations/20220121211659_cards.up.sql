CREATE TABLE IF NOT EXISTS cards
(
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    type_line TEXT NOT NULL,
    mana_cost TEXT,
    oracle_text TEXT,
    flavor_text TEXT,
    image_uri TEXT
);

CREATE TABLE card_lookups (
    search_term TEXT NOT NULL PRIMARY KEY,
    card_id INTEGER NOT NULL,
    last_updated INTEGER NOT NULL,
    FOREIGN KEY(card_id) REFERENCES cards(id)
);
