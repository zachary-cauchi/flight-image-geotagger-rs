use std::fmt::Display;

use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug)]
pub struct GeoPosition {
    pub timestamp: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64,
}

impl Display for GeoPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(timestamp: {}, Lat: {}, Lon: {})",
            self.timestamp, self.latitude, self.longitude
        )
    }
}

#[derive(Clone, Debug)]
pub struct FlightGeodata {
    pub flight_code: String,
    pub positions: Vec<GeoPosition>,
}

impl Display for FlightGeodata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Flight \"{}\", Total positions: {}",
            self.flight_code,
            self.positions.len()
        )
    }
}
