use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::sync::{Arc, RwLock};

use clap::Parser;
use recipebox::args::Args;
use recipebox::{header_map, server};
use recipebox::common::{header, status};
use recipebox::common::response::Response;
use recipebox::server::{Config, Router};
use recipebox::server::ListenerResult::{SendResponse, SendResponseArc};

fn main() -> Result<(), Error> {
    let mut router = Router::new();

    router.on("/secret/message/path", |_, _| {
        let message = b"You found the secret message!";
        SendResponse(Response {
            status: status::OK,
            headers: header_map![(header::CONTENT_LENGTH, "29")],
            body: message.to_vec(),
        })
    });

    router.route("/", file_router("./index.html"));

    let args = Args::parse();
    let addr = format!("{}:{}", args.host, args.port);

    println!("running on {}:{}", args.host, args.port);
    server::listen_http(Config {
        addr,
        connection_handler_threads: 5,
        router,
    })
}

fn file_router(directory: &'static str) -> Router {
    let mut router = Router::new();

    let cache: RwLock<HashMap<String, Arc<Response>>> = RwLock::new(HashMap::new());

    router.on_prefix("", move |uri, _| {
        let mut path = String::from(directory);
        path.push_str(uri);

        if path.ends_with('/') {
            path.push_str("index.html")
        }

        if let Some(response) = cache.read().unwrap().get(&path) { // read lock gets dropped after if statement
            return SendResponseArc(Arc::clone(response));
        }

        let response = Arc::new(file_response(&path));

        cache.write().unwrap().insert(path, Arc::clone(&response));

        SendResponseArc(response)
    });

    router
}

fn file_response(file_path: &str) -> Response {
    if let Ok(contents) = fs::read(file_path) {
        let headers = header_map![
            (header::CONTENT_LENGTH, contents.len().to_string()),
            (header::CONTENT_TYPE, get_content_type(file_path))
        ];

        return Response { status: status::OK, headers, body: contents };
    }
    status::NOT_FOUND.into()
}

fn get_content_type(path: &str) -> &'static str {
    if path.ends_with(".ico") {
        return "image/x-icon";
    } else if path.ends_with(".js") {
        return "application/javascript";
    } else if path.ends_with(".svg") {
        return "image/svg+xml";
    } else if path.ends_with(".html") {
        return "text/html";
    } else if path.ends_with(".css") {
        return "text/css";
    }
    "text/plain"
}
