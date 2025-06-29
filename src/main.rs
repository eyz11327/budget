use chrono::NaiveDate;
use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    fs,
    fs::File,
    io::{self, ErrorKind, Write},
    path::PathBuf,
    sync::LazyLock,
};
mod database;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use database::db;

#[derive(Debug)]
struct BudgetRecord {
    amount: f64,
    date: NaiveDate,
    card: String,
    description: String,
}
impl fmt::Display for BudgetRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Amount: {0} | Date: {1} | Card: {2} | Description: {3}",
            self.amount, self.date, self.card, self.description
        )
    }
}

#[derive(Debug)]
struct UploadDescription {
    description: String,
    primary_information: String,
    secondary_information: String,
    tertiary_information: String,
    additional_information: String,
}

fn standardize_description(description: &str) -> String {
    let raw_description = description.to_lowercase();
    // Hard coded mapping of purchases that contain a UUID in them that I want to standardize
    static DESCRIPTION_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
        let mut map = HashMap::new();

        // Online Stores
        map.insert("amazon", "amazon");
        map.insert("amzn", "amazon");
        map.insert("prime video", "tv");
        map.insert("amc", "amc");
        map.insert("petsmart", "petsmart");

        // In Person Stores
        map.insert("target", "target");
        map.insert("the home depot", "home depot");
        map.insert("rei", "rei");
        map.insert("barnes & noble", "barnes & noble");
        map.insert("autozone", "autozone");
        map.insert("crate & barrel", "crate & barrel");
        map.insert("vca animal hosp", "vca veterinarian");
        map.insert("laz parking", "laz parking");
        map.insert("spothero", "spothero");
        map.insert("walgreens", "walgreens");
        map.insert("831 bowlero", "bowlero");

        // Airlines & Travel
        map.insert("united", "united airlines");
        map.insert("delta", "delta airlines");
        map.insert("hilton", "hilton");
        map.insert("airbnb", "airbnb");

        // Restaurants
        map.insert("ihop", "ihop");
        map.insert("bonefish", "bonefish");
        map.insert("chick-fil-a", "chick-fil-a");
        map.insert("chipotle", "chipotle");
        map.insert("mad greens", "mad greens");
        map.insert("domino's", "dominos");
        map.insert("dunkin", "dunkin donuts");
        map.insert("panda express", "panda express");
        map.insert("noodles & co", "noodles & co");
        map.insert("olive garden", "olive garden");
        map.insert("oracl*waffle house", "waffle house");
        map.insert("bop & gogi", "bop & gogi");
        map.insert("paypal *domino's", "dominos");

        // Gas
        map.insert("safeway fuel", "safeway fuel");
        map.insert("king soopers fuel", "king soopers fuel");
        map.insert("conoco", "conoco");
        map.insert("phillips 66", "phillips 66");
        map.insert("stop 4 gas", "stop 4 gas");
        map.insert("circle k", "circle k");
        map.insert("shell", "shell");
        map.insert("7-eleven", "7-eleven");
        map.insert("qt", "quicktrip");
        map.insert("chevron", "chevron");
        map.insert("kum&go", "kum&go");

        // Groceries
        map.insert("trader joe s", "trader joe's");
        map.insert("publix", "publix");
        map.insert("safeway #", "safeway");
        map.insert("king soopers #", "king soopers");

        map
    });

    for (prefix, standardized) in DESCRIPTION_MAP.iter() {
        if raw_description.starts_with(prefix) {
            return String::from(*standardized);
        }
    }

    raw_description
}

fn description_input_parser() -> Option<String> {
    let mut input = String::new();

    loop {
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: _,
            state: _,
        }) = event::read().unwrap()
        {
            match (code, modifiers) {
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                    println!("\r");
                    return None;
                }
                (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                    println!("\r");
                    return Some("CRTL+A_ABORT".into());
                }
                (KeyCode::Enter, _) => {
                    println!("\r");
                    return Some(input);
                }
                (KeyCode::Char(c), _) => {
                    print!("{c}");
                    io::stdout().flush().unwrap();
                    input.push(c);
                }
                (KeyCode::Backspace, _) => {
                    if input.pop().is_some() {
                        print!("\x08 \x08");
                        io::stdout().flush().unwrap();
                    }
                }
                _ => {}
            }
        }
    }
}

