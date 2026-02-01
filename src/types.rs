use askama::Template;
use serde::{Deserialize, Serialize};
use tokio::process::Child;

pub struct AppState {
    pub process: Option<Child>,
    pub selection: usize,
    pub streams: Vec<(String, String)>,
    pub volume: u8,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index {
    pub streams: Vec<String>,
    pub selection: usize,
    pub volume: u8,
}

#[derive(Deserialize)]
pub struct SetVolume {
    pub volume: u8,
}

#[derive(Deserialize)]
pub struct SetStream {
    pub selection: usize,
}

#[derive(Deserialize, Serialize)]
pub struct Stream {
    pub name: String,
    pub address: String,
}
