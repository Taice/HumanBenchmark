mod app;
use crate::app::Game;

use std::io::{self, stdout};

use app::Menu;
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;

    let mut terminal = ratatui::init(); // Your init method

    // Run the app
    let app_result = Menu::run(&mut terminal);

    // Restore terminal settings
    execute!(stdout, DisableMouseCapture)?;
    disable_raw_mode()?;
    ratatui::restore(); // Your restore method

    app_result
}
