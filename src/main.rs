use csv;
use env_logger;
use log::{debug, error, info};
use roxmltree;
use std::collections::BTreeMap; // Sorted by keys!
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Read;
use std::process;
use std::time::Instant;
use zip;

fn load_file(path: &str) -> Result<String, Box<dyn Error>> {
    let read_timer = Instant::now();
    let file = fs::File::open(&path)?;

    let mut archive = zip::ZipArchive::new(file)?;

    let mut xml_file = archive.by_name("apple_health_export/export.xml")?;
    debug!("Found {} MB of data", xml_file.size() / 1024 / 1024);

    let mut text = String::new();
    info!("Reading {} from zip archive {}", xml_file.name(), path);
    xml_file.read_to_string(&mut text)?;
    info!("Read {} in {:?}", path, read_timer.elapsed());
    Ok(text)
}

fn record_to_map(record: roxmltree::Node) -> BTreeMap<String, String> {
    record
        .attributes()
        .iter()
        .map(|a| (a.name().to_string(), a.value().to_string()))
        .collect()
}

fn parse_health_xml(xml: String) -> Result<Vec<BTreeMap<String, String>>, Box<dyn Error>> {
    info!("Parsing XML...");
    let parse_timer = Instant::now();
    let document = roxmltree::Document::parse(&xml)?;
    info!("Parsed XML in {:?}", parse_timer.elapsed());

    let health_data = document
        .root()
        .children()
        .find(|e| e.has_tag_name("HealthData"))
        .ok_or("No HealthData element!")?;

    Ok(health_data
        .children()
        .filter(|e| e.has_tag_name("Record"))
        .map(record_to_map)
        .collect())
}

fn dump_csv(records: Vec<BTreeMap<String, String>>) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    wtr.write_record(records.first().ok_or("No records!")?.keys())?;
    for row in records.iter() {
        wtr.write_record(row.values())?;
    }
    wtr.flush()?;

    info!("Done writing records!");

    Ok(())
}

fn main() {
    let timer = Instant::now();
    env_logger::init();
    let args: Vec<_> = env::args().collect();
    let filename = args
        .get(1)
        .ok_or("Usage: cargo run -- export.zip")
        .unwrap_or_else(|e| {
            error!("Error {}", e);
            process::exit(1)
        });
    let raw_data = load_file(filename).unwrap_or_else(|e| {
        error!("Error {}", e);
        process::exit(1)
    });
    let data = parse_health_xml(raw_data).unwrap_or_else(|e| {
        error!("Error {}", e);
        process::exit(1)
    });
    info!("Read {} records", data.len());
    dump_csv(data).unwrap_or_else(|e| {
        error!("Error {}", e);
        process::exit(1)
    });

    info!("Done processing in {:?}", timer.elapsed());
}
