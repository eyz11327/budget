use super::schema::{description_information, records};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = records)]
#[diesel(primary_key())]
pub struct Record {
    pub amount: f32,
    pub date: NaiveDate,
    pub card: String,
    pub description: String,
    pub event_time: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = records)]
pub struct NewRecord<'a> {
    pub amount: &'a f32,
    pub date: &'a NaiveDate,
    pub card: &'a str,
    pub description: &'a str,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = description_information)]
pub struct Description {
    pub description: String,
    pub primary_information: Option<String>,
    pub secondary_information: Option<String>,
    pub tertiary_information: Option<String>,
    pub additional_information: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = description_information)]
pub struct NewDescription<'a> {
    pub description: &'a str,
    pub primary_information: &'a str,
    pub secondary_information: &'a str,
    pub tertiary_information: &'a str,
    pub additional_information: &'a str,
}
