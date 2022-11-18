use std::fmt::{Display, Formatter};

/// An HTTP method.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Method {
    /// GET method.
    GET,
    /// POST method.
    POST,
    /// DELETE method.
    DELETE,
    /// PUT method
    PUT,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// TODO macros
impl Method {
    /// Converts the given string to a method. Methods are case sensitive. Returns None if no Method matches.
    pub fn try_from_str(s: &str) -> Option<Method> {
        match s {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "DELETE" => Some(Method::DELETE),
            "PUT" => Some(Method::PUT),
            _ => None
        }
    }

    /// Converts the given string to a method. Methods are case sensitive. Returns None if no Method matches.
    pub fn try_from_bytes(s: &[u8]) -> Option<Method> {
        match s {
            b"GET" => Some(Method::GET),
            b"POST" => Some(Method::POST),
            b"DELETE" => Some(Method::DELETE),
            b"PUT" => Some(Method::PUT),
            _ => None
        }
    }
}