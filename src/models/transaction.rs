use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::schema::transactions;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub id: i32,
    pub user_id: i32,
    pub description: String,
    pub clacks: i32,
    pub modified_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub user_id: i32,
    pub description: String,
    pub clacks: i32,
}
