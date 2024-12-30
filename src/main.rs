mod data_providers;
mod image_geotagger;
mod models;
mod parsers;

use std::{fmt::Display, path::PathBuf, process::exit};

use chrono::{DateTime, Utc};
use clap::{Args, Parser, ValueEnum};
use data_providers::{
    flightradar24_provider::FlightRadar24ApiProvider, json_provider::FlightDataFileProvider,
    FlightDataProvider,
};
use image_geotagger::ImageGeotagger;
use models::result::{GTError, GTResult};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FlightDataSrc {
    Json,
    Api,
}

impl Display for FlightDataSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Json => "json",
            Self::Api => "api",
        };
        f.write_str(text)
    }
}

#[derive(Args)]
#[command(version, about)]
struct TagArgs {
    /// The flight code of the flight on which the images were taken.
    #[arg(long)]
    flight_code: String,

    /// Which source to use for flight geodata.
    #[arg(short, long, name = "src", default_value_t = FlightDataSrc::Json)]
    flight_data_src: FlightDataSrc,

    /// Date of flight departure.
    #[arg(short, long, name = "dod", default_value_t = Utc::now())]
    date_of_departure: DateTime<Utc>,

    /// File path to flight geodata json file.
    #[arg(short, long)]
    json_file: Option<PathBuf>,

    /// Path to directory containing all images to geotag.
    images_dir: PathBuf,
}

impl TagArgs {
    pub fn try_get_provider(&self) -> GTResult<Box<dyn FlightDataProvider>> {
        match self.flight_data_src {
            FlightDataSrc::Json => {
                if let Some(ref path) = self.json_file {
                    Ok(Box::new(FlightDataFileProvider::new(path.clone())))
                } else {
                    GTResult::Err(GTError::Args("Invalid configuration.".to_string()))
                }
            }
            FlightDataSrc::Api => Ok(Box::new(FlightRadar24ApiProvider::new(
                self.flight_code.clone(),
                self.date_of_departure,
            ))),
        }
    }
}

#[derive(Parser)]
#[command(name = "airmode-tagger")]
#[command(bin_name = "airmode-tagger")]
enum Cli {
    Tag(TagArgs),
}

fn main() {
    let Cli::Tag(tag) = Cli::parse();

    println!("Geotagger started!");

    if let Err(e) = run(tag) {
        println!("Fatal error running geotagger. Exiting. Error: {e}");

        exit(1)
    }
}

fn run(args: TagArgs) -> GTResult<()> {
    let provider = args.try_get_provider()?;
    let flight_data = provider.load_data()?;

    println!("Obtained data: {flight_data}");

    let output_dir = args.images_dir.join("geotagged");
    let mapper = ImageGeotagger::new(output_dir, flight_data);

    for entry_res in std::fs::read_dir(args.images_dir)? {
        let entry = entry_res?;
        let path = entry.path();

        if !path.exists() {
            println!(
                "Entry does not exist or is not file. Skipping {}",
                path.display()
            );
            continue;
        }

        if path.is_dir() {
            continue;
        }

        if let Err(e) = mapper.apply_gps_data(&path) {
            println!(
                "Error processing image {}: {e}",
                path.file_name().unwrap().to_str().unwrap()
            );
        }
    }

    println!("All done.");

    Ok(())
}
