use chrono::{DateTime, Utc};
use serde_derive::Deserialize;
use surrealdb::sql::Thing;

#[derive(Clone, Debug)]
pub struct User {
    pub uid : String,
    pub email : String,
    pub password : String,
    pub role : String
}

#[derive(Debug, Deserialize)]
pub struct UserDB {
    pub id: Option<Thing>,
    pub key: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub update_date: Option<DateTime<Utc>>,
    pub create_date: Option<DateTime<Utc>>,
    pub update_by: Option<String>,
    pub create_by: Option<String>
}
