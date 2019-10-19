CREATE TABLE card_lookups (
    id INTEGER NOT NULL PRIMARY KEY,
    search_term TEXT NOT NULL,
    card_id INTEGER NOT NULL,
    last_updated INTEGER NOT NULL,
    FOREIGN KEY(card_id) REFERENCES cards(id)
);

CREATE UNIQUE INDEX idx_card_lookup_search_term
ON card_lookups(search_term);