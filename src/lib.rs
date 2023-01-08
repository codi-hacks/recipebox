#![feature(file_create_new)]

/// Command-line argument parser
pub mod args;
/// Functions for dealing with page caching
pub mod cache;
/// Data store for accessing cache data when building request responses
pub mod data;
/// Data models for dashboard form POSTs
pub mod forms;
/// HTTP route definitions and response builders
pub mod router;
/// Procedures to run on application boot up
pub mod setup;
