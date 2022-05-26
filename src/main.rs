pub mod models;
use anyhow::{
    Context,
    Result,
};
use iced::{
    pure::Application,
    Settings,
};
use tracing_subscriber::{
    fmt,
    prelude::__tracing_subscriber_SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub mod app;
pub type AppTime = chrono::NaiveDateTime;
pub fn now() -> AppTime {
    chrono::Local::now().naive_local()
}
const FS_CONCURRENCY_LIMIT: usize = 128;
pub mod db;
pub mod filesystem {
    use std::path::PathBuf;

    use super::*;
    pub fn base_directory() -> Result<PathBuf> {
        let base_dir = std::env::current_exe()
            .context("Nie znaleziono folderu w którym znajduje się aplikacja")?
            .parent()
            .context("aplikacja musi być w jakimś folderze")?
            .to_owned();
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir).context("tworzenie folderu dla aplikacji")?;
        }
        Ok(base_dir)
    }
}
fn main() -> Result<()> {
    let logs_dir = filesystem::base_directory()?.join("logs");
    let file_appender = tracing_appender::rolling::daily(&logs_dir, "log.txt");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::TRACE.into()))
        .with(fmt::Layer::new().with_writer(std::io::stdout))
        .with(
            fmt::Layer::new()
                .compact()
                .with_ansi(false)
                .with_writer(non_blocking),
        );
    tracing::subscriber::set_global_default(subscriber)
        .context("Unable to set a global subscriber")?;
    app::ArchiwumZ::run(Settings::default()).context("błąd??")?;
    Ok(())
}
