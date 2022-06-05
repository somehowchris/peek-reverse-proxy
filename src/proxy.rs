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
    req: Request<Body>,
    destination: String,
    format_log_as_json: bool,
    pretty_json_fields: bool,
) -> Result<Response<Body>, Infallible> {
    let x_request_id_header = hyper::header::HeaderName::from_static("x-request-id");

    let request_id = if req.headers().contains_key(&x_request_id_header) {
        req.headers()
            .get(&x_request_id_header)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    } else {
        uuid::Uuid::new_v4().to_string()
    };

    let query: Option<String> = req.uri().query().map(|e| e.to_owned());
    let headers = req.headers().to_owned();
    let path = req.uri().path().to_owned();
    let method = req.method().to_owned();
    let version = req.version().to_owned();
    let request_id_forged = request_id.to_owned();
    let mut request_parts = req.into_parts();

    let body = hyper::body::to_bytes(&mut request_parts.1).await.unwrap();
    let body_clone = body.clone();

    let print_req = tokio::spawn(async move {
        let mut query_map = HashMap::new();
        for item in query.unwrap_or_else(|| "".to_owned()).split('&') {
            if !item.is_empty() {
                let kv = item.split('=').map(|el| el.to_owned()).collect::<Vec<_>>();
                query_map.insert(
                    kv.get(0).map(|el| el.to_owned()).unwrap(),
                    kv.get(1)
                        .map(|el| el.to_owned())
                        .unwrap_or_else(|| "".to_string()),
                );
            }
        }

        let body_output = if pretty_json_fields {
            if let Ok(body_value) =
                serde_json::from_str::<serde_json::Value>(str::from_utf8(&*body_clone).unwrap())
            {
                serde_json::to_string_pretty(&body_value).unwrap()
            } else {
                str::from_utf8(&*body_clone).unwrap().to_string()
            }
        } else {
            str::from_utf8(&*body_clone).unwrap().to_string()
        };

        let query_output = if pretty_json_fields {
            serde_json::to_string_pretty(&query_map).unwrap()
        } else {
            format!("{:?}", query_map)
        };

        let headers_output = if pretty_json_fields {
            serde_json::to_string_pretty(&format!("{:?}", &headers)).unwrap()
        } else {
            format!("{:?}", headers)
        };

        if format_log_as_json {
            info!("{{\"type\":\"request\"\"requestId\":\"{}\",\"path\": \"{}\", \"query\": {}, \"method\": \"{}\", \"version\": \"{:?}\", \"headers\": {}, \"body\": {}}}",
                request_id_forged,
                path,
                query_output,
                method,
                version,
                headers_output,
                body_output,
            );
        } else {
            info!(
                "
-----------------
RequestId: {}
Path: {}
Query: {}
Method: {}
Version: {:?}
Headers: {}
Body: {}
-----------------",
                request_id_forged, path, query_output, method, version, headers_output, body_output,
            );
        }
    });

    match hyper_reverse_proxy::call(
        client_ip,
        &destination,
        Request::from_parts(request_parts.0, Body::from(body)),
    )
    .await
    {
        Ok(response) => {
            let mut response_parts = response.into_parts();
            let (body_output, body) =
                if let Ok(body) = hyper::body::to_bytes(&mut response_parts.1).await {
                    if pretty_json_fields {
                        if let Ok(body_value) = serde_json::from_str::<serde_json::Value>(
                            str::from_utf8(body.as_ref()).unwrap(),
                        ) {
                            (serde_json::to_string_pretty(&body_value).unwrap(), body)
                        } else {
                            (str::from_utf8(body.as_ref()).unwrap().to_string(), body)
                        }
                    } else {
                        (str::from_utf8(body.as_ref()).unwrap().to_string(), body)
                    }
                } else {
                    ("".to_string(), ([].as_slice() as &[u8]).into())
                };

            let headers_output = if pretty_json_fields {
                serde_json::to_string_pretty(&format!("{:?}", &response_parts.0.headers)).unwrap()
            } else {
                format!("{:?}", response_parts.0.headers)
            };

            if format_log_as_json {
                info!(
                    "{{\"type\":\"response\"\"requestId\":\"{}\", \"headers\": {}, \"body\": {}, \"statusCode\": \"{}\"}}",
                    request_id, headers_output, body_output, response_parts.0.status,
                );
            } else {
                info!(
                    "
-----------------
RequestId: {}
Headers: {}
StatusCode: {}
Body: {}
-----------------",
                    request_id, headers_output, response_parts.0.status, body_output,
                );
            }
            print_req.await.unwrap();

            let resp = Response::from_parts(response_parts.0, Body::from(body));
            Ok(resp)
        }
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
