use crate::outgoing;

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use tracing::*;

const MAX_REQUEST_BYTES: usize = 2_usize.pow(20); // 1 MB

pub async fn handle_proxy_request(
    req: Request<hyper::body::Incoming>,
) -> anyhow::Result<Response<Full<Bytes>>> {
    match req.uri().path() {
        "/proxy" => {
            let body = http_body_util::Limited::new(req.into_body(), MAX_REQUEST_BYTES);
            match body.collect().await {
                Err(err) => {
                    error!("http error: {:?}", err);
                    super::HTTP_4xx.inc();
                    empty_http_response(StatusCode::BAD_REQUEST)
                }
                Ok(all) => match serde_json::from_str(std::str::from_utf8(&all.to_bytes())?) {
                    Err(_) => {
                        super::HTTP_4xx.inc();
                        empty_http_response(StatusCode::BAD_REQUEST)
                    }
                    Ok(req) => {
                        let response = outgoing::make_one_request(req).await?;
                        let body = serde_json::to_string(&response)?;
                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Full::<Bytes>::from(body))?)
                    }
                },
            }
        }
        path => {
            super::HTTP_4xx.inc();
            warn!("unexpected request to {}", path);
            empty_http_response(StatusCode::NOT_FOUND)
        }
    }
}

fn empty_http_response(status_code: StatusCode) -> Result<Response<Full<Bytes>>, anyhow::Error> {
    Ok(Response::builder()
        .status(status_code)
        .body(Full::<Bytes>::from(""))?)
}