fn parse_record(record: csv::StringRecord, origin: &str) -> Option<BudgetRecord> {
    match origin.to_lowercase().as_str() {
        "usaa" => {
            if record[1].contains("Capital One"){
                // I know I am up to date on all credit card payments, so we can skip card payments so they don't count towards totals
                return None
            }

            let amount = record[4].parse::<f64>().expect("The USAA record must include an amount");
            let date = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d").expect("The USAA record must include a date");
            let description = standardize_description(&record[1]);

            let budget_record = BudgetRecord{amount, date, card: String::from("USAA"), description};
            Some(budget_record)
        },
        "capitalone" => {
            let amount: f64;
            // If there is a credit amount, check whether it is a cash back or whether it is a payment onto the card
            if !record[6].is_empty() {
                if &record[3] == "CREDIT-CASH BACK REWARD"{
                    amount = record[6].parse::<f64>().expect("The Capital One record must include an amount");
                }
                else{
                    // I know I am up to date on all credit card payments, so we can skip card payments so they don't count towards totals
                    return None
                }
            }
            else {
                // Negative so that we normalize income/spend notation
                amount = -record[5].parse::<f64>().expect("The Capital One record must include an amount");
            }

            let date = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d").expect("The Capital One record must include a date.");
            let description = standardize_description(&record[3]);

            let budget_record = BudgetRecord{amount, date, card: String::from("CapitalOne"), description};
            Some(budget_record)
        },
        _ => panic!("You have entered an unknown origin. Options are 'usaa' or 'capitalone'. Your input: {origin}")
    }
}

fn read_budget_file(path: PathBuf) -> Result<Vec<BudgetRecord>, Box<dyn Error>> {
    let mut ret: Vec<BudgetRecord> = Vec::new();

    let file = File::open(&path)?;
    let mut rdr = csv::ReaderBuilder::new().from_reader(file);

    // Figure out if the budget file is from USAA or Capital One
    // USAA Headers: Date,Description,Original Description,Category,Amount,Status
    // Capital One Headers: Transaction Date,Posted Date,Card No.,Description,Category,Debit,Credit
    let headers = rdr.headers()?;
    let origin: String;
    if &headers[0].to_lowercase() == "date" {
        origin = "usaa".to_string();
    } else if &headers[0].to_lowercase() == "transaction date" {
        origin = "capitalone".to_string();
    } else {
        println!("Unknown header type found. First header: {:?}", &headers[0]);
        // TODO: Convert this into an error of some kind.
        return Ok(ret);
    }

    for csv_record in rdr.records() {
        let raw_record = csv_record?;
        let budget_record = parse_record(raw_record, &origin);
        match budget_record {
            Some(budget_record) => ret.push(budget_record),
            _ => continue,
        }
    }
    println!("Found {} budget records: {}", origin, ret.len());

    Ok(ret)
}

fn setup() -> Result<(PathBuf, PathBuf, serde_json::Value), Box<dyn Error>> {
    // Grab the current working directory
    let cwd = env::current_dir()?;

    // Grab the filepath
    let default_filepath = cwd.join("files/");
    let fp = match env::var("BUDGET_FILE_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_e) => {
            // println!("Error reading env var BUDGET_FILE_PATH. Error: '{e}'. Using default filepath: {:?}", default_filepath);
            default_filepath
        }
    };

    // Grab secret config
    let secret_config: serde_json::Value =
        serde_json::from_reader(File::open(cwd.join("config/secret_config.json"))?)?;

    // Grab the files to be processed

    Ok((cwd, fp, secret_config))
}

