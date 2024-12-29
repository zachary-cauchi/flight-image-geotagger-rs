use crate::models::{flight_geodata::FlightGeodata, result::GTResult};

pub mod json_provider;

pub trait FlightDataProvider {
    fn load_data(&self) -> GTResult<FlightGeodata>;
}
