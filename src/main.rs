use crate::types::{AppState, Index, SetStream, SetVolume, Stream};
use askama::Template;
use axum::{
    Form, Router,
    extract::State,
    response::{Html, Redirect},
    routing,
};
use std::{error::Error, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};

mod io;
mod types;

async fn index(State(state): State<Arc<Mutex<AppState>>>) -> Html<String> {
    let state = state.lock().await;
    let template = Index {
        streams: state.streams.iter().map(|(i, _)| i.to_string()).collect(),
        selection: state.selection,
        volume: state.volume,
    };

    Html::from(template.render().unwrap())
}

async fn add_stream(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(form): Form<Stream>,
) -> Redirect {
    let mut state = state.lock().await;
    let mut streams = state.streams.clone();
    streams.push((form.name, form.address));

    state.streams = streams;
    io::write_streams(&state.stream_file, &state.streams).unwrap();

    state.selection = state.streams.len();

    Redirect::to("/")
}

async fn delete_stream(State(state): State<Arc<Mutex<AppState>>>) -> Redirect {
    let mut state = state.lock().await;
    if state.selection != 0 {
        let mut streams = state.streams.clone();
        streams.remove(state.selection - 1);

        state.streams = streams;
        io::write_streams(&state.stream_file, &state.streams).unwrap();

        let should_decrease = state.streams.len() == 0 || state.selection != 1;
        state.selection -= if should_decrease { 1 } else { 0 };
    }

    Redirect::to("/")
}

async fn set_stream(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(form): Form<SetStream>,
) -> Redirect {
    let mut state = state.lock().await;
    if let Some(process) = &mut state.process {
        process.kill().await.unwrap();
    }

    state.selection = form.selection;
    if form.selection != 0 {
        let stream = state.streams.get(form.selection - 1).unwrap().1.to_string();
        let process = io::set_stream(&stream).unwrap();
        state.process = Some(process);
    }

    Redirect::to("/")
}

async fn set_volume(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(form): Form<SetVolume>,
) -> Redirect {
    state.lock().await.volume = form.volume;
    io::set_volume(form.volume).unwrap();

    Redirect::to("/")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    io::program_exists("ffmpeg")?;
    io::program_exists("wpctl")?;

    let (ip, port, stream_file) = io::get_arguments();
    let address = format!("{ip}:{port}");
    let listener = TcpListener::bind(&address).await;
    if listener.is_err() {
        return Err(format!("Couldn't bind to port {port}").into());
    }

    let state = Arc::new(Mutex::new(AppState {
        process: None,
        selection: 0,
        stream_file: stream_file.to_string(),
        streams: io::read_streams(&stream_file)?,
        volume: io::get_volume().await?,
    }));
    let app = Router::new()
        .route("/", routing::get(index))
        .route("/add_stream", routing::post(add_stream))
        .route("/delete_stream", routing::post(delete_stream))
        .route("/set_stream", routing::post(set_stream))
        .route("/set_volume", routing::post(set_volume))
        .with_state(state);

    println!("Running RDRPI @ http://{address}");
    axum::serve(listener?, app).await?;

    Ok(())
}
