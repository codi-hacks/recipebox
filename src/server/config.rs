use crate::server::Router;

/// The config for an HTTP server.
pub struct Config {
    /// The address to bind the server listener to.
    pub addr: String,
    /// The number of threads to spawn for handling connections. Each thread is used for one
    /// connection at a time.
    pub connection_handler_threads: usize,
    /// The router used for handling requests.
    pub router: Router,
}
