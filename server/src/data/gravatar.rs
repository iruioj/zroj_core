use actix_web::web::Bytes;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use super::error::DataError;
use super::types::EmailAddress;

pub struct DefaultDB {
    cdn_base: PathBuf, // http://www.gravatar.com/avatar/
}

impl DefaultDB {
    pub fn new(cdn_base: &str) -> Self {
        Self {
            cdn_base: cdn_base.into(),
        }
    }
}

fn hash(email: &EmailAddress) -> String {
    passwd::md5_hash(&email.to_string().to_lowercase())
}

/// hint: create client _inside_ `HttpServer::new` closure to have one per worker thread
pub struct GravatarClient(awc::Client);

impl GravatarClient {
    pub fn new(cfg: Arc<rustls::ClientConfig>) -> Self {
        Self(awc::Client::builder()
            .timeout(Duration::from_secs(30))
            .add_default_header(("user-agent", r#"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"#))
            // a "connector" wraps the stream into an encrypted connection
            .connector(awc::Connector::new().rustls_0_22(cfg))
            .finish())
    }
}

impl DefaultDB {
    /// return content of the jpg image
    pub async fn fetch(
        &self,
        client: Arc<GravatarClient>,
        email: &EmailAddress,
    ) -> Result<Bytes, DataError> {
        let hash = hash(email);

        let url = self.cdn_base.join(&hash);
        dbg!(url.display());
        let mut res = client
            .0
            .get(url.to_str().unwrap())
            .send()
            .await
            .map_err(|err| anyhow::anyhow!(err.to_string()).context("send request"))?;
        eprintln!("get response");
        let img = res.body().await.unwrap();
        eprintln!("get body");

        Ok(img)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_it() {
        // let db = DefaultDB::new("https://cn.gravatar.com/avatar/".into());
        let db = DefaultDB::new("https://sdn.geekzu.org/avatar/");
        let gclient = Arc::new(GravatarClient::new(Arc::new(crate::rustls_config())));
        let _ = dbg!(
            db.fetch(gclient, &EmailAddress::new("jy.cat@qq.com").unwrap())
                .await
        )
        .unwrap();
        // use std::io::Write;
        // let mut f = std::fs::File::options()
        //     .create(true)
        //     .write(true)
        //     .open("./avatar.jpg")
        //     .unwrap();
        // f.write_all(&content).unwrap();
    }
}
