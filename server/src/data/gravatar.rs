use std::io::Write;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::Duration;

use actix_files::NamedFile;
use async_trait::async_trait;
use md5::{Digest, Md5};
use store::Handle;

use super::error::Error;
use super::types::EmailAddress;

pub type GravatarDB = dyn Manager + Sync + Send;

/// do not require the manager to be thread safe
/// TODO: check if this is safe
#[async_trait(?Send)]
pub trait Manager {
    /// Get the gravatar, fetch if not cached
    async fn get(&self, email: &EmailAddress) -> Result<NamedFile, Error>;

    /// Get the gravatar, always fetch from CDN
    async fn fetch(&self, email: &EmailAddress) -> Result<NamedFile, Error>;
}

pub struct DefaultDB {
    cdn_base: PathBuf, // http://www.gravatar.com/avatar/
    dir: RwLock<Handle>,
}

impl DefaultDB {
    pub fn new(path: impl AsRef<std::path::Path>, cdn_base: String) -> Self {
        std::fs::create_dir_all(path.as_ref()).expect("creating dir");
        Self {
            cdn_base: cdn_base.into(),
            dir: RwLock::new(Handle::new(path)),
        }
    }
}

fn hash(email: &EmailAddress) -> String {
    let mut md5 = Md5::new();
    md5.update(email.to_string().to_lowercase());
    let hash = hex::encode(md5.finalize().as_slice());
    hash
}

#[async_trait(?Send)]
impl Manager for DefaultDB {
    async fn get(&self, email: &EmailAddress) -> Result<NamedFile, Error> {
        let hash = hash(email);
        let dir = self.dir.read()?;
        let path = dir.join(hash + ".jpg");
        drop(dir);

        if path.path().exists() {
            let file = NamedFile::open(path.path().to_path_buf()).unwrap();
            Ok(file)
        } else {
            self.fetch(email).await
        }
    }

    async fn fetch(&self, email: &EmailAddress) -> Result<NamedFile, Error> {
        let hash = hash(email);

        let url = self.cdn_base.join(&hash);
        let client = awc::Client::default();
        dbg!(url.display());
        let req = client
            .get(url.to_str().unwrap())
            .insert_header(("user-agent", r#"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"#))
            .timeout(Duration::from_secs(30));
        let mut res = req.send().await.map_err(|e| Error::FetchError(e))?;
        eprintln!("get response");
        let img = res.body().await.unwrap();
        eprintln!("get body");

        let dir = self.dir.write()?;
        let path = dir.join(hash + ".jpg");

        let mut f = std::fs::File::create(&path).expect("creating file");
        f.write_all(&img).unwrap();
        drop(f);

        dbg!(&dir);
        drop(dir);

        let file = NamedFile::open(path.path().to_path_buf()).expect("opening named file");
        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_it() {
        let dir = tempfile::TempDir::new().unwrap();
        // let db = DefaultDB::new(dir.path(), "http://cn.gravatar.com/avatar/".into());
        let db = DefaultDB::new(dir.path(), "http://sdn.geekzu.org/avatar/".into());
        let _ = dbg!(db.fetch(&EmailAddress::new("jy.cat@qq.com").unwrap()).await);
        drop(dir);
    }
}