fn main() {
    // Grab the setup information and ensure it is valid
    let setup = setup();
    let (cwd, fp, secret_config) = match setup {
        Ok((cwd, fp, secret_config)) => (cwd, fp, secret_config),
        Err(e) => panic!("There was an error during setup. Error: {e}"),
    };

    println!("CWD: {:?} | File Path: {:?}", cwd, fp);

    // Grab any new budget files to process
    let budget_files_to_process = fs::read_dir(&fp.join("new/"));
    let budget_files_to_process = match budget_files_to_process {
        Ok(budget_files_to_process) => {
            let budget_files: Vec<fs::DirEntry> = budget_files_to_process
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_file())
                .collect();
            budget_files
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => panic!(
                "There is no 'new/' directory available at filepath {:?}",
                fp
            ),
            _ => panic!("There was an error determining the files to process. Error: {e}"),
        },
    };

    if budget_files_to_process.is_empty() {
        println!("There are no new budget files to process.");
        std::process::exit(0);
    }

    println!(
        "Found {:?} new budget file(s) to process: {:?}",
        budget_files_to_process.len(),
        budget_files_to_process
    );

    // Process the new budget files
    let mut budget_records: Vec<BudgetRecord> = Vec::new();

    for budget_file in budget_files_to_process {
        let path = budget_file.path();

        let record_information = read_budget_file(path);
        let records = match record_information {
            Ok(records) => records,
            Err(e) => {
                println!(
                    "There was an error reading budget file {:?}. Error: {:?}",
                    budget_file.path(),
                    e
                );
                continue;
            }
        };

        budget_records.extend(records);
    }

    println!("Found total budget records: {}", budget_records.len());

    let connection = &mut db::establish_connection(secret_config);
    // let result = db::insert_records(connection, &budget_records);
    // match result {
    //     Ok(_) => (),
    //     Err(err) => panic!("There was an error {err}"),
    // }

    // Grab the unique standardized descriptions
    let mut unique_descriptions: HashSet<&String> = HashSet::new();
    for budget_record in &budget_records {
        unique_descriptions.insert(&budget_record.description);
    }
    println!("Unique Descriptions: {}", unique_descriptions.len());

    // Remove the descriptions that already have information stored for them
    let descriptions = db::select_descriptions(connection);
    for description in descriptions {
        unique_descriptions.remove(&description.description);
    }

    println!("Unique Descriptions: {}", unique_descriptions.len());
    let mut upload_descriptions: Vec<UploadDescription> = Vec::new();
    // Request information on the descriptions that remain
    println!("Requesting information on descriptions that have not been seen before.\nPress CRTL + S to skip the current description, and CRTL + A to skip all the remaining descriptions.");
    enable_raw_mode().unwrap();
    'outer: for description in unique_descriptions {
        println!("\r");
        println!("Please provide primary information for description '{description}':\r");
        let primary_information = match description_input_parser() {
            Some(s) if s == "CRTL+A_ABORT" => break 'outer,
            Some(s) => s,
            None => continue, // CRTL+S
        };

        println!("\r");
        println!("Please provide secondary information if it exists:\r");
        let secondary_information = match description_input_parser() {
            Some(s) if s == "CRTL+A_ABORT" => break 'outer,
            Some(s) => s,
            None => continue, // CRTL+S
        };

        println!("\r");
        println!("Please provide tertiary information if it exists:\r");
        let tertiary_information = match description_input_parser() {
            Some(s) if s == "CRTL+A_ABORT" => break 'outer,
            Some(s) => s,
            None => continue, // CRTL+S
        };

        println!("\r");
        println!("Please provide additional information if it exists:\r");
        let additional_information = match description_input_parser() {
            Some(s) if s == "CRTL+A_ABORT" => break 'outer,
            Some(s) => s,
            None => continue, // CRTL+S
        };

        let upload_description = UploadDescription {
            description: description.to_string(),
            primary_information,
            secondary_information,
            tertiary_information,
            additional_information,
        };

        println!("\r");
        println!("Description for upload: {:?}\n\n", upload_description);
        upload_descriptions.push(upload_description);
    }
    disable_raw_mode().unwrap();

    println!("{:?}", upload_descriptions);
    // Upload the new description information
    let result = db::insert_description(connection, &upload_descriptions);
    match result {
        Ok(_) => (),
        Err(err) => panic!("There was an error uploading the new description information to the database. Error: {err}")
    }

    // Very very basic analysis
    let mut spending_total: f64 = 0.00;
    let mut income_total: f64 = 0.00;
    for budget_record in budget_records {
        if budget_record.amount < 0.00 {
            spending_total += budget_record.amount;
        } else {
            income_total += budget_record.amount;
        }
        // println!("Individual record: {:?}", budget_record)
    }
    println!("Income total: {income_total:.2}");
    println!("Spending total: {spending_total:.2}");
    let difference = spending_total + income_total;
    println!("Difference: {difference:.2}");
}
