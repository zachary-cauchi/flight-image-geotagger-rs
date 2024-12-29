use std::{fs::File, io::BufReader, path::Path};

use chrono::{DateTime, Utc};
use exif::{Exif, In, Tag, Value};

use crate::models::{
    flight_geodata::FlightGeodata,
    result::{GTError, GTResult},
};

pub struct ImageGeotagger {
    flight_data: FlightGeodata,
}

impl ImageGeotagger {
    pub fn new(data: FlightGeodata) -> Self {
        Self { flight_data: data }
    }

    fn get_image_timestamp(&self, exif: &Exif) -> GTResult<DateTime<Utc>> {
        let field = exif
            .get_field(Tag::DateTimeOriginal, In::PRIMARY)
            .ok_or(GTError::MissingData("DateTimeOriginal".to_string()))?;

        let Value::Ascii(ref val) = field.value else {
            return Err(GTError::MissingData(
                "DateTimeOriginal not formatted correctly.".to_string(),
            ));
        };

        let datetime = exif::DateTime::from_ascii(&val[0])?;

        let naive_date = chrono::NaiveDate::from_ymd_opt(
            datetime.year.into(),
            datetime.month.into(),
            datetime.day.into(),
        )
        .ok_or(GTError::InvalidData(
            "Invalid date initialisation".to_string(),
        ))?;
        let naive_time = chrono::NaiveTime::from_hms_opt(
            datetime.hour.into(),
            datetime.minute.into(),
            datetime.second.into(),
        )
        .ok_or(GTError::InvalidData(
            "Invalid time initialisation".to_string(),
        ))?;

        Ok(chrono::NaiveDateTime::new(naive_date, naive_time).and_utc())
    }

    pub fn apply_gps_data(&self, image_path: &Path) -> GTResult<()> {
        let file = File::open(image_path)?;
        let mut reader = BufReader::new(file);
        let exif_reader = exif::Reader::new();
        let exif = exif_reader.read_from_container(&mut reader)?;

        let timestamp = self.get_image_timestamp(&exif)?;

        let image_position = self.flight_data.get_position_from_datetime(timestamp)?;

        Ok(())
    }
}
