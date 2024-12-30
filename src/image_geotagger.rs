use std::{
    fs::File,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use exif::{Exif, Field, In, Tag, Value};
use img_parts::{jpeg::Jpeg, ImageEXIF};

use crate::models::{
    flight_geodata::FlightGeodata,
    result::{GTError, GTResult},
};

pub struct ImageGeotagger {
    output_dir: PathBuf,
    flight_data: FlightGeodata,
}

impl ImageGeotagger {
    pub fn new(output_dir: PathBuf, data: FlightGeodata) -> Self {
        Self {
            output_dir,
            flight_data: data,
        }
    }

    fn load_image_exif(&self, jpeg: &Jpeg) -> GTResult<Exif> {
        let exif_raw = jpeg.exif().ok_or(GTError::MissingData(
            "No EXIF data found in image".to_string(),
        ))?;
        let exif_reader = exif::Reader::new();
        let exif = exif_reader.read_raw(exif_raw.into())?;

        Ok(exif)
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

    fn build_new_exif<'a>(&self, exif: &'a Exif, new_fields: &'a [Field]) -> Vec<&'a Field> {
        exif.fields()
            .filter(|f| match f.tag {
                Tag::GPSAltitudeRef
                | Tag::GPSAltitude
                | Tag::GPSLatitudeRef
                | Tag::GPSLatitude
                | Tag::GPSLongitudeRef
                | Tag::GPSLongitude => false,
                _ => true,
            })
            .chain(new_fields)
            .collect::<Vec<_>>()
    }

    fn save_new_image(&self, image_path: &Path, jpeg: Jpeg) -> GTResult<()> {
        let output_path = self.output_dir.join(image_path.file_name().unwrap());

        println!("\nSaving image to {}", output_path.display());

        std::fs::create_dir_all(&self.output_dir)?;

        let output_file = File::create(output_path)?;
        let output_writer = std::io::BufWriter::new(output_file);

        jpeg.encoder().write_to(output_writer)?;
        Ok(())
    }

    pub fn apply_gps_data(&self, image_path: &Path) -> GTResult<()> {
        let img_file = std::fs::read(image_path)?;
        let mut jpeg = Jpeg::from_bytes(img_file.into())?;

        let exif = self.load_image_exif(&jpeg)?;

        let timestamp = self.get_image_timestamp(&exif)?;

        println!("Done.\nGetting new GPS metadata.");
        let new_fields = self.flight_data.get_gps_exif_from_datetime(timestamp)?;

        println!("Building new EXIF.");

        let fields_to_write = self.build_new_exif(&exif, &new_fields);
        let mut buffer = std::io::Cursor::new(Vec::new());
        let mut writer = exif::experimental::Writer::new();
        for field in fields_to_write {
            writer.push_field(field);
        }
        writer.write(&mut buffer, false)?;

        println!("Done.");

        jpeg.set_exif(Some(buffer.into_inner().into()));
        self.save_new_image(image_path, jpeg)?;

        Ok(())
    }
}
