use app::App;
use color_eyre::Result;

mod app;
mod aws;
mod list;
mod search;
#[cfg(test)]
mod search_test;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let app_result = App::new().await?.run(&mut terminal).await;
    ratatui::restore();

    app_result
}
