use std::{fmt::Display, path::PathBuf};

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use clap::{command, Args, Parser, ValueEnum};

use crate::{
    data_providers::{
        flightradar24_provider::FlightRadar24ApiProvider, json_provider::FlightDataFileProvider,
        FlightDataProvider,
    },
    models::result::{GTError, GTResult},
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum FlightDataSrc {
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

fn parse_dod_date(s: &str) -> Result<DateTime<Utc>, String> {
    match NaiveDate::parse_from_str(s, "%d %b %Y") {
        Ok(date) => {
            let dod = NaiveDateTime::new(
                date,
                NaiveTime::from_num_seconds_from_midnight_opt(0, 0)
                    .expect("Midnight value should have succeeded."),
            );

            Ok(dod.and_utc())
        }
        Err(e) => Err(format!("Invalid dod value provided ('{s}'). Error: {e}")),
    }
}

#[derive(Args)]
#[command(version, about)]
pub struct TagArgs {
    /// The flight code of the flight on which the images were taken.
    #[arg(long)]
    pub flight_code: String,

    /// Which source to use for flight geodata.
    #[arg(short, long, name = "src", default_value_t = FlightDataSrc::Json)]
    pub flight_data_src: FlightDataSrc,

    /// Date of flight departure.
    #[arg(short, long, name = "dod", value_parser = parse_dod_date, default_value_t = Utc::now())]
    pub date_of_departure: DateTime<Utc>,

    /// File path to flight geodata json file.
    #[arg(short, long)]
    pub json_file: Option<PathBuf>,

    /// Path to directory containing all images to geotag.
    pub images_dir: PathBuf,
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
pub enum Cli {
    Tag(TagArgs),
}
