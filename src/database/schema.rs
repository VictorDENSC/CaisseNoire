table! {
    sanctions (id) {
        id -> Uuid,
        user_id -> Uuid,
        team_id -> Uuid,
        sanction_info -> Jsonb,
        created_at -> Date,
    }
}

table! {
    teams (id) {
        id -> Uuid,
        name -> Varchar,
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
        login -> Varchar,
        password -> Varchar,
        email -> Nullable<Varchar>,
        is_admin -> Bool,
    }
}

joinable!(sanctions -> users (user_id));

allow_tables_to_appear_in_same_query!(sanctions, teams, users,);
