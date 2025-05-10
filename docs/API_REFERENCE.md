# NRS API Reference

This document outlines the web API endpoints provided by the NRS backend server.

## Base URL

All API endpoints are relative to the base URL, which is by default:

```
http://localhost:4321
```

The port can be changed when starting the server using the `--port` flag:

```bash
nrs serve --port 8080
```

## Endpoints

### List All Notes

```
GET /api/notes
```

Returns a list of all notes with metadata.

#### Response

```json
[
  {
    "title": "Example Note",
    "slug": "example_note",
    "preview": "This is a preview of the note content...",
    "tags": ["example", "documentation"],
    "last_modified": 1620000000
  },
  {
    "title": "Another Note",
    "slug": "another_note",
    "preview": "This is another note...",
    "tags": ["example"],
    "last_modified": 1620100000
  }
]
```

The response is sorted by last_modified date (newest first).

### Get Note Details

```
GET /api/notes/{slug}
```

Returns details for a specific note identified by its slug.

#### Parameters

- `slug`: The slug identifier of the note (required)

#### Response

```json
{
  "title": "Example Note",
  "slug": "example_note", 
  "preview": "This is a preview of the note content...",
  "tags": ["example", "documentation"],
  "last_modified": 1620000000
}
```

#### Error Responses

- `404 Not Found`: Note with the specified slug does not exist
- `500 Internal Server Error`: Failed to extract note data

### Get Graph Data

```
GET /api/graph-data
```

Returns data for visualizing the graph of notes and their connections.

#### Response

```json
{
  "nodes": [
    {
      "id": "example_note",
      "is_tag": false
    },
    {
      "id": "another_note",
      "is_tag": false
    },
    {
      "id": "example",
      "is_tag": true
    },
    {
      "id": "documentation",
      "is_tag": true
    }
  ],
  "links": [
    {
      "source": "example_note",
      "target": "another_note"
    },
    {
      "source": "example_note",
      "target": "example"
    },
    {
      "source": "example_note",
      "target": "documentation"
    },
    {
      "source": "another_note",
      "target": "example"
    }
  ]
}
```

The graph data consists of:

- `nodes`: Array of note and tag nodes
  - `id`: The identifier (slug for notes, tag name for tags)
  - `is_tag`: Boolean indicating if the node is a tag (true) or a note (false)
- `links`: Array of connections between nodes
  - `source`: The source node ID
  - `target`: The target node ID

Connections are created by:
1. Wiki-style links (`[[link]]`) in note content
2. Tags assigned to notes

## Web Routes

Besides the API endpoints, the server also serves web routes:

### Home Page

```
GET /
```

Returns the main web UI page.

### Note Page

```
GET /notes/{slug}
```

Returns the page for viewing a specific note.

### Graph Visualization

```
GET /graph
```

Returns the graph visualization page.

## Static Assets

Static assets are served from the following paths:

```
GET /assets/{filename}
```

Serves assets like JavaScript and CSS files.

```
GET /static/{filename}
```

Serves legacy static files.

## Content Types

All API responses are returned as `application/json` unless otherwise specified.

Web routes return `text/html` content.

## Error Handling

The API uses standard HTTP status codes to indicate the success or failure of a request:

- `200 OK`: Request succeeded
- `404 Not Found`: Requested resource not found
- `500 Internal Server Error`: Server error

Error responses include a simple message in the response body.