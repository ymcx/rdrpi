use askama::Template;
use axum::{
    Form, Router,
    extract::State,
    response::{Html, Redirect},
    routing,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::{net::TcpListener, process::Child, sync::Mutex};

mod io;

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
    if form.selection != 0 {
        let stream = STREAMS[form.selection - 1].1;
        state.process = io::set_stream(stream);
    }

    Redirect::to("/")
}

async fn set_volume(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(form): Form<VolumeForm>,
) -> Redirect {
    state.lock().await.volume = form.volume;
    io::set_volume(form.volume);

    Redirect::to("/")
}

#[tokio::main]
async fn main() {
    let (ip, port) = io::get_arguments();
    let address = format!("{ip}:{port}");
    let listener = TcpListener::bind(&address).await;

    if listener.is_err() {
        return eprintln!("Couldn't bind to port {port}");
    }

    let state = Arc::new(Mutex::new(AppState {
        process: None,
        selection: 0,
        volume: io::get_volume().await,
    }));
    let app = Router::new()
        .route("/", routing::get(index))
        .route("/set_stream", routing::post(set_stream))
        .route("/set_volume", routing::post(set_volume))
        .with_state(state);

    println!("Running RDRPI @ http://{address}");
    axum::serve(listener.unwrap(), app).await.unwrap();
}
