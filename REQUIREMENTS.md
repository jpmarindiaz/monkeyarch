

📁 Raspberry Pi Web File Manager

Technical Specification (Rust)

1. Purpose

Build a lightweight web-based file manager running on a Raspberry Pi Zero 2 W that allows a user on the local network to:
	•	Browse directories
	•	Upload MP3 and image files
	•	Move / rename files
	•	Create folders
	•	Delete files/folders

Access is via a browser over HTTP.
No cloud services. Local network only.

⸻

2. Target Platform
	•	Hardware: Raspberry Pi Zero 2 W
	•	OS: Raspberry Pi OS (64-bit or 32-bit)
	•	CPU: ARMv7
	•	RAM: 512 MB
	•	Network: LAN (Wi-Fi or Ethernet via USB)

⸻

3. Architecture Overview

3.1 High-level design

Browser (HTML/JS)
        ↓ HTTP (JSON + multipart)
Rust Web Server
        ↓
Filesystem (restricted root directory)

	•	Single Rust binary
	•	Serves:
	•	REST-like JSON API
	•	Static frontend (HTML + JS)
	•	No database

⸻

4. Technology Stack

Backend (Rust)
	•	Rust stable
	•	Web framework (choose one):
	•	Axum (preferred) OR
	•	Actix-web
	•	Async runtime: Tokio
	•	File uploads: multipart/form-data
	•	Serialization: serde, serde_json

Frontend
	•	Plain HTML + CSS
	•	Vanilla JavaScript (no framework)
	•	Fetch API
	•	Drag-and-drop optional (nice to have)

⸻

5. Filesystem Constraints (IMPORTANT)

5.1 Root directory jail

All operations MUST be restricted to a single configurable root, e.g.:

/home/pi/media/

Rules:
	•	No access outside root
	•	Reject any path traversal (.., symlinks escaping root)
	•	Canonicalize paths before use

⸻

6. HTTP API Specification

6.1 List directory

GET /api/list

Query:

path=/relative/path

Response:

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

Errors:
	•	400 invalid path
	•	403 outside root
	•	404 not found

⸻

6.2 Upload file

POST /api/upload?path=/relative/path

Content-Type:

multipart/form-data

Rules:
	•	Accept only:
	•	audio/mpeg
	•	image/*
	•	Max file size (configurable, default 100 MB)
	•	Overwrite behavior:
	•	default: reject if exists
	•	optional flag: overwrite=true

Response:

{ "status": "ok" }


⸻

6.3 Move / rename

POST /api/move

Body:

{
  "from": "music/old.mp3",
  "to": "music/new.mp3"
}

Rules:
	•	Same filesystem only
	•	Reject overwrites unless explicitly allowed

⸻

6.4 Create directory

POST /api/mkdir

Body:

{
  "path": "music/new_album"
}


⸻

6.5 Delete

POST /api/delete

Body:

{
  "path": "music/old.mp3",
  "recursive": false
}


⸻

7. Frontend Requirements

7.1 Features
	•	Directory tree or breadcrumb navigation
	•	File list (name, size, type)
	•	Upload button (and drag-drop if easy)
	•	Move / rename (modal or inline)
	•	Delete confirmation

7.2 UI Constraints
	•	Mobile-friendly
	•	No external CDN dependencies
	•	Must work in Chromium / Firefox

⸻

8. Static File Serving
	•	Frontend served at /
	•	Assets embedded OR served from /static
	•	Prefer embedding HTML/JS via:
	•	include_str!() or
	•	static directory

⸻

9. Configuration

Via:
	•	Environment variables OR
	•	Config file (config.toml)

Configurable items:
	•	Root directory
	•	Bind address (default 0.0.0.0)
	•	Port (default 8000)
	•	Max upload size
	•	Enable/disable delete

⸻

10. Security Requirements

Minimum:
	•	Path sanitization & canonicalization
	•	MIME validation on upload
	•	File size limits

Optional but recommended:
	•	HTTP Basic Auth
	•	IP allow-list (LAN only)

⸻

11. Performance & Resource Constraints
	•	Must run under <50 MB RAM
	•	No blocking filesystem calls on async runtime
	•	Stream uploads (do not load entire file in memory)

⸻

12. Logging & Errors
	•	Structured logs (info / warn / error)
	•	Meaningful HTTP error codes
	•	JSON error responses:

{ "error": "reason" }


⸻

13. Build & Deployment

Build

cargo build --release

Run

./filemgr

Optional
	•	systemd service unit
	•	Auto-start on boot

⸻

14. Deliverables
	1.	Rust source code
	2.	README with:
	•	Build steps
	•	Config instructions
	•	API summary
	3.	Minimal frontend
	4.	Example systemd service file

⸻

15. Explicit Non-Goals
	•	No cloud sync
	•	No user accounts
	•	No media streaming (download only)

⸻

