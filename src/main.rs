mod models;
mod parsers;

use std::{fmt::Display, path::PathBuf, process::exit};

use clap::{Args, Parser, ValueEnum};
use models::result::GTResult;
use parsers::json_parser::{FlightRadar24JsonParser, JsonParser};

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
    if let Some(json_path) = args.json_file {
        println!("Loading geodata from JSON file...");

        let reader = std::io::BufReader::new(std::fs::File::open(json_path)?);
        let json: serde_json::Value = serde_json::from_reader(reader)?;
        let parser = FlightRadar24JsonParser {};
        let geopositions = parser.try_parse_geodata(json)?;

        println!("{geopositions:#?}")
    }

    Ok(())
}
