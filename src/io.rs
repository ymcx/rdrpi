use crate::types::{AppState, Stream};
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{BufReader, BufWriter},
};
use tokio::{process::Command, sync::MutexGuard};

fn get_argument_value(
    arguments: &Vec<String>,
    argument_a: &str,
    argument_b: &str,
) -> Option<String> {
    arguments
        .iter()
        .position(|argument| argument == argument_a || argument == argument_b)
        .and_then(|index| {
            arguments
                .get(index + 1)
                .and_then(|value| Some(value.to_string()))
        })
}

pub fn get_arguments() -> (String, String, String) {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let ip = get_argument_value(&arguments, "-i", "--ip").unwrap_or("0.0.0.0".to_string());
    let port = get_argument_value(&arguments, "-p", "--port").unwrap_or("8080".to_string());
    let file = get_argument_value(&arguments, "-f", "--file").unwrap_or("streams.json".to_string());

    (ip, port, file)
}

pub fn set_volume(volume: u8) -> Result<(), Box<dyn Error>> {
    let volume = (volume as f32 / 100.0).to_string();
    Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_SINK@", &volume])
        .spawn()?;

    Ok(())
}

pub async fn get_volume() -> Result<u8, Box<dyn Error>> {
    let process = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_SINK@"])
        .output()
        .await?;
    let stdout = String::from_utf8_lossy(&process.stdout);
    let volume = stdout[8..12].replace('.', "").parse()?;

    Ok(volume)
}

// https://stackoverflow.com/a/35046243
pub fn program_exists(program: &str) -> Result<(), Box<dyn Error>> {
    let path = env::var("PATH")?;
    for directory in path.split(":") {
        let program_path = format!("{}/{}", directory, program);
        if fs::metadata(program_path).is_ok() {
            return Ok(());
        }
    }

    Err(format!("Couldn't find {program}, is it installed?").into())
}

pub fn read_streams(stream_file: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let file = File::open(stream_file)?;
    let reader = BufReader::new(file);
    let streams: Vec<Stream> = serde_json::from_reader(reader)?;
    let streams = streams.into_iter().map(|i| (i.name, i.address)).collect();

    Ok(streams)
}

pub fn write_streams(
    stream_file: &str,
    streams: &Vec<(String, String)>,
) -> Result<(), Box<dyn Error>> {
    let streams: Vec<Stream> = streams
        .iter()
        .map(|(name, address)| Stream {
            name: name.to_string(),
            address: address.to_string(),
        })
        .collect();

    let file = File::create(stream_file)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &streams)?;

    Ok(())
}

pub async fn start_stream(state: &mut MutexGuard<'_, AppState>) -> Result<bool, Box<dyn Error>> {
    if state.streams.is_empty() {
        return Ok(true);
    }

    let stream = state.streams[state.selection].1.to_string();
    let process = Command::new("ffmpeg")
        .args([
            "-vn", "-f", "pulse", "default", "-v", "quiet", "-i", &stream,
        ])
        .spawn()?;

    state.process = Some(process);

    Ok(false)
}

pub async fn stop_stream(state: &mut MutexGuard<'_, AppState>) -> Result<bool, Box<dyn Error>> {
    if let Some(process) = &mut state.process {
        process.kill().await?;
    }

    Ok(true)
}
