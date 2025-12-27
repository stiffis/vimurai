mod app;
pub mod ui;

use crate::utils::Result;
pub use app::App;

pub fn run() -> Result<()> {
    let mut app = App::new()?;
    app.run()
}
