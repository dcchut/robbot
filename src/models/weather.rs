use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::make_client;
use std::time::Duration;

type RateLimiter = governor::RateLimiter<
    governor::state::NotKeyed,
    governor::state::InMemoryState,
    governor::clock::DefaultClock,
    governor::middleware::NoOpMiddleware,
>;

fn rate_limiter() -> &'static RateLimiter {
    static PS_RATE_LIMITER: once_cell::sync::OnceCell<RateLimiter> =
        once_cell::sync::OnceCell::new();
    PS_RATE_LIMITER.get_or_init(|| {
        governor::RateLimiter::direct(
            governor::Quota::with_period(Duration::from_millis(1500)).unwrap(),
        )
    })
}

#[derive(Clone)]
pub struct NominatimClient {
    limiter: &'static RateLimiter,
    client: reqwest::Client,
}

#[derive(Clone, Debug, Serialize)]
struct NominatimRequest {
    q: String,
    format: &'static str,
    limit: u8,
}

#[derive(Clone, Debug, Deserialize)]
struct NominatimElement {
    lat: String,
    lon: String,
    display_name: String,
}

#[derive(Clone, Debug, Deserialize)]
struct NominatimResponse(Vec<NominatimElement>);

impl NominatimClient {
    pub fn new() -> Self {
        Self {
            limiter: rate_limiter(),
            client: make_client(),
        }
    }

    /// Return the coordinates associated with the given query string.
    pub async fn search<T: Into<String>>(
        &self,
        query: T,
    ) -> Result<Option<(String, String, String)>> {
        let request = NominatimRequest {
            q: query.into(),
            format: "jsonv2",
            limit: 1,
        };

        self.limiter.until_ready().await;
        let response = self
            .client
            .get("https://nominatim.openstreetmap.org/search.php")
            .query(&request)
            .send()
            .await?;

        // Note: by limit=1 above we know that the last element of payload, if it exists,
        // will also be the first.
        let mut payload: NominatimResponse = response.json().await?;
        Ok(payload.0.pop().map(|e| (e.display_name, e.lat, e.lon)))
    }
}

pub struct OpenWeatherMapClient {
    client: reqwest::Client,
    api_key: String,
}

#[derive(Clone, Debug, Serialize)]
struct OpenWeatherMapQuery<'a> {
    lat: &'a str,
    lon: &'a str,
    #[serde(rename = "appid")]
    app_id: &'a str,
    units: &'static str,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OWMMain {
    pub temp: f32,
    pub temp_min: f32,
    pub temp_max: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OWMSys {
    pub country: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OWMWeather {
    pub main: String,
    pub description: String,
    // TODO
    pub icon: String,
}

fn emoji_for_icon(icon: &str) -> &'static str {
    match icon {
        "01d" => ":sunny:",
        "01n" => ":crescent_moon:",
        "02d" => ":white_sun_cloud:",
        "02n" | "03d" | "03n" | "04d" | "04n" => ":cloud:",
        "09d" | "09n" | "10d" | "10n" => ":cloud_rain:",
        "11d" | "11n" => ":cloud_lightning:",
        "13d" | "13n" => ":cloud_snow:",
        "50d" | "50n" => ":fog",
        _ => panic!("unexpected icon {}", icon),
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpenWeatherMapResponse {
    pub main: OWMMain,
    pub sys: OWMSys,
    pub weather: Vec<OWMWeather>,
    pub name: String,
}

impl OpenWeatherMapClient {
    pub fn new<T: Into<String>>(api_key: T) -> Self {
        Self {
            client: make_client(),
            api_key: api_key.into(),
        }
    }

    pub async fn get(
        &self,
        lat: &str,
        lon: &str,
    ) -> Result<Option<(OpenWeatherMapResponse, String)>> {
        let query = OpenWeatherMapQuery {
            lat,
            lon,
            app_id: &self.api_key,
            units: "metric",
        };

        let response = self
            .client
            .get("https://api.openweathermap.org/data/2.5/weather")
            .query(&query)
            .send()
            .await?;

        let payload: OpenWeatherMapResponse = response.json().await?;
        let emoji = emoji_for_icon(&payload.weather[0].icon);

        Ok(Some((payload, emoji.to_string())))
    }
}
