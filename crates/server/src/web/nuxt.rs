use actix_web::web;
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The official frontend framework dependends on the Nuxt.js, so we provide a
/// WebSocket proxy thanks to [this issue](https://github.com/actix/examples/issues/269).
pub async fn nuxt_websocket(
    req: actix_web::HttpRequest,
    mut payload: web::Payload,
    client: web::Data<awc::Client>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    tracing::info!(request = ?req, "receive nuxt websocket request");
    if !req.head().upgrade() {
        return Err(actix_web::error::ErrorBadRequest("invalid /_nuxt/ request"));
    }
    let ws_host = "127.0.0.1:3456";
    let mut ws_req = client
        .ws(format!("ws://{ws_host}{}", req.uri()))
        .set_header_if_none("origin", format!("http://{ws_host}"))
        .set_header_if_none("host", ws_host);
    for (k, v) in req.headers() {
        ws_req = ws_req.set_header_if_none(k, v)
    }
    tracing::info!(request = ?ws_req, "prepare for proxy websocket request");

    let (res, socket) = ws_req
        .connect()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    tracing::info!(proxy_response = ?res, "proxy response");

    // client response header.
    let mut resp = actix_web::HttpResponse::SwitchingProtocols();
    for (k, v) in res.headers() {
        resp.insert_header((k, v));
    }

    // check if response is switching protocol and continue.
    assert_eq!(
        res.status().as_u16(),
        actix_http::StatusCode::SWITCHING_PROTOCOLS
    );

    // take the websocket io only so we can transfer raw binary data between source and dest.
    let mut io = socket.into_parts().io;

    // a channel for push response body to stream.
    let (mut tx, rx) = futures::channel::mpsc::unbounded();

    // a buffer to read from dest and proxy it back to source.
    let mut buf = web::BytesMut::new();

    // spawn a task read payload stream and forward to websocket connection.
    actix_web::rt::spawn(async move {
        loop {
            tokio::select! {
                // body from source.
                res = payload.next() => {
                    match res {
                        None => return,
                        Some(body) => {
                            let body = body.unwrap();
                            io.write_all(&body).await.unwrap();
                        }
                    }
                }

                // body from dest.
                res = io.read_buf(&mut buf) => {
                    let size = res.unwrap();
                    let bytes = buf.split_to(size).freeze();
                    tx.send(Ok::<_, actix_web::Error>(bytes)).await.unwrap();
                }
            }
        }
    });

    let resp = resp.streaming(rx);
    tracing::info!(response = ?resp, "websocket client response");

    Ok(resp)
}
