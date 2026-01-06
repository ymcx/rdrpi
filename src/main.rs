use askama::Template;
use axum::{
    Form, Router,
    extract::State,
    response::{Html, Redirect},
    routing,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    process::{Child, Command},
    sync::Mutex,
};

const STREAMS: &[(&str, &str)] = &[
    (
        "Yle Radio Suomi",
        "https://yleradiolive.akamaized.net/hls/live/2027675/in-YleRS/256/variant.m3u8",
    ),
    (
        "YleX",
        "https://yleradiolive.akamaized.net/hls/live/2027674/in-YleX/256/variant.m3u8",
    ),
];

struct AppState {
    process: Option<Child>,
    selection: usize,
    volume: u8,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    streams: Vec<String>,
    selection: usize,
    volume: u8,
}

#[derive(Deserialize)]
struct VolumeForm {
    volume: u8,
}

#[derive(Deserialize)]
struct StreamForm {
    selection: usize,
}

async fn index(State(state): State<Arc<Mutex<AppState>>>) -> Html<String> {
    let state = state.lock().await;
    let template = IndexTemplate {
        streams: STREAMS.iter().map(|(i, _)| i.to_string()).collect(),
        selection: state.selection,
        volume: state.volume,
    };

    Html::from(template.render().unwrap())
}

async fn set_stream(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(form): Form<StreamForm>,
) -> Redirect {
    let mut state = state.lock().await;
    if let Some(process) = &mut state.process {
        process.kill().await.unwrap();
    }

    state.selection = form.selection;
    if form.selection == 0 {
        return Redirect::to("/");
    }

    let stream = STREAMS[form.selection - 1].1;
    state.process = Command::new("ffmpeg")
        .args(["-vn", "-f", "pulse", "default", "-v", "quiet", "-i", stream])
        .spawn()
        .ok();

    Redirect::to("/")
}

async fn set_volume(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(form): Form<VolumeForm>,
) -> Redirect {
    state.lock().await.volume = form.volume;

    let volume = (form.volume as f32 / 100.0).to_string();
    Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_SINK@", &volume])
        .spawn()
        .unwrap();

    Redirect::to("/")
}

async fn get_volume() -> u8 {
    let process = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_SINK@"])
        .output()
        .await
        .unwrap();
    let stdout = String::from_utf8_lossy(&process.stdout);
    let volume = stdout[8..12].replace('.', "");

    volume.parse().unwrap()
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(AppState {
        process: None,
        selection: 0,
        volume: get_volume().await,
    }));
    let app = Router::new()
        .route("/", routing::get(index))
        .route("/set_stream", routing::post(set_stream))
        .route("/set_volume", routing::post(set_volume))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
