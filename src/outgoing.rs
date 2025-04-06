use crate::api;

use anyhow::{Result, anyhow};
use http_body_util::{BodyExt, Full};
use hyper::Request;
use hyper::body::Bytes;
use hyper::client::conn::http1::SendRequest;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

const MAX_RESPONSE_BYTES: usize = 2_usize.pow(14); // 16 kB

pub async fn make_one_request(req: api::Request) -> Result<api::BatchResponse> {
    // TODO reuse connections, keep a pool
    let mut conn = connect_once(&req.url).await?;
    let request_id = req.id.clone();
    let response = conn.send_request(build_request(req)?).await?;
    let status = response.status();
    let body = {
        let limited = http_body_util::Limited::new(response.into_body(), MAX_RESPONSE_BYTES);
        let collected = limited
            .collect()
            .await
            .or(Err(anyhow!("error while waiting for downstream response")))?;
        let bytes = collected.to_bytes();
        let mut vec = Vec::new();
        vec.extend(&bytes);
        vec
    };
    let mut responses = Vec::new();
    responses.push(api::Response {
        id: request_id,
        status_code: status,
        body,
    });
    Ok(api::BatchResponse { responses })
}

async fn connect_once(address: &str) -> Result<SendRequest<Full<Bytes>>> {
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });
    Ok(sender)
}

fn build_request(req: api::Request) -> Result<Request<Full<Bytes>>> {
    let mut builder = Request::builder().method(req.method).uri(req.url);
    for (key, val) in &req.headers {
        builder = builder.header(key, val)
    }
    Ok(builder.body(Full::<Bytes>::from(req.body))?)
}
