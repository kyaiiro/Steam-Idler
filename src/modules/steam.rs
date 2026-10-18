use super::Game;
use std::{collections::HashMap, process, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread, time::{Duration, Instant}};
use steamworks::Client;
use serde::Deserialize;

pub fn idle(appid: u32, duration: u32, stop: Arc<AtomicBool>) {
    let client = Client::init_app(appid).unwrap_or_else(|e| {
        eprintln!("Steam init failed: {e}. Is Steam running and do you own this app?");
        process::exit(1);
    });

    if duration == 0 {
        println!("Idling {appid} until Ctrl+C.");
    } else {
        println!("Idling {appid} for {duration}s. Ctrl+C to stop early");
    }

    let start = Instant::now();
    let max_duration = Duration::from_secs(duration as u64);

    loop {
        if stop.load(Ordering::SeqCst) {
            println!("Stopping idle for {appid}");
            break;
        }

        if duration != 0 && start.elapsed() >= max_duration {
            break;
        }

        client.run_callbacks();
        thread::sleep(Duration::from_secs(1));
    }

    println!("Done");
    drop(client); //TODO Make the drop logic actually drop the process
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
