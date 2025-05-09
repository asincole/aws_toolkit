use app::App;
use color_eyre::Result;

mod app;
mod aws;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let app_result = App::new().await?.run(&mut terminal);
    ratatui::restore();

    app_result
}
