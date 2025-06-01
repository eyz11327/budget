use crate::{BudgetRecord, UploadDescription};

use super::schema::{description_information, records};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = records)]
#[diesel(primary_key())]
pub struct Record {
    pub id: i64,
    pub amount: f64,
    pub date: NaiveDate,
    pub card: String,
    pub description: String,
    pub event_time: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = records)]
pub struct NewRecord<'a> {
    pub amount: f64,
    pub date: NaiveDate,
    pub card: &'a str,
    pub description: &'a str,
}

impl<'a> From<&'a BudgetRecord> for NewRecord<'a> {
    fn from(record: &'a BudgetRecord) -> Self {
        NewRecord {
            amount: record.amount,
            date: record.date,
            card: &record.card,
            description: &record.description,
        }
    }
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = description_information)]
pub struct Description {
    pub description: String,
    pub primary_information: Option<String>,
    pub secondary_information: Option<String>,
    pub tertiary_information: Option<String>,
    pub additional_information: Option<String>,
    pub event_time: DateTime<Utc>,
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

impl<'a> From<&'a UploadDescription> for NewDescription<'a> {
    fn from(description: &'a UploadDescription) -> Self {
        NewDescription {
            description: &description.description,
            primary_information: &description.primary_information,
            secondary_information: &description.secondary_information,
            tertiary_information: &description.tertiary_information,
            additional_information: &description.additional_information,
        }
    }
}
