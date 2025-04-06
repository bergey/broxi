use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct BatchRequest {
    timeout_s: Option<f64>,
    requests: Vec<Request>
}

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub id: String,
    pub url: String,
    #[serde(with = "http_serde::method")]
    pub method: http::Method, // more specific?
    pub body: String,
    #[serde(with = "http_serde::header_map")]
    pub headers: http::HeaderMap,
    // response_headers: Vec<String>, 
}

#[derive(Clone, Debug, Serialize)]
pub struct BatchResponse {
    pub responses: Vec<Response>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Response {
    pub id: String,
    #[serde(with = "http_serde::status_code")]
    pub status_code: http::StatusCode,
    pub body: Vec<u8>,
    // headers: HashMap<String, String>,
}

/// response body when there is no space in the queue
#[derive(Clone, Debug, Serialize)]
pub struct Backpressure {
    pub queue_capacity: u32,
    pub queue_free_space: u32,
}
