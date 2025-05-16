# Rust EPUB Metadata Web Server

This project is a simple web server built with Rust that provides an API for uploading EPUB files and retrieving their metadata. It utilizes the Actix web framework for handling HTTP requests.

## Project Structure

```
rust-web-server
├── src
│   ├── main.rs          # Entry point of the application
│   ├── api              # Contains API endpoint definitions
│   │   └── mod.rs       # API module
│   ├── services         # Contains business logic
│   │   └── epub_parser.rs # EPUB parsing logic
│   └── models           # Data structures
│       └── metadata.rs  # EPUB metadata representation
├── Cargo.toml           # Cargo configuration file
└── README.md            # Project documentation
```

## Setup Instructions

1. **Clone the repository:**
   ```
   git clone <repository-url>
   cd rust-web-server
   ```

2. **Install Rust:**
   Follow the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install) to install Rust and Cargo.

3. **Build the project:**
   ```
   cargo build
   ```

4. **Run the server:**
   ```
   cargo run
   ```

The server will start and listen for incoming requests.

## API Usage

### Upload EPUB File

- **Endpoint:** `POST /upload`
- **Request Body:** Multipart form data containing the EPUB file.
- **Response:** JSON object containing the metadata of the EPUB file.

### Example Request

```bash
curl -X POST http://localhost:8000/upload -F "file=@path/to/your/file.epub"
```

### Example Response

```json
{
  "title": "Example Title",
  "author": "Example Author",
  "publication_date": "2023-01-01"
}
```

## Dependencies

This project uses the following dependencies:

- Actix-web for the web server functionality.
- Additional libraries for handling EPUB file parsing (to be specified in `Cargo.toml`).

## License

This project is licensed under the MIT License. See the LICENSE file for details.