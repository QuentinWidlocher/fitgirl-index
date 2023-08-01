use chrono::NaiveDateTime;
use rusqlite::Connection;
use serde::Deserialize;

pub mod genres;
pub mod last_releases;
pub mod release;
pub mod search;

#[cfg(debug_assertions)]
const DB_PATH: &str = "db/fitgirl.db";

#[cfg(not(debug_assertions))]
const DB_PATH: &str = "opt/shuttle/shuttle-builds/fitgirl-index/db/fitgirl.db";

fn get_connection() -> Connection {
    Connection::open(DB_PATH).unwrap()
}

#[derive(Deserialize)]
pub struct Link {
    pub name: String,
    pub link: String,
}

#[derive(Deserialize)]
pub struct Mirror {
    pub links: Vec<Link>,
}

pub struct Release {
    pub title: String,
    pub link: String,
    pub published: NaiveDateTime,
    pub cover_src: String,
    pub original_size: String,
    pub repack_size: String,
    pub mirrors: Vec<Mirror>,
    pub screenshots: Vec<String>,
    pub repack_description: String,
    pub game_description: String,
}
