use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::os::unix::net::UnixStream;
use std::io::{Write, BufReader, BufRead};
use std::fs::{read_to_string, read_dir};
use std::path::{Path, PathBuf};

// Structure to store media state
struct MediaState {
    paused: bool,
    volume: i32,
}

impl MediaState {
    fn new() -> Self {
        MediaState { paused: true, volume: 50 }
    }
}

// Serve the HTML interface
#[get("/")]
async fn index() -> impl Responder {
    match read_to_string("static/index.html") {
        Ok(contents) => HttpResponse::Ok().content_type("text/html").body(contents),
        Err(_) => HttpResponse::InternalServerError().body("Could not load interface"),
    }
}

// Handle MPV commands
#[post("/mpv/{command}")]
async fn control_mpv(path: web::Path<String>, state: web::Data<Arc<Mutex<MediaState>>>) -> impl Responder {
    let command = path.into_inner();
    let json_command = match command.as_str() {
        "play" => json!({"command": ["set_property", "pause", false]}),
        "pause" => json!({"command": ["set_property", "pause", true]}),
        "stop" => json!({"command": ["quit"]}),
        "volume_up" => json!({"command": ["add", "volume", 5]}),
        "volume_down" => json!({"command": ["add", "volume", -5]}),
        "seek_forward" => json!({"command": ["seek", 10, "relative"]}),
        "seek_backward" => json!({"command": ["seek", -10, "relative"]}),
        "next_track" => match get_next_or_prev_file(true) {
            Some(next_file) => json!({"command": ["loadfile", next_file]}),
            None => return HttpResponse::InternalServerError().body("No next file found"),
        },
        "prev_track" => match get_next_or_prev_file(false) {
            Some(prev_file) => json!({"command": ["loadfile", prev_file]}),
            None => return HttpResponse::InternalServerError().body("No previous file found"),
        },
        _ => return HttpResponse::BadRequest().body("Invalid command"),
    };

    match send_to_mpv(json_command.to_string()) {
        Ok(response) => {
            update_state(state.get_ref().clone()); // Update media state
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("MPV error: {}", e)),
    }
}

// Function to update the state of the player
fn update_state(state: Arc<Mutex<MediaState>>) {
    let pause_status = send_to_mpv(json!({"command": ["get_property", "pause"]}).to_string());
    let volume_status = send_to_mpv(json!({"command": ["get_property", "volume"]}).to_string());

    let mut media_state = state.lock().unwrap();
    
    if let Ok(response) = pause_status {
        if let Some(paused) = response["data"].as_bool() {
            media_state.paused = paused;
        }
    }

    if let Ok(response) = volume_status {
        if let Some(volume) = response["data"].as_f64() {
            media_state.volume = volume as i32;
        }
    }
}

// Function to continuously display media state
fn start_status_thread(state: Arc<Mutex<MediaState>>) {
    thread::spawn(move || {
        loop {
            {
                let media_state = state.lock().unwrap();
                let status = if media_state.paused { "Paused" } else { "Playing" };
                println!("Video Status = {} : Volume = {}", status, media_state.volume);
            }
            thread::sleep(Duration::from_secs(2));
        }
    });
}

// Get next or previous file in the current folder
fn get_next_or_prev_file(next: bool) -> Option<String> {
    let socket_path = "/tmp/mpvsocket";
    if !Path::new(socket_path).exists() {
        return None;
    }

    // Get currently playing file
    let current_file_command = json!({"command": ["get_property", "path"]}).to_string();
    if let Ok(response) = send_to_mpv(current_file_command) {
        if let Some(current_file) = response["data"].as_str() {
            let current_path = Path::new(current_file);
            let folder = current_path.parent()?;

            // Get sorted list of files in the folder
            let mut files: Vec<PathBuf> = read_dir(folder)
                .ok()?
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .filter(|path| path.is_file())
                .collect();

            files.sort();

            // Find the index of the current file
            if let Some(pos) = files.iter().position(|p| p == current_path) {
                if next && pos + 1 < files.len() {
                    return files[pos + 1].to_str().map(|s| s.to_string());
                } else if !next && pos > 0 {
                    return files[pos - 1].to_str().map(|s| s.to_string());
                }
            }
        }
    }
    None
}

// Send JSON commands to MPV IPC socket
fn send_to_mpv(command: String) -> std::io::Result<Value> {
    let socket_path = "/tmp/mpvsocket";

    if !Path::new(socket_path).exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "MPV socket not found"));
    }

    let mut stream = UnixStream::connect(socket_path)?;
    writeln!(stream, "{}", command)?;
    stream.flush()?; 

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;

    match serde_json::from_str::<Value>(&response.trim()) {
        Ok(json) => Ok(json),
        Err(_) => Ok(json!({"error": "Invalid response from MPV"})),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = Arc::new(Mutex::new(MediaState::new()));
    start_status_thread(state.clone());

    println!("🚀 MPV Controller running at http://0.0.0.0:8080/");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(index)
            .service(control_mpv)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
