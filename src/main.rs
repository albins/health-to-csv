use csv;
use env_logger;
use log::{debug, error, info, warn};
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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    data_type: String,
    unit: Option<String>,
    value: Option<String>,
    source_name: String,
    source_version: Option<String>,
    device: Option<String>,
    creation_date: Option<String>,
    start_date: String,
    end_date: String,
}

impl Record {
    fn from_dict(record: BTreeMap<String, String>) -> Record {
        Record {
            data_type: record.get("type").unwrap().to_string(),
            unit: record.get("unit").map(|v| v.to_string()),
            value: record.get("value").map(|v| v.to_string()),
            source_name: record.get("sourceName").unwrap().to_string(),
            source_version: record.get("sourceVersion").map(|v| v.to_string()),
            device: record.get("device").map(|v| v.to_string()),
            creation_date: record.get("creationDate").map(|v| v.to_string()),
            start_date: record.get("startDate").unwrap().to_string(),
            end_date: record.get("endDate").unwrap().to_string(),
        }
    }
}

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

fn xml_to_dict(record: roxmltree::Node) -> BTreeMap<String, String> {
    record
        .attributes()
        .iter()
        .map(|a| (a.name().to_string(), a.value().to_string()))
        .collect()
}

fn parse_health_xml(xml: String) -> Result<Vec<Record>, Box<dyn Error>> {
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
        .map(xml_to_dict)
        .map(Record::from_dict)
        .collect())
}

fn dump_csv(records: Vec<Record>) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for record in records.into_iter() {
        if let Err(e) = wtr.serialize(&record) {
            warn!("Error {} writing record {:?}; skipping!", e, record);
        }
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
