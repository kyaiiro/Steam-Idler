use super::Game;
use serde::Deserialize;
use std::{
    collections::HashMap,
    thread,
    time::{Duration, Instant},
};
use steamworks::Client;

pub fn idle(appid: u32, duration: Option<Duration>) -> Result<(), String> {
    let client = Client::init_app(appid).map_err(|e| {
        format!("Steam init failed: {e}. Is Steam running and do you own this app?")
    })?;

    println!("Idling: {appid}");
    let deadline = duration.map(|d| Instant::now() + d);

    while deadline.map_or(true, |d| Instant::now() < d) {
        client.run_callbacks();
        thread::sleep(Duration::from_millis(250));
    }

    println!("Done");
    Ok(())
}

#[derive(Deserialize)]
struct AppDetailsResponse {
    success: bool,
    data: Option<AppDetails>,
}

#[derive(Deserialize)]
struct AppDetails {
    name: String,
    header_image: String,
}

pub fn get_game_info(app_id: u32) -> Result<Game, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("https://store.steampowered.com/api/appdetails?appids={app_id}&l=en");

    let response: HashMap<String, AppDetailsResponse> = reqwest::blocking::get(url)?
        .error_for_status()?
        .json()?;

    let details = response
        .get(&app_id.to_string())
        .filter(|result| result.success)
        .and_then(|result| result.data.as_ref())
        .ok_or("Steam returned no details for that app ID")?;

    Ok(Game {
        id: app_id.to_string(),
        title: details.name.clone(),
        cover: details.header_image.clone(),
    })
}