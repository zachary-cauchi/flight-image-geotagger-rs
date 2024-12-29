mod data_providers;
mod image_geotagger;
mod models;
mod parsers;

use std::{fmt::Display, path::PathBuf, process::exit};

use clap::{Args, Parser, ValueEnum};
use data_providers::{json_provider::FlightDataFileProvider, FlightDataProvider};
use image_geotagger::ImageGeotagger;
use models::result::{GTError, GTResult};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FlightDataSrc {
    Json,
}

impl Display for FlightDataSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Json => "json",
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
    #[arg(short, default_value_t = FlightDataSrc::Json)]
    flight_data_src: FlightDataSrc,

    /// File path to flight geodata json file.
    #[arg(short, long)]
    json_file: Option<PathBuf>,

    /// Path to directory containing all images to geotag.
    images_dir: PathBuf,
}

impl TagArgs {
    pub fn try_get_provider(&self) -> GTResult<impl FlightDataProvider> {
        if let Some(ref path) = self.json_file {
            Ok(FlightDataFileProvider::new(path.clone()))
        } else {
            GTResult::Err(GTError::Args("Invalid configuration.".to_string()))
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

    let mapper = ImageGeotagger::new(flight_data);

    for entry_res in std::fs::read_dir(args.images_dir)? {
        let entry = entry_res?;
        let path = entry.path();

        if !path.exists() || !path.is_file() {
            println!(
                "Entry does not exist or is not file. Skipping {}",
                path.display()
            );
            continue;
        }

        mapper.apply_gps_data(&path)?;
    }

    Ok(())
}
