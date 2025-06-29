use crate::{BudgetRecord, UploadDescription};

use super::models::*;
use diesel::prelude::*;

fn generate_database_url(secret_config: serde_json::Value) -> String {
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        secret_config["database"]["username"],
        secret_config["database"]["password"],
        secret_config["database"]["host"],
        secret_config["database"]["port"],
        secret_config["database"]["name"]
    );
    return url.replace('"', "").to_string();
}

pub fn establish_connection(secret_config: serde_json::Value) -> PgConnection {
    let database_url = generate_database_url(secret_config);
    PgConnection::establish(&database_url).unwrap_or_else(|_| {
        panic!(
            "Error connecting to database. Ensure the database URL is valid. URL: {database_url}"
        )
    })
}

pub fn insert_records<'a>(
    connection: &mut PgConnection,
    records: &'a [BudgetRecord],
) -> QueryResult<usize> {
    use super::schema::records;
    let insertable_records: Vec<NewRecord> = records.iter().map(|r| r.into()).collect();
    diesel::insert_into(records::table)
        .values(&insertable_records)
        .execute(connection)
}

pub fn select_descriptions(connection: &mut PgConnection) -> Vec<super::models::Description> {
    use super::schema::description_information;

    let results = description_information::table
        .select(Description::as_select())
        .load(connection)
        .expect("Error loading descriptions");
    return results;
}

pub fn insert_description<'a>(
    connection: &mut PgConnection,
    descriptions: &'a [UploadDescription],
) -> QueryResult<usize> {
    use super::schema::description_information;
    let insertable_records: Vec<NewDescription> = descriptions.iter().map(|r| r.into()).collect();
    diesel::insert_into(description_information::table)
        .values(&insertable_records)
        .execute(connection)
}
