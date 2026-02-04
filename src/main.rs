use crate::types::{AppState, ChangeStream, Index, SetVolume, Stream};
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
        paused: state.paused,
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

    state.streams.push((form.name, form.address));
    state.selection = state.streams.len() - 1;

    io::write_streams(&state.stream_file, &state.streams).unwrap();
    io::stop_stream(&mut state).await.unwrap();
    state.paused = io::start_stream(&mut state).await.unwrap();

    Redirect::to("/")
}

async fn change_stream(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(form): Form<ChangeStream>,
) -> Redirect {
    let mut state = state.lock().await;

    state.paused = if state.selection == form.selection {
        io::pause_stream(&mut state).await.unwrap()
    } else {
        state.selection = form.selection;
        io::stop_stream(&mut state).await.unwrap();
        io::start_stream(&mut state).await.unwrap()
    };

    Redirect::to("/")
}

async fn delete_stream(State(state): State<Arc<Mutex<AppState>>>) -> Redirect {
    let mut state = state.lock().await;

    if state.streams.is_empty() {
        return Redirect::to("/");
    }

    let selection = state.selection;
    state.streams.remove(selection);
    state.selection = selection.saturating_sub(1);

    io::write_streams(&state.stream_file, &state.streams).unwrap();
    io::stop_stream(&mut state).await.unwrap();
    state.paused = io::start_stream(&mut state).await.unwrap();

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
        paused: true,
        process: None,
        selection: 0,
        stream_file: stream_file.to_string(),
        streams: io::read_streams(&stream_file)?,
        volume: io::get_volume().await?,
    }));
    let app = Router::new()
        .route("/", routing::get(index))
        .route("/add_stream", routing::post(add_stream))
        .route("/change_stream", routing::post(change_stream))
        .route("/delete_stream", routing::post(delete_stream))
        .route("/set_volume", routing::post(set_volume))
        .with_state(state);

    println!("Running RDRPI @ http://{address}");
    axum::serve(listener?, app).await?;

    Ok(())
}
