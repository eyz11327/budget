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

pub fn select_records(connection: &mut PgConnection) -> Vec<super::models::Record> {
    use super::schema::records;

    let results = records::table
        .select(Record::as_select())
        .load(connection)
        .expect("Error loading records");
    println!("Directly after records: {:?}", results);
    return results;
}

pub fn insert_records(connection: &mut PgConnection) -> () {}

pub fn select_descriptions(connection: &mut PgConnection) -> Vec<super::models::Description> {
    use super::schema::description_information;

    let results = description_information::table
        .select(Description::as_select())
        .load(connection)
        .expect("Error loading descriptions");
    return results;
}

pub fn insert_description(connection: &mut PgConnection) -> () {}
