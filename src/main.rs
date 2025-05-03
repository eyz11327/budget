use std::{error::Error, fs::File};
use chrono::NaiveDate;

#[derive(Debug)]
struct BudgetRecord {
    amount: f32,
    date: NaiveDate,
    card: String,
    description: String
}

fn read_capital_one_csv(raw_capital_one_file: std::fs::File) -> Result<Vec<BudgetRecord>, Box<dyn Error>> {
    let mut ret: Vec<BudgetRecord> = Vec::new();

    let mut rdr = csv::ReaderBuilder::new().from_reader(raw_capital_one_file);
    for result in rdr.records() {
        let capital_one_record = result?;
        let amount: f32;
        if &capital_one_record[6] != "" {
            if &capital_one_record[3] == "CREDIT-CASH BACK REWARD" {
                amount = capital_one_record[6].parse::<f32>().expect("The Capital One record must include an amount");
            }
            else{
                // We want to ignore payments into the card since they will be covered from USAA budget file. Ignore these lines :)
                continue;
            }
        }
        else {
            amount = -1.00 * capital_one_record[5].parse::<f32>().expect("The Capital One record must include an amount");
        }

        let date = NaiveDate::parse_from_str(&capital_one_record[0], "%Y-%m-%d").expect("The Capital One record must include a date.");
        
        let budget_record = BudgetRecord{amount: amount, date: date, card: String::from("CapitalOne"), description: capital_one_record[3].to_string()};
        ret.push(budget_record);
    }
    println!("Found Capital One budget records: {:?}", ret.len());
    Ok(ret)
}

fn read_usaa_csv(raw_usaa_file: std::fs::File) -> Result<Vec<BudgetRecord>, Box<dyn Error>> {
    let mut ret: Vec<BudgetRecord> = Vec::new();
    // Build the csv reader and iterate over each record
    let mut rdr = csv::ReaderBuilder::new().from_reader(raw_usaa_file);
    for result in rdr.records() {
        let usaa_record=result?;

        let amount = usaa_record[4].parse::<f32>().expect("The USAA record must include an amount");
        let date = NaiveDate::parse_from_str(&usaa_record[0], "%Y-%m-%d").expect("The USAA record must include a date");

        let budget_record = BudgetRecord{amount: amount, date: date, card: String::from("USAA"), description: usaa_record[1].to_string()};
        ret.push(budget_record);
    }
    println!("Found USAA budget records: {:?}", ret.len());
    Ok(ret)
}


fn main() {
    let raw_usaa_file = File::open("./files/usaa_raw.csv").expect("Must have access to a raw usaa file to process.");
    let raw_capital_one_file = File::open("./files/capital_one_raw.csv").expect("Must have access to a raw capital one file to process.");
    let mut budget_records = read_usaa_csv(raw_usaa_file).expect("USAA file must successfully process");
    let mut capital_one_records = read_capital_one_csv(raw_capital_one_file).expect("Capital one file must successfully process");

    budget_records.append(&mut capital_one_records);
    println!("Found total budget records: {:?}", budget_records.len());

    let mut spending_total = 0.00;
    let mut income_total = 0.00;
    for budget_record in budget_records {
        if budget_record.amount < 0.00 {
            spending_total += budget_record.amount;
        }
        else {
            income_total += budget_record.amount;
        }
        println!("Individual record: {:?}", budget_record)
    }
    println!("Income total: {income_total}");
    println!("Spending total: {spending_total}");
}
