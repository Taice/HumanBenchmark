mod reaction_time;

use std::{io, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Styled},
    symbols::border,
    widgets::{Block, Paragraph, Widget},
};

const DIR_NAME: &str = "HumanBenchmark";

#[derive(Default)]
pub struct Menu {
    exit: bool,
    index: i8,
}

impl Menu {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_input(terminal)?;
            continue;
        }

        Ok(())
    }

    fn handle_input(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if event::poll(Duration::from_millis(1))? {
            let event = event::read()?;
            if let event::Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Enter => self.go(terminal)?,
                    KeyCode::Right => self.increase(),
                    KeyCode::Left => self.decrease(),
                    KeyCode::Up => self.up(),
                    KeyCode::Down => self.down(),
                    _ => (),
                }
            }
        }
        Ok(())
    }

    fn go(&self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        match self.index {
            0 => reaction_time::ReactionTime::run(terminal)?,
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
