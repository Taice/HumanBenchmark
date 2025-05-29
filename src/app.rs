mod aim_trainer;
mod reaction_time;
mod sequence_memory;

use std::{
    fmt::Debug,
    fs::{self, OpenOptions},
    io::{self, Write},
    time::Duration,
};

use chrono::{DateTime, Local};
use directories::BaseDirs;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode, KeyEvent, MouseEvent, MouseEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Styled},
    symbols::border,
    widgets::{Block, Paragraph, Widget},
};

pub trait Game {
    fn run(terminal: &mut DefaultTerminal) -> io::Result<()>;
    fn handle_input(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()>;
    fn draw(&self, frame: &mut Frame);
}

pub trait Filed<'a> {
    const NAME: &'a str;
    type SaveState: serde::Deserialize<'a> + serde::Serialize + Debug;

    fn get_savestate(&self) -> Self::SaveState;
    fn from_savestate(savestate: Self::SaveState) -> Self;

    fn save(&self) {
        if let Some(file) = Self::get_save_file() {
            let savestate = self.get_savestate();
            if let Ok(json) = serde_json::to_string(&savestate) {
                match fs::create_dir_all(Self::get_dir().unwrap()) {
                    Ok(_) => (),
                    Err(e) => Self::write_log(e.to_string()),
                }
                match fs::write(file, json) {
                    Ok(_) => Self::write_log(format!("{savestate:?}")),
                    Err(e) => Self::write_log(e.to_string()),
                }
            }
        }
    }

    fn load() -> Option<Self>
    where
        Self: std::marker::Sized,
        Self::SaveState: serde::de::DeserializeOwned,
    {
        let file = Self::get_save_file()?;
        match std::fs::read_to_string(&file) {
            Ok(contents) => {
                let thing: serde_json::Result<Self::SaveState> = serde_json::from_str(&contents);
                match thing {
                    Ok(savestate) => return Some(Self::from_savestate(savestate)),
                    Err(e) => Self::write_log(e.to_string()),
                }
            }
            Err(e) => Self::write_log(e.to_string()),
        }
        None
    }

    fn get_save_file() -> Option<String> {
        let dirs = BaseDirs::new()?;
        let dir = dirs.data_dir();
        Some(
            dir.join(format!("{DIR_NAME}/{}.json", Self::NAME))
                .to_str()?
                .to_owned(),
        )
    }

    fn get_dir() -> Option<String> {
        let dirs = BaseDirs::new()?;
        let dir = dirs.data_dir();
        Some(dir.join(DIR_NAME).to_str()?.to_owned())
    }

    fn write_log(log: String) {
        if let Some(file) = get_log_file() {
            if let Ok(data_file) = &mut OpenOptions::new().append(true).create(true).open(file) {
                let now: DateTime<Local> = Local::now();
                let log = format!(
                    "[{}] {}: {}\n",
                    now.format("%Y-%m-%d %H:%M:%S"),
                    Self::NAME,
                    log
                );
                let _ = data_file.write(log.as_bytes());
            }
        }
    }
}

const DIR_NAME: &str = "HumanBenchmark";

#[derive(Default)]
pub struct Menu {
    exit: bool,
    index: i8,
}

