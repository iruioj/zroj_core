//! fetch gravatar using https client

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::data::types::EmailAddress;

fn hash(email: &EmailAddress) -> String {
    passwd::md5_hash(&email.to_string().to_lowercase())
}

/// hint: create client _inside_ `HttpServer::new` closure to have one per worker thread
pub struct GravatarClient {
    cdn_base: PathBuf, // http://www.gravatar.com/avatar/
    client: awc::Client,
}

impl GravatarClient {
    pub fn new(
        cdn_base: &std::path::Path, // http://www.gravatar.com/avatar/
        cfg: Arc<rustls::ClientConfig>,
    ) -> Self {
        let client = awc::Client::builder()
            .timeout(Duration::from_secs(30))
            .add_default_header(("user-agent", r#"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"#))
            // a "connector" wraps the stream into an encrypted connection
            .connector(awc::Connector::new().rustls_0_22(cfg))
            .finish();
        Self {
            cdn_base: cdn_base.to_path_buf(),
            client,
        }
    }

    /// return content of the jpg image
    pub async fn fetch(&self, email: &EmailAddress) -> anyhow::Result<bytes::Bytes> {
        let hash = hash(email);

        let url = self.cdn_base.join(&hash);
        dbg!(url.display());
        let mut res = self
            .client
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
        let gclient = GravatarClient::new(
            "https://sdn.geekzu.org/avatar/".as_ref(),
            Arc::new(crate::utils::rustls_config()),
        );
        let _ = dbg!(
            gclient
                .fetch(&EmailAddress::new("jy.cat@qq.com").unwrap())
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
