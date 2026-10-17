use super::Game;
use std::{collections::HashMap, error::Error, env, process, thread, time::{Duration, Instant}};
use eframe::egui::Key::P;
use steamworks::Client;
use serde::Deserialize;

pub fn idle() -> &'static str {
    let appid: u32 = env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| {
            eprintln!("usage: idler <appid> [duration]");
            process::exit(1);
        });

    let duration: Option<Duration> = env::args()
        .nth(2)
        .map(|s| {
            let secs: f64 = s.parse().unwrap_or_else(|_| {
                eprintln!("seconds must be a number");
                process::exit(1);
            });
            if !secs.is_finite() || secs <= 0.0 {
                eprintln!("seconds must be a positive number");
                process::exit(1);
            }
            Duration::from_secs_f64(secs)
        });

    let client = Client::init_app(appid).unwrap_or_else(|e| {
        eprintln!("Steam init failed: {e}. Is Steam running and do you own this app?");
        process::exit(1);
    });

    match duration {
        Some(l) => println!("Idling {appid} for {}s. Ctrl+C to stop early.", l.as_secs()),
        None => println!("Idling {appid} until Ctrl+C."),
    }

    let start = Instant::now();
    loop {
        if duration.map_or(false, |l| start.elapsed() >= l) {
            break;
        }
        client.run_callbacks();
        thread::sleep(Duration::from_secs(1));
    }

    return "Done";
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
