use std::env;
use tokio::process::{Child, Command};

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

pub fn get_arguments() -> (String, String) {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let ip = get_argument_value(&arguments, "-i", "--ip").unwrap_or("0.0.0.0".to_string());
    let port = get_argument_value(&arguments, "-p", "--port").unwrap_or("8080".to_string());

    (ip, port)
}

pub fn set_stream(stream: &str) -> Option<Child> {
    Command::new("ffmpeg")
        .args(["-vn", "-f", "pulse", "default", "-v", "quiet", "-i", stream])
        .spawn()
        .ok()
}

pub fn set_volume(volume: u8) {
    let volume = (volume as f32 / 100.0).to_string();
    Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_SINK@", &volume])
        .spawn()
        .unwrap();
}

pub async fn get_volume() -> u8 {
    let process = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_SINK@"])
        .output()
        .await
        .unwrap();
    let stdout = String::from_utf8_lossy(&process.stdout);
    let volume = stdout[8..12].replace('.', "");

    volume.parse().unwrap()
}
