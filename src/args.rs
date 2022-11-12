use clap::Parser;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("GIT_HASH"), ")");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// The HTTP server for the recipebox. Specify a directory to store recipes to get started!
#[derive(Parser)]
#[command(author = AUTHORS, version = VERSION, about)]
pub struct Args {
    /// (Optional) Host name or IP address to serve from.
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,
    #[arg(short, long, default_value_t = 4000)]
    /// (Optional) Port number to open on host.
    pub port: usize
}
