use std::{error::Error, fs::File, process};

fn read_csv() -> Result<(), Box<dyn Error>> {
    // Build the csv reader and iterate over each record
    let mut rdr = csv::ReaderBuilder::new().from_reader(File::open("files/bk_download.csv")?);
    for result in rdr.records() {
        let record=result?;
        println!("{:?}", record);
    }
    Ok(())
}


fn main() {
    if let Err(err) = read_csv() {
        println!("Error running example {err}");
        process::exit(1);
    }
}
