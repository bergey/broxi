mod api;
mod incoming;
mod observability;
mod outgoing;

use anyhow::Context;
use std::env;
// use std::time::Instant;
use tracing::*;
#[macro_use]
extern crate lazy_static;
use prometheus::{self, register_int_counter, IntCounter};
// use prometheus::{register_histogram, Histogram};

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

lazy_static! {
    static ref HTTP_REQUEST: IntCounter =
        register_int_counter!("http_request", "HTTP requests started").unwrap();
    // TODO use Labels?
    static ref HTTP_200: IntCounter =
        register_int_counter!("http_200", "HTTP 200 responses sent").unwrap();
    static ref HTTP_4xx: IntCounter =
        register_int_counter!("http_4xx", "HTTP 4xx responses sent").unwrap();
    static ref HTTP_5xx: IntCounter =
        register_int_counter!("http_5xx", "HTTP 5xx responses sent").unwrap();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    observability::init()?;
    let listen: SocketAddr = env::var("LISTEN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 3000)));
    let metrics_address = env::var("METRICS_ADDRESS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 9000)));

    let tcp_listener = TcpListener::bind(listen)
        .await
        .context(format!("tcp_listener {}", listen))?;
    let metrics_listener = TcpListener::bind(metrics_address)
        .await
        .context(format!("metrics_listener {}", metrics_address))?;

    loop {
        tokio::select! {
                r_stream = tcp_listener.accept() => match r_stream {
                    Err(err) => {
                        error!("fatal http error: {}", err);
                        std::process::exit(101);
                    }
                    Ok((stream, _)) => {
                        HTTP_REQUEST.inc();

                        // Use an adapter to access something implementing `tokio::io` traits as if they implement
                        // `hyper::rt` IO traits.
                        let io = TokioIo::new(stream);

                        // Spawn a tokio task to serve multiple connections concurrently
                        tokio::task::spawn(async move {
                            if let Err(err) = http1::Builder::new()
                                .serve_connection(io, service_fn(incoming::handle_proxy_request))
                                .await
                            {
                                error!("Error serving connection: {:?}", err);
                            }
                        });
                }
            },
            r_stream = metrics_listener.accept() => match r_stream {
                Err(err) => {
                    error!("fatal http metrics error: {}", err);
                    std::process::exit(102);
                }
                Ok((stream, _)) => {
                    // Use an adapter to access something implementing `tokio::io` traits as if they implement
                    // `hyper::rt` IO traits.
                    let io = TokioIo::new(stream);

                    // Spawn a tokio task to serve multiple connections concurrently
                    tokio::task::spawn(async move {
                        if let Err(err) = http1::Builder::new()
                            .serve_connection(io, service_fn(observability::prometheus_metrics))
                            .await
                        {
                            error!("Error serving connection: {:?}", err);
                        }
                    });
                }
            }
        }
    }
}
