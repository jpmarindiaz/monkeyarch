# Monkeyarch API Reference

Base URL: `http://<host>:<port>/api`

All responses are JSON (except `/api/file`). Errors return `{"error": "message"}`.

---

## List Directory

```
GET /api/list?path=<relative-path>
```

**Query Parameters:**
- `path` (optional): Relative path within root. Empty = root directory.

**Response:**
```json
{
  "path": "music/albums",
  "entries": [
    {
      "name": "song.mp3",
      "type": "file",
      "size": 3456789,
      "modified": "2025-01-15T12:34:56Z"
    },
    {
      "name": "covers",
      "type": "directory"
    }
  ]
}
```

**Entry Fields:**
- `name`: Filename or folder name
- `type`: `"file"` or `"directory"`
- `size`: File size in bytes (files only)
- `modified`: ISO 8601 timestamp (files only)

**Errors:**
- `400` - Invalid path
- `403` - Path outside root (security)
- `404` - Directory not found

---

## Get File (Download/View)

```
GET /api/file?path=<relative-path>
```

**Query Parameters:**
- `path`: Path to the file (required)

**Response:**
- Returns the raw file content with appropriate `Content-Type` header
- `Content-Disposition: inline` for viewing in browser

**Use Cases:**
- Display images: `<img src="/api/file?path=photos/cat.jpg">`
- Play audio: `<audio src="/api/file?path=music/song.mp3">`
- Download: Link directly to the URL

**Errors:**
- `400` - Invalid path
- `403` - Path outside root
- `404` - File not found

---

## Upload File

```
POST /api/upload?path=<relative-path>&overwrite=<bool>
Content-Type: multipart/form-data
```

**Query Parameters:**
- `path` (optional): Destination directory. Empty = root.
- `overwrite` (optional): `true` to overwrite existing files. Default: `false`.

**Body:**
Standard multipart form with one or more files.

**Allowed File Types:**
- `audio/mpeg` (MP3)
- `image/*` (any image type)

**Response:**
```json
{"status": "ok"}
```

**Errors:**
- `400` - Missing filename, invalid path
- `403` - Path outside root
- `409` - File already exists (when `overwrite=false`)
- `413` - File too large (exceeds `max_upload_size`)
- `415` - Unsupported media type (not MP3 or image)

**JavaScript Example:**
```javascript
async function uploadFiles(files, path = '', overwrite = false) {
  const formData = new FormData();
  for (const file of files) {
    formData.append('file', file);
  }

  const response = await fetch(
    `/api/upload?path=${encodeURIComponent(path)}&overwrite=${overwrite}`,
    { method: 'POST', body: formData }
  );

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error);
  }
  return response.json();
}
```

---

## Move / Rename

```
POST /api/move
Content-Type: application/json
```

**Body:**
```json
{
  "from": "music/old-name.mp3",
  "to": "music/new-name.mp3",
  "overwrite": false
}
```

**Fields:**
- `from`: Source path (must exist)
- `to`: Destination path
- `overwrite` (optional): Allow overwriting destination. Default: `false`.

**Response:**
```json
{"status": "ok"}
```

**Errors:**
- `400` - Invalid path, cannot move root, cannot move into itself
- `403` - Path outside root
- `404` - Source not found
- `409` - Destination exists (when `overwrite=false`)

---

## Create Directory

```
POST /api/mkdir
Content-Type: application/json
```

**Body:**
```json
{
  "path": "music/new-album"
}
```

**Response:**
```json
{"status": "ok"}
```

**Errors:**
- `400` - Invalid path
- `403` - Path outside root
- `409` - Path already exists

---

## Delete

```
POST /api/delete
Content-Type: application/json
```

**Body:**
```json
{
  "path": "music/old-song.mp3",
  "recursive": false
}
```

**Fields:**
- `path`: Path to delete
- `recursive` (optional): If `true`, delete non-empty directories. Default: `false`.

**Response:**
```json
{"status": "ok"}
```

**Errors:**
- `400` - Invalid path, cannot delete root
- `403` - Delete disabled in config, or path outside root
- `404` - Path not found
- `409` - Directory not empty (when `recursive=false`)

---

## Common Error Codes

| Code | Meaning |
|------|---------|
| 400  | Bad Request - Invalid input |
| 403  | Forbidden - Security violation or disabled feature |
| 404  | Not Found |
| 409  | Conflict - Already exists / not empty |
| 413  | Payload Too Large |
| 415  | Unsupported Media Type |
| 500  | Internal Server Error |

---

## CORS

The API does not set CORS headers. When developing locally with a separate frontend server, either:
1. Use `static_directory` config to serve frontend from the same origin
2. Use a proxy in your dev server

---

## Example: Fetch Wrapper

```javascript
const API_BASE = '/api';

async function api(endpoint, options = {}) {
  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  });

  const data = await response.json();

  if (!response.ok) {
    throw new Error(data.error || `HTTP ${response.status}`);
  }

  return data;
}

// Usage:
const files = await api('/list?path=music');
await api('/mkdir', { method: 'POST', body: JSON.stringify({ path: 'new-folder' }) });
await api('/delete', { method: 'POST', body: JSON.stringify({ path: 'old-file.mp3' }) });
```
