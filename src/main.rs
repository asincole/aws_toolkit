use app::App;
use color_eyre::Result;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};
use tui_logger::{LevelFilter, init_logger, set_default_level};

mod app;
mod aws;
mod search;
#[cfg(test)]
mod search_test;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tui_logger::TuiTracingSubscriberLayer)
        .init();
    init_logger(LevelFilter::Trace)?;
    set_default_level(LevelFilter::Info);

    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let app_result = App::new().await?.run(&mut terminal).await;
    ratatui::restore();

    app_result
}
