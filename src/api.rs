use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct BatchRequest {
    timeout_s: Option<f64>,
    requests: Vec<Request>
}

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    id: String,
    url: String,
    method: String, // more specific?
    body: String,
    headers: HashMap<String,String>, // something better from http?
    // response_headers: Vec<String>, 
}

#[derive(Clone, Debug, Serialize)]
pub struct BatchResponse {
    responses: Vec<Response>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Response {
    id: String,
    code: u16, // http::StatusCode
    body: String,
    // headers: HashMap<String, String>,
}

/// response body when there is no space in the queue
#[derive(Clone, Debug, Serialize)]
pub struct Backpressure {
    queue_capacity: u32,
    queue_free_space: u32,
}
