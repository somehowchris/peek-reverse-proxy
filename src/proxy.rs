use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::IpAddr;
use std::str;

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

    if format_log_as_json {
        info!("{{\"path\": \"{}\", \"query\": {}, \"method\": \"{}\", \"version\": \"{:?}\", \"headers\": {}, \"body\": {}}}",
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
            Path: {}
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
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    struct Message {
        message: String,
    }

    use hyper_reverse_proxy::ProxyError::{HyperError, InvalidUri};

    match hyper_reverse_proxy::call(client_ip, &destination, req).await {
        Ok(response) => Ok(response),
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
