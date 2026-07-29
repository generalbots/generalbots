diesel::table! {
    bots (id) {
        id -> Uuid,
        name -> Varchar,
        is_public -> Bool,
    }
}

diesel::table! {
    bot_configuration (id) {
        id -> Uuid,
        bot_id -> Uuid,
        key -> Varchar,
        value -> Text,
    }
}
