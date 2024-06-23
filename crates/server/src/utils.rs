//! Development utilities

use actix_web::web::{self, Data};
use problem::sample::{a_plus_b_data, a_plus_b_statment};
use std::sync::Arc;

use crate::data::{
    databases::{
        problem_ojdata::{self, OJDataDB},
        problem_statement, user,
    },
    file_system::FileSysDb,
    mysql::MysqlDb,
    types::*,
};
use crate::web::rev_proxy::RevProxy;

/// 将非 `/api` 开头的请求转发到 `base_url`
pub fn frontend_rev_proxy(base_url: String) -> RevProxy {
    RevProxy::create(base_url).path_trans(|s| {
        if s.starts_with("/api") {
            None
        } else {
            // forward to front-end server
            Some(s.to_string())
        }
    })
}

/// 存储在文件中的用户数据库
///
/// 预先插入用户名 `testtest`，密码 `testtest` 的用户
pub fn test_userdb(mysqldb: &MysqlDb) -> Data<user::UserDB> {
    let db = user::UserDB::new(mysqldb);
    let r = web::Data::new(db);
    // 预先插入一个用户方便测试
    if r.query_by_username(&Username::new("testtest").unwrap())
        .is_err()
    {
        let user = r
            .new_user(
                &Username::new("testtest").unwrap(),
                &passwd::register_hash("testtest"),
                &EmailAddress::new("test@test.com").unwrap(),
            )
            .unwrap();
        tracing::info!(?user, "user 'testtset' added");
    } else {
        tracing::info!("user 'testtset' already exists");
    }
    r
}

/// 用于测试的题面数据库
///
/// 预先插入若干个 A + B problem 的题面
pub fn test_stmtdb(
    mysqldb: &MysqlDb,
    filesysdb: &FileSysDb,
) -> web::Data<problem_statement::StmtDB> {
    let stmt_db = problem_statement::StmtDB::new(mysqldb, filesysdb);
    if stmt_db.get(1).is_err() {
        let id = stmt_db
            .insert_new(a_plus_b_statment())
            .expect("fail to insert A + B Problem");
        assert!(id == 1);
    }
    tracing::info!("test statement db initialized");
    web::Data::new(stmt_db)
}

pub fn test_ojdata_db(filesysdb: &FileSysDb) -> web::Data<OJDataDB> {
    let db = web::Data::new(problem_ojdata::DefaultDB::new(filesysdb));

    db.insert(1, a_plus_b_data())
        .expect("fail to insert A + B Problem data");

    db
}

/// logging configuration for development
pub fn logging_setup(max_level: &'static tracing::Level, log_file: Option<String>) {
    use tracing_subscriber::{
        filter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
    };

    let terminal_log = tracing_subscriber::fmt::layer()
        .pretty()
        .with_thread_names(true)
        .with_filter(filter::filter_fn(|meta| {
            // let is_invalid_identity = meta
            //     .module_path()
            //     .is_some_and(|s| s.contains("actix_session::session"));

            meta.level() <= max_level // && !from_actix_session
        }));

    let file_log = log_file
        .and_then(|log_file| std::fs::File::create(log_file).ok())
        .map(|file| {
            let file = std::sync::Mutex::new(Arc::new(file));
            tracing_subscriber::fmt::layer()
                .json()
                .with_thread_names(true)
                .with_writer(move || file.lock().unwrap().clone())
                .with_filter(filter::filter_fn(|meta| {
                    // the smaller, the more prior
                    meta.level() <= max_level &&
            // too annoying to verbose
            !meta
                .module_path()
                .is_some_and(|s| s.contains("actix_session::session"))
                }))
        });
    tracing_subscriber::registry()
        .with(file_log)
        .with(terminal_log)
        .init();
}

use rustls::{ClientConfig, RootCertStore};

/// Create simple rustls client config from root certificates.
pub fn rustls_config() -> ClientConfig {
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}

/// Convenient shortcut for [`actix_web::web::block`], which executes blocking
/// function on a thread pool, returns future that resolves to result
/// of the function execution.
#[macro_export]
macro_rules! block_it {
    ($s:expr) => {
        actix_web::web::block(move || $s).await?
    };
}

pub mod rating {
    pub trait RatingChanger {
        fn rating(&mut self) -> &mut i32;
    }

    fn calc_percentage(p1: i32, p2: i32) -> f64 {
        1f64 / (1f64 + 10f64.powf(((p2 - p1) as f64) / (400f64)))
    }

    fn calc<T: RatingChanger>(contestants: &mut [T], pi: i32, i: usize) -> f64 {
        let mut sum: f64 = 0.0;
        for (j, c) in contestants.iter_mut().enumerate() {
            if i == j {
                continue;
            } else {
                sum += calc_percentage(*c.rating(), pi);
            }
        }
        sum
    }

    /// k 表示变化力度（得分变化为默认值的 k 倍）
    /// 假设 rank = 下标+1
    /// minrating = -10000 ,maxrating = 10000

    pub fn modify_rating<T: RatingChanger>(contestants: &mut [T], k: f64) {
        let mut seed: Vec<f64> = Vec::new();
        for i in 0..contestants.len() {
            let num = *contestants[i].rating();
            let mut sum: f64 = calc(contestants, num, i);
            sum += 1f64;
            seed.push(sum);
        }
        let mut dir: Vec<i32> = Vec::new();
        let mut totdir = 0;
        for i in 0..contestants.len() {
            let mi = (seed[i] * ((i + 1) as f64)).sqrt();
            let mut l: i32 = -10000;
            let mut r: i32 = 10000;
            while r > l + 1 {
                let mid = (l + r) / 2;
                //println!("{} {}",mid,calc(contestants,mid,i)+1.0);
                if calc(contestants, mid, i) + 1.0 >= mi {
                    l = mid;
                } else {
                    r = mid;
                }
            }
            let nowdir = (l - *contestants[i].rating()) / 2;
            //println!("{} {} {}",seed[i],mi,nowdir);
            dir.push(nowdir);
            totdir += nowdir;
        }
        let inc = (0f64).min((-10f64).max(-(totdir as f64) / (contestants.len() as f64)));
        for i in 0..contestants.len() {
            *contestants[i].rating() += (((dir[i] as f64) + inc) * k) as i32;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            struct Users {
                rate: i32,
            }
            impl RatingChanger for Users {
                fn rating(&mut self) -> &mut i32 {
                    &mut self.rate
                }
            }
            let mut users: Vec<Users> = Vec::new();
            for i in 0..50 {
                let person = Users {
                    rate: 2500 - i * 10,
                };
                users.push(person);
            }
            for u in &users {
                println!("{}", u.rate);
            }
            modify_rating(&mut users, 1.0);
            for u in &users {
                println!("{}", u.rate);
            }
        }
    }
}
