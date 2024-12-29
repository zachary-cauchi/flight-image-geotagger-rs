mod models;

use std::path::PathBuf;

use clap::{Args, Parser, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FlightDataSrc {
    Json
}

#[derive(Args)]
#[command(version, about)]
struct TagArgs {
    /// The flight code of the flight on which the images were taken.
    #[arg(long)]
    flight_code: String,

    /// Which source to use for flight geodata.
    #[arg(short)]
    flight_data_src: FlightDataSrc,

    /// File path to flight geodata json file.
    #[arg(short)]
    json_file: Option<PathBuf>,

    /// Path to directory containing all images to geotag.
    images_dir: PathBuf,
}

#[derive(Parser)]
#[command(name = "airmode-tagger")]
#[command(bin_name = "airmode-tagger")]
enum Cli {
    Tag(TagArgs)
}

fn main() {
    let Cli::Tag(tag) = Cli::parse();
    
    println!("Hello, world!");
}
