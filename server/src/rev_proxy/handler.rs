use std::cell::RefCell;

// use actix_web::guard::{Guard, GuardContext};

use super::*;

thread_local! {
    static CLIENT: RefCell<awc::Client> = RefCell::new(awc::Client::default());
}

pub async fn rev_proxy(
    req: HttpRequest,
    payload: Payload,
    cfg: actix_web::web::Data<RevProxy>,
    client: actix_web::web::Data<awc::Client>,
) -> actix_web::Result<HttpResponse> {
    // eprintln!("rev_proxy trigger {}", req.path());
    // let client = awc::Client::default();
    cfg.forward(&client, req, payload).await
}
