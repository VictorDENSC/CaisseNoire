table! {
    teams (id) {
        id -> Uuid,
        name -> Varchar,
        rules -> Array<Jsonb>,
    }
}
