#[macro_use]
extern crate tracing;

use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::str;

use url::Url;

mod logging;
mod proxy;

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
    let (style, _log_level) = logging::setup_logging();

    // validate local address setup
    if let Ok(address_str) = std::env::var("HOST_ADDRESS") {
        match validate_address(&address_str) {
            Ok(socket_addr) => {
                // validate destination
                if let Ok(destination_str) = std::env::var("DESTINATION_URL") {
                    match validate_uri(&destination_str) {
                        Ok(_url) => {
                            
                            // Setup proxy
                            let pretty_svc = make_service_fn(|conn: &AddrStream| {
                                let remote_addr = conn.remote_addr().ip();
                                let print_style = style.clone();
                                async move {
                                    Ok::<_, Infallible>(service_fn(move |req| {
                                        proxy::handle(
                                            remote_addr,
                                            req,
                                            std::env::var("DESTINATION_URL").unwrap(),
                                            print_style == logging::PrintStyle::Json,
                                            print_style == logging::PrintStyle::Pretty,
                                        )
                                    }))
                                }
                            });

                            let plain_svc = make_service_fn(|conn: &AddrStream| {
                                let remote_addr = conn.remote_addr().ip();
                                let print_style = style.clone();
                                async move {
                                    Ok::<_, Infallible>(service_fn(move |req| {
                                        proxy::handle(
                                            remote_addr,
                                            req,
                                            std::env::var("DESTINATION_URL").unwrap(),
                                            print_style == logging::PrintStyle::Json,
                                            print_style == logging::PrintStyle::Pretty,
                                        )
                                    }))
                                }
                            });

                            info!("Running server on {:?}", socket_addr);

                            // Run proxy
                            if style == logging::PrintStyle::Pretty {
                                let server = Server::bind(&socket_addr).serve(pretty_svc);

                                if let Err(e) = server.await {
                                    error!("server error: {}", e);
                                }
                            } else {
                                let server = Server::bind(&socket_addr).serve(plain_svc);

                                if let Err(e) = server.await {
                                    error!("server error: {}", e);
                                }
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
