use std::{fs::File, io::BufReader, path::PathBuf};

use serde_json::Value;

use crate::parsers::json_parser::{FlightRadar24JsonParser, JsonParser};

use super::GeodataProvider;

pub struct GeodataFileProvider {
    src_path: PathBuf,
}

impl GeodataProvider for GeodataFileProvider {
    fn load_data(
        &self,
    ) -> crate::models::result::GTResult<crate::models::flight_geodata::FlightGeodata> {
        let reader = BufReader::new(File::open(&self.src_path)?);
        let json: Value = serde_json::from_reader(reader)?;
        let parser = FlightRadar24JsonParser {};
        let geodata = parser.try_parse_geodata(json)?;

        Ok(geodata)
    }
}

impl GeodataFileProvider {
    pub fn new(src_path: PathBuf) -> Self {
        Self { src_path }
    }
}
