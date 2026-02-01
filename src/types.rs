use askama::Template;
use serde::Deserialize;
use tokio::process::Child;

pub struct AppState {
    pub process: Option<Child>,
    pub selection: usize,
    pub streams: Vec<(String, String)>,
    pub volume: u8,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub streams: Vec<String>,
    pub selection: usize,
    pub volume: u8,
}

#[derive(Deserialize)]
pub struct VolumeForm {
    pub volume: u8,
}

#[derive(Deserialize)]
pub struct StreamForm {
    pub selection: usize,
}

#[derive(Deserialize)]
pub struct Stream {
    pub name: String,
    pub address: String,
}
