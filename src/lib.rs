#![feature(file_create_new)]

/// Command-line argument parser
pub mod args;
/// HTTP data types.
pub mod common;
/// Components for running an HTTP server and handling requests.
pub mod server;

/// Functions for dealing with page caching
pub mod cache;

/// Data models for dashboard form POSTs
pub mod forms;

/// Procedures to run on application boot up
pub mod setup;

/// Utility components.
pub mod util;

/// Components for parsing HTTP requests and responses.
pub(crate) mod parse;
