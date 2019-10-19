CREATE TABLE cards (
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    type_line TEXT NOT NULL,
    mana_cost TEXT,
    oracle_text TEXT,
    flavor_text TEXT,
    image_uri TEXT
)