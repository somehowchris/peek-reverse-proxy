use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
use hyper_reverse_proxy::ProxyError::{HyperError, InvalidUri};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::IpAddr;
use std::str;

#[derive(serde::Deserialize, serde::Serialize)]
struct Message {
    message: String,
}

#[inline]
pub async fn handle(
    client_ip: IpAddr,
    mut req: Request<Body>,
    destination: String,
    format_log_as_json: bool,
    pretty_json_fields: bool,
) -> Result<Response<Body>, Infallible> {
    let mut query = HashMap::new();
    for item in req.uri().query().unwrap_or("").split('&') {
        if !item.is_empty() {
            let kv = item.split('=').map(|el| el.to_owned()).collect::<Vec<_>>();
            query.insert(
                kv.get(0).map(|el| el.to_owned()).unwrap(),
                kv.get(1)
                    .map(|el| el.to_owned())
                    .unwrap_or_else(|| "".to_string()),
            );
        }
    }

    let body = hyper::body::to_bytes(&mut req.body_mut()).await.unwrap();

    let body_output = if pretty_json_fields {
        if let Ok(body_value) =
            serde_json::from_str::<serde_json::Value>(str::from_utf8(body.as_ref()).unwrap())
        {
            serde_json::to_string_pretty(&body_value).unwrap()
        } else {
            str::from_utf8(body.as_ref()).unwrap().to_string()
        }
    } else {
        str::from_utf8(body.as_ref()).unwrap().to_string()
    };

    let query_output = if pretty_json_fields {
        serde_json::to_string_pretty(&query).unwrap()
    } else {
        format!("{:?}", query)
    };

    let headers_output = if pretty_json_fields {
        serde_json::to_string_pretty(&format!("{:?}", &req.headers())).unwrap()
    } else {
        format!("{:?}", req.headers())
    };

    let x_request_id_header = hyper::header::HeaderName::from_static("x-request-id");

    let request_id = if req.headers().contains_key(&x_request_id_header) {req.headers().get(&x_request_id_header).unwrap().to_str().unwrap().to_string()} else {uuid::Uuid::new_v4().to_string()};

    if format_log_as_json {
        info!("{{\"type\":\"request\"\"requestId\":\"{}\",\"path\": \"{}\", \"query\": {}, \"method\": \"{}\", \"version\": \"{:?}\", \"headers\": {}, \"body\": {}}}",
            request_id,
            req.uri().path(),
            query_output,
            req.method(),
            req.version(),
            headers_output,
            body_output,
        );
    } else {
        info!(
            "
            -----------------
            RequestId: {}
            Path: {}
            Query: {:?}
            Method: {}
            Version: {:?}
            Headers: {:?}
            Body: {}
            -----------------",
            request_id,
            req.uri().path(),
            query,
            req.method(),
            req.version(),
            headers_output,
            body_output,
        );
    }

    match hyper_reverse_proxy::call(client_ip, &destination, req).await {
        Ok(mut response) => {
            let body = hyper::body::to_bytes(&mut response.body_mut()).await.unwrap();

            let body_output = if pretty_json_fields {
                if let Ok(body_value) =
                    serde_json::from_str::<serde_json::Value>(str::from_utf8(body.as_ref()).unwrap())
                {
                    serde_json::to_string_pretty(&body_value).unwrap()
                } else {
                    str::from_utf8(body.as_ref()).unwrap().to_string()
                }
            } else {
                str::from_utf8(body.as_ref()).unwrap().to_string()
            };

            let headers_output = if pretty_json_fields {
                serde_json::to_string_pretty(&format!("{:?}", &response.headers())).unwrap()
            } else {
                format!("{:?}", response.headers())
            };

            if format_log_as_json {
                info!("{{\"type\":\"response\"\"requestId\":\"{}\", \"headers\": {}, \"body\": {}}}",
                    request_id,
                    headers_output,
                    body_output,
                );
            } else {
                info!(
                    "
                    -----------------
                    RequestId: {}
                    Headers: {:?}
                    Body: {}
                    -----------------",
                    request_id,
                    headers_output,
                    body_output,
                );
            }

            Ok(response)
        },
        Err(HyperError(err)) => {
            error!("{:?}", err);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&Message {
                        message: err.to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap())
        }
        Err(InvalidUri(err)) => {
            error!("{:?}", err);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&Message {
                        message: err.to_string(),
                    })
                    .unwrap(),
                ))
                .unwrap())
        }
        Err(_error) => {
            error!("{:?}", _error);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap())
        }
    }
}
