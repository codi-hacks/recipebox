use std::sync::Arc;
use std::thread::{sleep, spawn};
use std::time::{Duration};

use recipebox::common::request::Request;
use recipebox::common::response::Response;
use recipebox::server::{Config, Router};
use recipebox::server;
use recipebox::server::ListenerResult::{Next, SendResponseArc};

use crate::util::curl;

pub fn test_server(config: Config, num_connections: usize, messages: Vec<(Request, Response)>) {
    let addr = config.addr.clone();
    start_server(config, &messages);

    let messages = Arc::new(messages);

    let mut handlers = vec![];
    for _ in 0..num_connections {
        let addr = addr.clone();
        let messages = Arc::clone(&messages);
        handlers.push(spawn(move || {
            let requests: Vec<&Request> = messages.iter().map(|(req, _)| req).collect();
            let expected_output: Vec<u8> = messages.iter().flat_map(|(_, res)| &res.body).copied().collect();
            let expected_output = String::from_utf8_lossy(&expected_output).to_string();

            let actual_output = curl::requests(&addr, &requests, false);
            assert_eq!(actual_output, expected_output);
        }));
    }

    for handler in handlers {
        handler.join().unwrap();
    }
}

fn start_server(mut server_config: Config, messages: &Vec<(Request, Response)>) {
    server_config.router = get_router(messages);

    spawn(|| {
        server::listen_http(server_config).unwrap()
    });
    sleep(Duration::from_millis(100));
}

fn get_router(messages: &Vec<(Request, Response)>) -> Router {
    let mut router = Router::new();

    for (request, response) in messages {
        let uri = &request.uri;
        let response = Arc::new(response.clone());
        let request = request.clone();
        router.on(uri, move |_, req|
            if request.eq(req) {
                SendResponseArc(response.clone())
            } else {
                Next
            },
        );
    }

    router
}
