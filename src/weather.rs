use serde::Deserialize;
use reqwest::Error;


#[derive(Deserialize, Debug)]
struct GeoLocation {
    location: Location,
    accuracy: f32
}

#[derive(Deserialize, Debug)]
struct Location {
    lat: f32,
    log: f32
}

impl GeoLocation {
    async fn new() -> Result<GeoLocation, Error> {
        reqwest::get("https://location.services.mozilla.com/v1/geolocate?key=geoclue").await?
            .json::<GeoLocation>().await
    }
}

async fn fetch_current_weather() {
    let gloc = match GeoLocation::new().await {
        Ok(gloc) => gloc,
        Err(error) => panic!("{:?}", error)
    };

}
