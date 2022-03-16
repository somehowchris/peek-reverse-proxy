use env_logger::Env;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use log::{error, info};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::str;
use url::Url;

async fn handle(
    client_ip: IpAddr,
    mut req: Request<Body>,
    destination: String,
) -> Result<Response<Body>, Infallible> {
    let mut query = HashMap::new();

    for item in req.uri().query().unwrap_or("").split("&") {
        let kv = item.split("=").map(|el| el.to_owned()).collect::<Vec<_>>();
        query.insert(
            kv.get(0).map(|el| el.to_owned()).unwrap(),
            kv.get(1).map(|el| el.to_owned()).unwrap(),
        );
    }

    let body = hyper::body::to_bytes(&mut req.body_mut()).await.unwrap();

    info!(
        "Path: {}
        Query: {:?}
        Method: {}
        Version: {:?}
        Headers: {:?}
        Body: {}",
        req.uri().path(),
        query,
        req.method(),
        req.version(),
        req.headers(),
        str::from_utf8(body.as_ref()).unwrap(),
    );

    match hyper_reverse_proxy::call(client_ip, &destination, req).await {
        Ok(response) => Ok(response),
        Err(_error) => {
            error!("{:?}", _error);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap())
        }
    }
}

fn validate_uri(url: &str) -> Result<Url, &str> {
    let parsed_url = url.parse::<Url>();
    if let Ok(url) = parsed_url {
        Ok(url)
    } else {
        Err("Environment variable 'DESTINATION_URL' needs to be defined as a valid url")
    }
}

fn validate_address(address: &str) -> Result<SocketAddr, &str> {
    let parsed_address = address.parse::<SocketAddr>();
    if let Ok(address) = parsed_address {
        if let Ok(socket_addr) = address.to_string().parse() {
            Ok(socket_addr)
        } else {
            Err("Could not parse ip:port of environment variable 'HOST_ADDRESS'")
        }
    } else {
        Err("Environment variable 'HOST_ADDRESS' needs to be defined as a valid hosting address")
    }
}

#[tokio::main]
async fn main() {
    // Setup logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // validate local address setup
    if let Ok(address_str) = std::env::var("HOST_ADDRESS") {
        match validate_address(&address_str) {
            Ok(socket_addr) => {
                // validate destination
                if let Ok(destination_str) = std::env::var("DESTINATION_URL") {
                    match validate_uri(&destination_str) {
                        Ok(_url) => {
                            // Setup proxy
                            let make_svc = make_service_fn(|conn: &AddrStream| {
                                let remote_addr = conn.remote_addr().ip();
                                async move {
                                    Ok::<_, Infallible>(service_fn(move |req| {
                                        handle(
                                            remote_addr,
                                            req,
                                            std::env::var("DESTINATION_URL").unwrap(),
                                        )
                                    }))
                                }
                            });

                            let server = Server::bind(&socket_addr).serve(make_svc);

                            info!("Running server on {:?}", socket_addr);

                            // Run proxy
                            if let Err(e) = server.await {
                                error!("server error: {}", e);
                            }
                        }
                        Err(message) => {
                            error!("{}", message);
                            std::process::exit(1);
                        }
                    }
                } else {
                    error!("Environment variable 'DESTINATION_URL' needs to be defined");
                    std::process::exit(1);
                }
            }
            Err(message) => {
                error!("{}", message);
                std::process::exit(1);
            }
        }
    } else {
        error!("Environment variable 'HOST_ADDRESS' needs to be defined");
        std::process::exit(1);
    }
}
