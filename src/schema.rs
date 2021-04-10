table! {
    card_lookups (id) {
        id -> Integer,
        search_term -> Text,
        card_id -> Integer,
        last_updated -> Integer,
    }
}

table! {
    cards (id) {
        id -> Integer,
        name -> Text,
        type_line -> Text,
        mana_cost -> Nullable<Text>,
        oracle_text -> Nullable<Text>,
        flavor_text -> Nullable<Text>,
        image_uri -> Nullable<Text>,
    }
}

table! {
    countdowns (id) {
        id -> Integer,
        end -> Integer,
        active -> Bool,
    }
}

table! {
    rocks (user_id) {
        user_id -> Integer,
        count -> Integer,
    }
}

joinable!(card_lookups -> cards (card_id));

allow_tables_to_appear_in_same_query!(card_lookups, cards, countdowns, rocks,);
