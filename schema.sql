create table cards
(
    id INTEGER not null
        primary key,
    name TEXT not null
        unique,
    type_line TEXT not null,
    mana_cost TEXT,
    oracle_text TEXT,
    flavor_text TEXT,
    image_uri TEXT
);

create table card_lookups
(
    id INTEGER not null
        primary key,
    search_term TEXT not null,
    card_id INTEGER not null
        references cards,
    last_updated INTEGER not null
);

create unique index idx_card_lookup_search_term
    on card_lookups (search_term);

create table countdowns
(
    id INTEGER not null
        primary key,
    end INTEGER not null,
    active BOOLEAN not null,
    guild INTEGER default 307765179373060096
);

create table rocks
(
    user_id INTEGER not null
        primary key,
    count INTEGER default 1 not null
);
