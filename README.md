# EPUB Server API Documentation

This server provides an API for parsing, storing, and retrieving EPUB documents, as well as generating audio from the content.

## Table of Contents

- [Getting Started](#getting-started)
- [API Endpoints](#api-endpoints)
  - [Upload EPUB](#upload-epub)
  - [Get Document](#get-document)
  - [Get Chapter by Index](#get-chapter-by-index)
  - [Get Audio for Chapter](#get-audio-for-chapter)
- [Error Handling](#error-handling)
- [Examples](#examples)

## Getting Started

The server runs on `http://127.0.0.1:8081` by default. You can start it by running:

```bash
cargo run
```

## API Endpoints

### Upload EPUB

Upload an EPUB file to parse and store in the database.

- **Endpoint:** `POST /upload`
- **Content-Type:** `multipart/form-data`
- **Form Parameter:**
  - `file`: The EPUB file to upload

**Response:**

- **Success (200 OK):** Returns JSON with the document metadata and a `document_id`
- **Error (400 Bad Request):** Invalid request or non-EPUB file
- **Error (500 Internal Server Error):** Server-side processing error

**Example:**

```bash
curl -X POST http://127.0.0.1:8081/upload -F "file=@path/to/your/book.epub"
```

### Get Document

Retrieves metadata and chapter information for a specific document.

- **Endpoint:** `GET /document/{id}`
- **Parameters:**
  - `id`: The document ID (integer)

**Response:**

- **Success (200 OK):** JSON containing document metadata and chapters
- **Error (404 Not Found):** Document not found

**Example:**

```bash
curl http://127.0.0.1:8081/document/1
```

### Get Chapter by Index

Retrieve a specific chapter by its index.

- **Endpoint:** `GET /document/{id}/chapter/{index}`
- **Parameters:**
  - `id`: The document ID (integer)
  - `index`: The chapter index (integer)

**Response:**

- **Success (200 OK):** HTML content of the chapter
- **Error (404 Not Found):** Chapter not found
- **Error (500 Internal Server Error):** Server-side processing error

**Example:**

```bash
curl http://127.0.0.1:8081/document/1/chapter/0
```

### Get Audio for Chapter

Generate and stream audio for a specific chapter.

- **Endpoint:** `GET /document/{id}/chapter/{index}/audio`
- **Parameters:**
  - `id`: The document ID (integer)
  - `index`: The chapter index (integer)
- **Headers:**
  - `Accept-Language`: Preferred language for TTS (e.g., `en-US`, `ru-RU`). Defaults to English if not specified.

**Response:**

- **Success (200 OK):** Audio stream in WAV format
- **Error (404 Not Found):** Chapter not found
- **Error (500 Internal Server Error):** Server-side processing error

**Example:**

```bash
curl http://127.0.0.1:8081/document/1/chapter/0/audio -H "Accept-Language: en-US" --output chapter.wav
```

## Error Handling

The API returns appropriate HTTP status codes for different error conditions:

- **400 Bad Request:** Invalid request format or parameters
- **404 Not Found:** Resource not found
- **500 Internal Server Error:** Server-side processing error

Error responses include a message explaining the issue.

## Examples

### Upload an EPUB file and display the result

```bash
curl -X POST http://127.0.0.1:8081/upload -F "file=@/path/to/book.epub" | jq
```

### Get document metadata

```bash
curl http://127.0.0.1:8081/document/1 | jq
```

### Get chapter content

```bash
curl http://127.0.0.1:8081/document/1/chapter/1
```

### Get audio for a chapter with a specific language

```bash
curl http://127.0.0.1:8081/document/1/chapter/1/audio -H "Accept-Language: ru-RU" --output chapter.wav
```

### Get audio for a chapter with default language (English)

```bash
curl http://127.0.0.1:8081/document/1/chapter/1/audio --output chapter.wav
```
