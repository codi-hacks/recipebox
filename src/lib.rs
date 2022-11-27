#![feature(file_create_new)]

/// Command-line argument parser
pub mod args;
/// HTTP data types.
pub mod common;
/// Components for running an HTTP server and handling requests.
pub mod server;

/// Data models for dashboard form POSTs
pub mod forms;

/// Utility components.
pub mod util;

/// Components for parsing HTTP requests and responses.
pub(crate) mod parse;
