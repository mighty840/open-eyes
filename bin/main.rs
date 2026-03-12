#![allow(non_snake_case)]

#[allow(clippy::expect_used)]
fn main() {
    #[cfg(feature = "web")]
    {
        dioxus_logger::init(tracing::Level::DEBUG).expect("Failed to init logger");
        dioxus::web::launch::launch_cfg(
            open_eyes_dashboard::App,
            dioxus::web::Config::new().hydrate(true),
        );
    }

    #[cfg(feature = "server")]
    {
        dotenvy::dotenv().ok();
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .init();

        open_eyes_dashboard::infrastructure::server_start(open_eyes_dashboard::App)
            .map_err(|e| {
                tracing::error!("Unable to start server: {e}");
            })
            .expect("Server start failed")
    }
}