impl Menu {
    fn key_event(&mut self, key_event: KeyEvent, terminal: &mut DefaultTerminal) -> io::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
            KeyCode::Enter => self.go(terminal)?,
            KeyCode::Right => self.increase(),
            KeyCode::Left => self.decrease(),
            KeyCode::Up => self.up(),
            KeyCode::Down => self.down(),
            _ => (),
        }
        Ok(())
    }

    fn mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        terminal: &mut DefaultTerminal,
    ) -> io::Result<()> {
        self.mouse_index(mouse_event, terminal);
        if let MouseEventKind::Down(event::MouseButton::Left) = mouse_event.kind {
            self.go(terminal)?;
        }
        Ok(())
    }

    fn mouse_index(&mut self, mouse_event: MouseEvent, terminal: &mut DefaultTerminal) {
        let area = terminal.get_frame().area();
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // ---
                Constraint::Length(5), // Top
                Constraint::Length(5), // Mid
                Constraint::Length(5), // Bot
                Constraint::Min(0),    // ---
            ])
            .split(area);

        let top = &Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),         // ---
                Constraint::Percentage(20), // Left
                Constraint::Percentage(20), // Mid
                Constraint::Percentage(20), // Right
                Constraint::Min(0),         // ---
            ])
            .split(vert[2])[1..=3];
        let mid = &Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),         // ---
                Constraint::Percentage(20), // Left
                Constraint::Percentage(20), // Mid
                Constraint::Percentage(20), // Right
                Constraint::Min(0),         // ---
            ])
            .split(vert[3])[1..=3];
        let bot = &Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),         // ---
                Constraint::Percentage(30), // Left
                Constraint::Percentage(30), // Right
                Constraint::Min(0),         // ---
            ])
            .split(vert[4])[1..=2];

        let rects = [top, mid, bot];

        let mouse_rect = Rect::new(mouse_event.column, mouse_event.row, 1, 1);

        for (i, row) in rects.iter().enumerate() {
            for (j, rect) in row.iter().enumerate() {
                if mouse_rect.intersects(*rect) {
                    self.index = (i * 3 + j) as i8;
                    return;
                }
            }
        }
        self.index = -1;
    }

    fn go(&self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        match self.index {
            0 => reaction_time::ReactionTime::run(terminal)?,
            1 => sequence_memory::SequenceMemory::run(terminal)?,
            2 => aim_trainer::AimTrainer::run(terminal)?,
            _ => (),
        }
        Ok(())
    }

    fn increase(&mut self) {
        if self.index % 3 != 2 && self.index < 7 {
            self.index += 1;
        }
    }

    fn decrease(&mut self) {
        if self.index % 3 != 0 && self.index > 0 {
            self.index -= 1;
        }
    }

    fn down(&mut self) {
        if self.index > 5 {
            return;
        }
        self.index += 3;
        if self.index > 7 {
            self.index = 7;
        }
    }

    fn up(&mut self) {
        self.index -= 3;
        if self.index < 0 {
            self.index = 0;
        } else if self.index == 4 {
            self.index = 5;
        }
    }
}

impl Game for Menu {
    fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut menu = Menu::default();

        while !menu.exit {
            terminal.draw(|frame| menu.draw(frame))?;
            menu.handle_input(terminal)?;
            continue;
        }

        Ok(())
    }

    fn handle_input(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if event::poll(Duration::MAX)? {
            match event::read()? {
                event::Event::Key(key_event) => self.key_event(key_event, terminal)?,
                event::Event::Mouse(mouse_event) => self.mouse_event(mouse_event, terminal)?,
                _ => (),
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &Menu {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // ---
                Constraint::Length(5), // Top
                Constraint::Length(5), // Mid
                Constraint::Length(5), // Bot
                Constraint::Min(0),    // ---
            ])
            .split(area);

        let top_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),         // ---
                Constraint::Percentage(20), // Left
                Constraint::Percentage(20), // Mid
                Constraint::Percentage(20), // Right
                Constraint::Min(0),         // ---
            ])
            .split(vert[2]);

        let mid_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),         // ---
                Constraint::Percentage(20), // Left
                Constraint::Percentage(20), // Mid
                Constraint::Percentage(20), // Right
                Constraint::Min(0),         // ---
            ])
            .split(vert[3]);

        let bot_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),         // ---
                Constraint::Percentage(30), // Left
                Constraint::Percentage(30), // Right
                Constraint::Min(0),         // ---
            ])
            .split(vert[4]);

        // title
        Paragraph::new("HumanBenchmark-CLI")
            .set_style(Color::Blue)
            .centered()
            .block(Block::bordered().border_set(border::DOUBLE))
            .render(vert[0], buf);

        // top row
        widget("Reaction Time", self.index == 0, top_row[1], buf);
        widget("Sequence Memory", self.index == 1, top_row[2], buf);
        widget("Aim Trainer", self.index == 2, top_row[3], buf);

        // mid row
        widget("Number memory", self.index == 3, mid_row[1], buf);
        widget("Verbal Memory", self.index == 4, mid_row[2], buf);
        widget("Chimp Test", self.index == 5, mid_row[3], buf);

        // bot row
        widget("Visual Memory", self.index == 6, bot_row[1], buf);
        widget("Typing", self.index == 7, bot_row[2], buf);
    }
}

fn widget(text: &str, color: bool, area: Rect, buf: &mut ratatui::prelude::Buffer) {
    if color {
        Paragraph::new(text)
            .set_style(Color::LightRed)
            .centered()
            .block(Block::bordered().border_set(border::THICK))
            .render(area, buf);
    } else {
        Paragraph::new(text)
            .centered()
            .block(Block::bordered().border_set(border::THICK))
            .render(area, buf);
    }
}

fn get_log_file() -> Option<String> {
    let dirs = BaseDirs::new()?;
    let dir = dirs.data_dir();
    Some(
        dir.join(format!("{DIR_NAME}/logs.txt"))
            .to_str()?
            .to_owned(),
    )
}
