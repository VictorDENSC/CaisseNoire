table! {
    teams (id) {
        id -> Uuid,
        name -> Varchar,
        admin_password -> Varchar,
        rules -> Array<Jsonb>,
    }
}

table! {
    users (id) {
        id -> Uuid,
        team_id -> Uuid,
        firstname -> Varchar,
        lastname -> Varchar,
        nickname -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
    }
}

joinable!(users -> teams (team_id));

allow_tables_to_appear_in_same_query!(
    teams,
    users,
);
