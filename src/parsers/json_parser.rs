use chrono::DateTime;

use crate::models::{
    flight_geodata::{FlightGeodata, GeoPosition},
    result::{GTError, GTResult},
};

pub trait JsonParser {
    fn try_parse_geodata(&self, src: serde_json::Value) -> GTResult<FlightGeodata>;
}

/// Parses flight geodata from a FlightRadar24 JSON payload.
pub struct FlightRadar24JsonParser {}

impl JsonParser for FlightRadar24JsonParser {
    fn try_parse_geodata(&self, src: serde_json::Value) -> GTResult<FlightGeodata> {
        let flight_code = self.get_flight_code(&src).ok_or(GTError::Parser)?;

        let positions = self.get_geopositions(&src).ok_or(GTError::Parser)?;

        Ok(FlightGeodata::new(flight_code, positions))
    }
}

impl FlightRadar24JsonParser {
    fn get_flight_code(&self, src: &serde_json::Value) -> Option<String> {
        let value = src
            .pointer("/result/response/data/flight/identification/number/default")?
            .as_str()?;
        Some(value.to_string())
    }

    fn get_geopositions(&self, src: &serde_json::Value) -> Option<Vec<GeoPosition>> {
        let value = src
            .pointer("/result/response/data/flight/track")?
            .as_array()?;

        value
            .into_iter()
            .map(|v| self.get_geoposition(v))
            .collect::<Option<Vec<GeoPosition>>>()
    }

    fn get_geoposition(&self, src: &serde_json::Value) -> Option<GeoPosition> {
        Some(GeoPosition {
            timestamp: DateTime::from_timestamp(src.get("timestamp")?.as_i64()?, 0)?,
            latitude: src.get("latitude")?.as_f64()?,
            longitude: src.get("longitude")?.as_f64()?,
            altitude: src.pointer("/altitude/meters")?.as_i64()?,
        })
    }
}
