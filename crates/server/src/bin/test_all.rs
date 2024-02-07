//! 约定：放在这里测试的服务也需要写在 gen_docs 里面

#[actix_web::main]

async fn main() -> anyhow::Result<()> {
    // logging setup
    server::utils::logging_setup(&tracing::Level::INFO, Some("runtime.log".into()));

    let app_cfg = server::ServerAppConfig::load_or_save_default(
        "local.server_app_test.json",
        server::test_server_app_cfg,
    )?;

    eprintln!("config = {app_cfg:#?}");
    let app = server::ServerApp::new(app_cfg);
    app.start().await
}
