use crate::types::{AppState, IndexTemplate, StreamForm, VolumeForm};
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
    let template = IndexTemplate {
        streams: state.streams.iter().map(|(i, _)| i.to_string()).collect(),
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
        let stream = state.streams.get(form.selection - 1).unwrap().1.to_string();
        let process = io::set_stream(&stream).unwrap();
        state.process = Some(process);
    }

    Redirect::to("/")
}

async fn set_volume(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(form): Form<VolumeForm>,
) -> Redirect {
    state.lock().await.volume = form.volume;
    io::set_volume(form.volume).unwrap();

    Redirect::to("/")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    io::program_exists("ffmpeg")?;
    io::program_exists("wpctl")?;

    let (ip, port) = io::get_arguments();
    let address = format!("{ip}:{port}");
    let listener = TcpListener::bind(&address).await;
    if listener.is_err() {
        return Err(format!("Couldn't bind to port {port}").into());
    }

    let state = Arc::new(Mutex::new(AppState {
        process: None,
        selection: 0,
        streams: io::parse_streams()?,
        volume: io::get_volume().await?,
    }));
    let app = Router::new()
        .route("/", routing::get(index))
        .route("/set_stream", routing::post(set_stream))
        .route("/set_volume", routing::post(set_volume))
        .with_state(state);

    println!("Running RDRPI @ http://{address}");
    axum::serve(listener?, app).await?;

    Ok(())
}
