# Monkeyarch

A lightweight web-based file manager for Raspberry Pi Zero 2 W.

## Features

- Browse directories
- Upload MP3 and image files
- Move/rename files and folders
- Create folders
- Delete files and folders
- Mobile-friendly web interface
- Single binary deployment

## Requirements

- Rust 1.70+ (for building)
- Raspberry Pi Zero 2 W (or compatible ARM device)

## Building

### Local build

```bash
cargo build --release
```

The binary will be at `target/release/monkeyarch`.

### Cross-compilation for Raspberry Pi

```bash
# Install cross
cargo install cross

# Build for ARM
cross build --release --target armv7-unknown-linux-gnueabihf
```

## Configuration

Configuration can be set via environment variables or a `config.toml` file.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `MONKEYARCH_ROOT_DIRECTORY` | `/home/pi/media` | Root directory for file operations |
| `MONKEYARCH_BIND_ADDRESS` | `0.0.0.0` | Listen address |
| `MONKEYARCH_PORT` | `8000` | Listen port |
| `MONKEYARCH_MAX_UPLOAD_SIZE` | `104857600` | Max upload size in bytes (100 MB) |
| `MONKEYARCH_ENABLE_DELETE` | `true` | Enable delete operations |
| `RUST_LOG` | `info` | Log level |

### Config File

Copy `config.example.toml` to `config.toml`:

```toml
root_directory = "/home/pi/media"
bind_address = "0.0.0.0"
port = 8000
max_upload_size = 104857600
enable_delete = true
```

## Running

```bash
# Using environment variables
MONKEYARCH_ROOT_DIRECTORY=/path/to/files ./monkeyarch

# Using config file
./monkeyarch
```

Access the web interface at `http://<raspberry-pi-ip>:8000`

## Deployment on Raspberry Pi

1. Copy the binary to `/usr/local/bin/`:
   ```bash
   sudo cp monkeyarch /usr/local/bin/
   sudo chmod +x /usr/local/bin/monkeyarch
   ```

2. Create the media directory:
   ```bash
   mkdir -p /home/pi/media
   ```

3. Install the systemd service:
   ```bash
   sudo cp monkeyarch.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable monkeyarch
   sudo systemctl start monkeyarch
   ```

4. Check status:
   ```bash
   sudo systemctl status monkeyarch
   ```

## API Reference

### List Directory

```
GET /api/list?path=<relative-path>
```

Response:
```json
{
  "path": "music",
  "entries": [
    {
      "name": "song.mp3",
      "type": "file",
      "size": 3456789,
      "modified": "2025-09-18T12:34:56Z"
    },
    {
      "name": "albums",
      "type": "directory"
    }
  ]
}
```

### Upload File

```
POST /api/upload?path=<relative-path>&overwrite=false
Content-Type: multipart/form-data
```

Accepted file types: `audio/mpeg`, `image/*`

### Move/Rename

```
POST /api/move
Content-Type: application/json

{
  "from": "music/old.mp3",
  "to": "music/new.mp3",
  "overwrite": false
}
```

### Create Directory

```
POST /api/mkdir
Content-Type: application/json

{
  "path": "music/new_album"
}
```

### Delete

```
POST /api/delete
Content-Type: application/json

{
  "path": "music/old.mp3",
  "recursive": false
}
```

## Security

- All file operations are restricted to the configured root directory
- Path traversal attempts are blocked
- Only MP3 and image files can be uploaded
- File size limits are enforced during streaming upload
- Symlinks escaping the root directory are rejected

## License

MIT

