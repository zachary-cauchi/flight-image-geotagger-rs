use std::fmt::Display;

use chrono::{DateTime, Utc};

use super::result::{GTError, GTResult};

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
    positions: Vec<GeoPosition>,
}

impl FlightGeodata {
    pub fn new(flight_code: String, mut positions: Vec<GeoPosition>) -> Self {
        positions.sort_by_key(|p| p.timestamp);

        Self {
            flight_code,
            positions,
        }
    }

    fn binary_search_positions(&self, timestamp: DateTime<Utc>) -> GTResult<usize> {
        if timestamp < self.positions[0].timestamp {
            return Err(GTError::MissingData(format!(
                "Image timestamp '{timestamp}' is older than flight position data. Oldest timestamp: {}",
                self.positions[0].timestamp
            )));
        }

        if timestamp > self.positions[self.positions.len() - 1].timestamp {
            return Err(GTError::MissingData(format!(
                "Image timestamp '{timestamp}' is newer than flight position data. Newest timestamp: {}",
                self.positions[self.positions.len() - 1].timestamp
            )));
        }

        match self
            .positions
            .binary_search_by_key(&timestamp, |p| p.timestamp)
        {
            Ok(i) => Ok(i),
            Err(i) => Ok(i),
        }
    }

    pub fn get_position_from_datetime(&self, timestamp: DateTime<Utc>) -> GTResult<GeoPosition> {
        let closest_position = self.binary_search_positions(timestamp)?;

        println!("Closest position index: {closest_position}");

        if self.positions[closest_position].timestamp == timestamp {
            return Ok(self.positions[closest_position].clone());
        }

        let lower_position = &self.positions[closest_position - 1];
        let higher_position = &self.positions[closest_position];

        let linear_factor = (timestamp.timestamp() - lower_position.timestamp.timestamp()) as f64
            / (higher_position.timestamp.timestamp() - lower_position.timestamp.timestamp()) as f64;

        let interpolated_position = GeoPosition {
            timestamp,
            latitude: lower_position.latitude
                + linear_factor * (higher_position.latitude - lower_position.latitude),
            longitude: lower_position.longitude
                + linear_factor * (higher_position.longitude - lower_position.longitude),
        };

        println!(
            "Lower position: {:?}\nInterpolated position: {:?}\nHigher position: {:?}",
            lower_position, interpolated_position, higher_position
        );

        Ok(interpolated_position)
    }
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
