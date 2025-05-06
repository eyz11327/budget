use chrono::NaiveDate;
use std::{error::Error, fs::File};

#[derive(Debug)]
struct BudgetRecord {
    amount: f64,
    date: NaiveDate,
    card: String,
    description: String,
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

            let budget_record = BudgetRecord{amount: amount, date: date, card: String::from("USAA"), description: record[1].to_string()};
            Some(budget_record)
        },
        "capitalone" => {
            let amount: f64;
            // If there is a credit amount, check whether it is a cash back or whether it is a payment onto the card
            if &record[6] != "" {
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

            let budget_record = BudgetRecord{amount: amount, date: date, card: String::from("CapitalOne"), description: record[3].to_string()};
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
    println!("Found total budget records: {:?}", budget_records.len());

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
        println!("Individual record: {:?}", budget_record)
    }
    println!("Income total: {income_total:.2}");
    println!("Spending total: {spending_total:.2}");
    let difference = spending_total + income_total;
    println!("Difference: {difference:.2}");
}
