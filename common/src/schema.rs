table! {
    tiles (id) {
        id -> Unsigned<Integer>,
        latitude -> Integer,
        longitude -> Integer,
        elevation_data -> Mediumblob,
        imagery_data -> Mediumblob,
    }
}
