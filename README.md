# MPV Controller using Actix Web

## Overview
This project is a web-based controller for **MPV Media Player** built with **Rust** using the **Actix Web** framework. It allows users to control MPV playback through a web interface and API commands.

## Features
- Start/Stop/Pause MPV playback
- Adjust volume (increase/decrease)
- Seek forward or backward
- Skip to the next or previous track
- Monitor playback status (paused/playing, volume level)
- Web-based UI for easy interaction

## Requirements
- Rust (latest stable version recommended)
- MPV installed with IPC enabled
- Actix Web and Serde JSON dependencies

## Installation
1. Clone the repository:
   ```sh
   git clone <repository-url>
   cd <project-folder>
   ```
2. Ensure MPV is running with IPC socket:
   ```sh
   mpv --input-ipc-server=/tmp/mpvsocket <media-file>
   ```
3. Install Rust dependencies:
   ```sh
   cargo build --release
   ```
4. Run the server:
   ```sh
   cargo run
   ```

## API Endpoints
| Method | Endpoint          | Description                |
|--------|------------------|----------------------------|
| `GET`  | `/`              | Serve the web interface   |
| `POST` | `/mpv/play`      | Play media                |
| `POST` | `/mpv/pause`     | Pause media               |
| `POST` | `/mpv/stop`      | Stop MPV player           |
| `POST` | `/mpv/volume_up` | Increase volume           |
| `POST` | `/mpv/volume_down` | Decrease volume        |
| `POST` | `/mpv/seek_forward` | Seek forward 10s      |
| `POST` | `/mpv/seek_backward` | Seek backward 10s     |
| `POST` | `/mpv/next_track` | Play next media file     |
| `POST` | `/mpv/prev_track` | Play previous media file |

## Web UI
- The web interface is served from `static/index.html`
- Modify this file for a custom UI experience

## How It Works
- The API sends JSON commands to MPV via a **Unix socket** (`/tmp/mpvsocket`).
- The Rust server reads responses and updates the internal media state.
- A background thread continuously logs playback status.

## Troubleshooting
- Ensure MPV is running with `--input-ipc-server=/tmp/mpvsocket`
- Check if `/tmp/mpvsocket` exists before sending API requests
- Run the Rust server in **debug mode** for detailed logs:
  ```sh
  RUST_LOG=debug cargo run
  ```


