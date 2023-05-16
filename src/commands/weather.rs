use crate::{Context, Error};
use futures::{Stream, StreamExt};
use serde::Deserialize;

const BASE_URI: &str = "http://api.openweathermap.org";

#[derive(Deserialize, Debug)]
pub struct Coord {
    pub lon: f64,
    pub lat: f64,
}

#[derive(Deserialize, Debug)]
pub struct Weather {
    pub id: u64,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Deserialize, Debug)]
pub struct Main {
    pub temp: f64,
    pub feels_like: f64,
    pub pressure: f64,
    pub humidity: f64,
    pub temp_min: f64,
    pub temp_max: f64,
    pub sea_level: Option<f64>,
    pub grnd_level: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Wind {
    pub speed: f64,
    pub deg: f64,
    pub gust: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Clouds {
    pub all: f64,
}

#[derive(Deserialize, Debug)]
pub struct Volume {
    #[serde(rename = "1h")]
    pub h1: Option<f64>,
    #[serde(rename = "3h")]
    pub h3: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct Sys {
    #[serde(rename = "type")]
    pub type_: Option<u64>,
    pub id: Option<u64>,
    pub message: Option<f64>,
    pub country: String,
    pub sunrise: i64,
    pub sunset: i64,
}

#[derive(Deserialize, Debug)]
pub struct CurrentWeather {
    pub coord: Coord,
    pub weather: Vec<Weather>,
    pub base: String,
    pub main: Main,
    pub visibility: u64,
    pub wind: Wind,
    pub clouds: Clouds,
    pub rain: Option<Volume>,
    pub snow: Option<Volume>,
    pub dt: i64,
    pub sys: Sys,
    pub timezone: i64,
    pub id: u64,
    pub name: String,
    pub cod: u64,
}

async fn autocomplete_city<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(&_ctx.data().cities)
        .filter(move |city| {
            let query = partial.to_lowercase();
            let name_contains = city.name.to_lowercase().contains(&query);
            let country_contains = city.country.to_lowercase().contains(&query);
            futures::future::ready(name_contains | country_contains)
        })
        .map(|city| city.name.to_owned())
}

/// Gets the weather
#[poise::command(slash_command)]
pub async fn weather(
    ctx: Context<'_>,
    #[description = "The city to get weather for"]
    #[rename = "city"]
    #[autocomplete = "autocomplete_city"]
    query: String,
) -> Result<(), Error> {
    let city = ctx.data().cities.iter().find(|city| city.name == query);
    let key = std::env::var("OPENWEATHER_KEY")?;

    if let Some(city) = city {
        let weather = reqwest::get(format!(
            "{}/data/2.5/weather?lat={}&lon={}&appid={}&units=metric",
            BASE_URI, city.lat, city.lon, key
        ))
        .await?
        .json::<CurrentWeather>()
        .await?;

        ctx.send(|m| {
            m.embed(|e| {
                e.title("Weather")
                    .thumbnail(format!(
                        "https://openweathermap.org/img/wn/{}@2x.png",
                        weather.weather[0].icon
                    ))
                    .colour(0xe10054)
                    .fields([
                        (
                            "Location",
                            format!("{}, {}", city.name, city.country),
                            false,
                        ),
                        ("Temperature", format!("{}°C", weather.main.temp), false),
                        (
                            "Feels like",
                            format!("{}°C", weather.main.feels_like),
                            false,
                        ),
                        ("Humidity", format!("{}%", weather.main.humidity), false),
                        ("Wind", format!("{} km/h", weather.wind.speed), false),
                        (
                            "Condition",
                            weather.weather[0].description.to_owned(),
                            false,
                        ),
                        ("Pressure", format!("{} hPa", weather.main.pressure), false),
                    ])
            })
        })
        .await?;
    } else {
        ctx.say("City not found.").await?;
    }

    Ok(())
}
