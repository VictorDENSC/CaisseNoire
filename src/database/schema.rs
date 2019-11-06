table! {
    sanctions (id) {
        id -> Uuid,
        user_id -> Uuid,
        team_id -> Uuid,
        sanction_info -> Jsonb,
        price -> Float4,
        created_at -> Date,
    }
}

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

joinable!(sanctions -> users (user_id));

allow_tables_to_appear_in_same_query!(sanctions, teams, users,);
