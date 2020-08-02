use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs::File;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct Record {
    name: String,
    age: u64,
    prefecture: String,
}

fn main() {
    let start = Instant::now();

    let record_count = parse_csv();
    let end = start.elapsed();
    println!("{:?}", record_count);
    println!(
        "{}.{:03}秒経過しました。",
        end.as_secs(),
        end.subsec_nanos() / 1_000_000
    );
}

fn parse_csv() -> Result<usize, Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("Can not find arg");
    let file = File::open(path).expect("Can not open file");
    let mut rdr = csv::Reader::from_reader(file);
    let mut records: Vec<Record> = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result.unwrap();
        println!("{:?}", record);
        records.push(record);
    }
    Ok(records.len())
}
