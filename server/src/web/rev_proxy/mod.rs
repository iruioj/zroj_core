use std::{net::SocketAddr, path::PathBuf, time::Duration};

use actix_web::{
    http::header::{HeaderMap, HeaderName},
    web::Payload,
    Error, HttpRequest, HttpResponse,
};
use futures::TryStreamExt;
use lazy_static::lazy_static;

pub mod handler;

/// 反向代理配置，目前主要用于开发
pub struct RevProxy {
    /// 如果返回 None 则不代理，否则按照修改后的 path 代理
    ///
    /// 默认转发所有路径，不做修改
    path_transform: Box<dyn Fn(&str) -> Option<String> + Sync + Send>,
    /// 转发的 prefix url
    base_url: PathBuf,
    /// 默认 60s
    timeout: Duration,
}

impl RevProxy {
    /// 如果返回 None 则不代理，否则按照修改后的 path 代理
    ///
    /// 默认转发所有路径，不做修改
    pub fn path_trans(
        self,
        trans: impl Fn(&str) -> Option<String> + Sync + Send + 'static,
    ) -> Self {
        Self {
            path_transform: Box::new(trans),
            base_url: self.base_url.clone(),
            timeout: self.timeout,
        }
    }
    pub fn create(forward_base_url: impl AsRef<str>) -> Self {
        Self {
            path_transform: Box::new(|s| Some(s.to_owned())),
            base_url: forward_base_url.as_ref().into(),
            timeout: Duration::from_secs(60),
        }
    }

    fn x_forwarded_for_value(&self, req: &HttpRequest) -> String {
        let mut result = String::new();

        for (key, value) in req.headers() {
            if key == *HEADER_X_FORWARDED_FOR {
                result.push_str(value.to_str().unwrap());
                break;
            }
        }

        // adds client IP address
        // to x-forwarded-for header
        // if it's available
        if let Some(peer_addr) = req.peer_addr() {
            add_client_ip(&mut result, peer_addr);
        }

        result
    }

    fn forward_uri(&self, req: &HttpRequest) -> String {
        let forward_url = self.base_url.display();

        let forward_uri = match req.uri().query() {
            Some(query) => format!(
                "{}{}?{}",
                forward_url,
                (self.path_transform)(req.uri().path()).unwrap(),
                query
            ),
            None => format!("{}{}", forward_url, req.uri().path()),
        };

        forward_uri
    }

    // pub fn path_trans(&self, path: &str) -> Option<String> {
    //     (self.0.path_transform)(path.to_owned())
    // }

    /// 目前是把 body 直接当 bytes 提取
    pub async fn forward(
        &self,
        client: &awc::Client,
        http_req: HttpRequest,
        payload: Payload,
    ) -> Result<HttpResponse, Error> {
        let payload = payload.into_inner();
        // let (http_req, payload) = req.parts_mut();
        let mut forward_req = client
            .request(http_req.method().to_owned(), self.forward_uri(&http_req))
            .timeout(self.timeout);

        // remove_connection_headers(forward_req.headers_mut());
        // remove_request_hop_by_hop_headers(forward_req.headers_mut());

        // copy headers
        for (key, value) in http_req.headers() {
            // if !HOP_BY_HOP_HEADERS.contains(key) {
            forward_req = forward_req.insert_header((key.clone(), value.clone()));
            // }
        }
        forward_req = forward_req
            .insert_header((
                &(*HEADER_X_FORWARDED_FOR),
                self.x_forwarded_for_value(&http_req),
            ))
            .insert_header_if_none((actix_web::http::header::USER_AGENT, ""))
            .insert_header(("host", self.base_url.to_str().unwrap()));

        // println!("#### REVERSE PROXY REQUEST HEADERS");
        // for (key, value) in forward_req.headers() {
        //     println!("[{:?}] = {:?}", key, value);
        // }

        let resp = forward_req.send_stream(payload).await.map_err(|error| {
            println!("Error: {}", error);
            actix_web::error::ErrorInternalServerError(error)
            // error.to_string().into()
        })?;
        let mut back_rsp = HttpResponse::build(resp.status());

        // copy headers
        for (key, value) in resp.headers() {
            // if !HOP_BY_HOP_HEADERS.contains(key) {
            back_rsp.insert_header((key.clone(), value.clone()));
            // }
        }
        let back_rsp = back_rsp.streaming(resp.into_stream());

        // remove_connection_headers(back_rsp.headers_mut());

        // println!("#### REVERSE PROXY RESPONSE HEADERS");
        // for (key, value) in back_rsp.headers() {
        //     println!("[{:?}] = {:?}", key, value);
        // }

        Ok(back_rsp)
    }
}

lazy_static! {
    static ref HEADER_X_FORWARDED_FOR: HeaderName =
        HeaderName::from_lowercase(b"x-forwarded-for").unwrap();
    static ref HOP_BY_HOP_HEADERS: Vec<HeaderName> = vec![
        HeaderName::from_lowercase(b"connection").unwrap(),
        HeaderName::from_lowercase(b"proxy-connection").unwrap(),
        HeaderName::from_lowercase(b"keep-alive").unwrap(),
        HeaderName::from_lowercase(b"proxy-authenticate").unwrap(),
        HeaderName::from_lowercase(b"proxy-authorization").unwrap(),
        HeaderName::from_lowercase(b"te").unwrap(),
        HeaderName::from_lowercase(b"trailer").unwrap(),
        HeaderName::from_lowercase(b"transfer-encoding").unwrap(),
        HeaderName::from_lowercase(b"upgrade").unwrap(),
    ];
    static ref HEADER_TE: HeaderName = HeaderName::from_lowercase(b"te").unwrap();
    static ref HEADER_CONNECTION: HeaderName = HeaderName::from_lowercase(b"connection").unwrap();
}

fn add_client_ip(fwd_header_value: &mut String, client_addr: SocketAddr) {
    if !fwd_header_value.is_empty() {
        fwd_header_value.push_str(", ");
    }

    let client_ip_str = &format!("{}", client_addr.ip());
    fwd_header_value.push_str(client_ip_str);
}

fn _remove_connection_headers(headers: &mut HeaderMap) {
    let mut headers_to_delete: Vec<String> = Vec::new();
    let header_connection = &(*HEADER_CONNECTION);

    if let Some(val) = headers.get(header_connection) {
        if let Ok(connection_header_value) = val.to_str() {
            for h in connection_header_value.split(',').map(|s| s.trim()) {
                headers_to_delete.push(String::from(h));
            }
        }
    }

    for h in headers_to_delete {
        headers.remove(h);
    }
}

// https://book.hacktricks.xyz/pentesting-web/abusing-hop-by-hop-headers
fn _remove_request_hop_by_hop_headers(headers: &mut HeaderMap) {
    for h in HOP_BY_HOP_HEADERS.iter() {
        if let Some(v) = headers.get(h) {
            if v.is_empty() || (h == *HEADER_TE && v == "trailers") {
                continue;
            }
        }
        headers.remove(h);
    }
}
