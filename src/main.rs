mod cli;
mod data_providers;
mod image_geotagger;
mod models;
mod parsers;

use std::process::exit;

use clap::Parser;
use cli::{Cli, TagArgs};
use image_geotagger::ImageGeotagger;
use models::result::GTResult;

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
