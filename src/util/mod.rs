/// Basic thread pool utility.
pub mod thread_pool;

/// Utility for creating mock trait implementations.
#[cfg(test)]
pub mod mock;

/// Stream utility for combining Read and Write traits into one.
pub mod stream;

pub fn get_content_type(path: &str) -> &'static str {
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
