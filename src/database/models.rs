use crate::database::schema::api_keys;
use diesel::{Insertable, Queryable};
use uuid::Uuid;

#[derive(Queryable, Insertable)]
#[table_name = "api_keys"]
pub struct ApiKeys {
    pub id: Uuid,
    pub apikey: Vec<u8>,
}
