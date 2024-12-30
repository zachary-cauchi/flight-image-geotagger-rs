use chrono::{DateTime, NaiveDateTime, NaiveTime, Utc};
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use crate::{
    models::{
        flight_geodata::FlightGeodata,
        result::{GTError, GTResult},
    },
    parsers::json_parser::{FlightRadar24JsonParser, JsonParser},
};

use super::FlightDataProvider;

pub struct FlightRadar24ApiProvider {
    flight_code: String,
    dod: DateTime<Utc>,
}

impl FlightRadar24ApiProvider {
    const WEBSITE_URL: &'static str = "https://www.flightradar24.com";
    const API_URL: &'static str = "https://api.flightradar24.com/common/v1";

    const FLIGHTS_TABLE_SELECTOR: &'static str = "#tbl-datatable tbody";

    pub fn new(flight_code: String, dod: DateTime<Utc>) -> Self {
        let dod = NaiveDateTime::new(
            dod.date_naive(),
            NaiveTime::from_num_seconds_from_midnight_opt(0, 0)
                .expect("Midnight value should have succeeded."),
        )
        .and_utc();

        Self { flight_code, dod }
    }

    fn get_data_link(&self, client: &Client) -> GTResult<String> {
        let flights_response = client
            .get(format!(
                "{}/data/flights/{}",
                Self::WEBSITE_URL,
                self.flight_code
            ))
            .send()?
            .error_for_status()?;
        let flights_text = flights_response.text()?;
        let dom = Html::parse_document(&flights_text);

        let table_selector = Selector::parse(Self::FLIGHTS_TABLE_SELECTOR)?;
        let Some(table) = dom.select(&table_selector).next() else {
            return Err(GTError::HtmlSelection(format!(
                "Could not find flight history table ({})",
                Self::FLIGHTS_TABLE_SELECTOR
            )));
        };

        let format = self.dod.format("%d %b %Y").to_string();
        println!("Checking for row with date of departure '{format}'");
        let Some(row) = table
            .child_elements()
            .filter(|tr| tr.text().any(|t| t.contains(&format)))
            .next()
        else {
            return Err(GTError::HtmlSelection(format!(
                "Could not find flight record with DoD '{format}'."
            )));
        };

        let playback_btn_selector = Selector::parse(".btn-playback")?;
        let Some(playback_btn) = row.select(&playback_btn_selector).next() else {
            return Err(GTError::HtmlSelection(format!(
                "Could not find playback button for flight row.",
            )));
        };

        let Some(hex) = playback_btn.attr("data-flight-hex") else {
            return Err(GTError::HtmlSelection(
                "No data-flight-hex found in expected location.".to_string(),
            ));
        };

        let Some(data_timestamp) = playback_btn.attr("data-timestamp") else {
            return Err(GTError::HtmlSelection(
                "No data-timestamp found in expected location.".to_string(),
            ));
        };

        let url = format!(
            "{}/flight-playback.json?flightId={}&timestamp={}",
            Self::API_URL,
            hex,
            data_timestamp
        );

        return Ok(url);
    }

    fn download_flight_data(
        &self,
        client: &Client,
        flight_data_url: String,
    ) -> GTResult<FlightGeodata> {
        let response = client.get(flight_data_url).send()?.error_for_status()?;
        let parsed_json: serde_json::Value = serde_json::from_str(response.text()?.as_str())?;

        let parser = FlightRadar24JsonParser {};
        let flight_data = parser.try_parse_geodata(parsed_json)?;

        Ok(flight_data)
    }
}

impl FlightDataProvider for FlightRadar24ApiProvider {
    fn load_data(&self) -> GTResult<FlightGeodata> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.append(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_10_1)".parse()?,
        );

        let client = reqwest::blocking::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        println!("Getting flight Id.");

        let flight_data_url = self.get_data_link(&client)?;

        println!("Obtained url. Downloading data from '{flight_data_url}'.");

        let data = self.download_flight_data(&client, flight_data_url)?;

        Ok(data)
    }
}
