mod app;

use std::io;

use app::Menu;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = Menu::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
