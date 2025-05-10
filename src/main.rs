use chrono::NaiveDate;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    sync::LazyLock,
};

#[derive(Debug)]
struct BudgetRecord {
    amount: f64,
    date: NaiveDate,
    card: String,
    description: String,
}

fn standardize_description(description: &str) -> String {
    let raw_description = description.to_lowercase();
    // Hard coded mapping of purchases that contain a UUID in them that I want to standardize
    // TODO: Move to the database for persistence & easier long term management
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

fn read_budget_file(
    file: std::fs::File,
    origin: &str,
) -> Result<Vec<BudgetRecord>, Box<dyn Error>> {
    let mut ret: Vec<BudgetRecord> = Vec::new();
    let mut rdr = csv::ReaderBuilder::new().from_reader(file);
    for csv_record in rdr.records() {
        let raw_record = csv_record?;
        let budget_record = parse_record(raw_record, origin);
        match budget_record {
            Some(budget_record) => ret.push(budget_record),
            _ => continue,
        }
    }
    println!("Found {} budget records: {}", origin, ret.len());

    Ok(ret)
}

fn main() {
    let raw_usaa_file = File::open("./files/usaa_raw.csv").expect("Hard coded file exists :)");
    let raw_capital_one_file =
        File::open("./files/capital_one_raw.csv").expect("Hard coded file exists :)");

    let mut budget_records: Vec<BudgetRecord> = Vec::new();

    // TODO: For loop over all files in dir, use file name or header to determine origin type
    let record_information = read_budget_file(raw_usaa_file, "usaa");
    let mut usaa_records = match record_information {
        Ok(usaa_records) => usaa_records,
        Err(error) => panic!("There was an error while parsing the USAA csv. Error: {error}"),
    };

    let record_information = read_budget_file(raw_capital_one_file, "capitalone");
    let mut capital_one_records = match record_information {
        Ok(capital_one_records) => capital_one_records,
        Err(error) => {
            panic!("There was an error while parsing the Capital One csv. Error: {error}")
        }
    };

    budget_records.append(&mut usaa_records);
    budget_records.append(&mut capital_one_records);
    println!("Found total budget records: {}", budget_records.len());

    // Grab the unique standardized descriptions and request additional information for anything that is not stored in the DB already
    let mut unique_descriptions: HashSet<&String> = HashSet::new();
    for budget_record in &budget_records {
        unique_descriptions.insert(&budget_record.description);
    }
    println!("Unique Descriptions: {}", unique_descriptions.len());

    // TODO: Parse out unique descriptions which have already been given additional information

    // Request additional information for the remaining descriptions
    for name in &unique_descriptions {
        println!("- {}", name)
    }

    // TODO: Upload normalized record information to self-hosted postgres instance

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
