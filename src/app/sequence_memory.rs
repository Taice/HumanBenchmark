use super::{DIR_NAME, Filed, Game};

use chrono::{DateTime, Local};
use directories::BaseDirs;
use rand::{Rng, rng};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    time::{Duration, Instant, SystemTime},
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode, MouseEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Styled},
    symbols::border,
    widgets::{Block, Paragraph, Widget},
};

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "SequenceMemory.json";
// when should the square fade out in millis
const FADE_OUT: u64 = 500;

#[derive(Default)]
pub struct SequenceMemory {
    exit: bool,

    clicked: Option<(u8, Instant)>,
    scores: Vec<u32>,
    savestate: SMSaveState,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SMSaveState {
    avg_score: f32,
    num_entries: u32,
}

impl Game for SequenceMemory {
    fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut game = Self::load().unwrap_or_default();

        while !game.exit {
            terminal.draw(|frame| game.draw(frame))?;
            game.handle_input(terminal)?;
        }

        game.save();
        Ok(())
    }

    fn handle_input(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if event::poll(Duration::from_secs(10))? {
            let event = event::read()?;
            match event {
                event::Event::Mouse(e) => {
                    if let MouseEventKind::Down(_) = e.kind {
                        let mouse_rect = Rect::new(e.column, e.row, 1, 1);
                        let area = terminal.get_frame().area();

                        let vert = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Length(3), // Title
                                Constraint::Min(0),    // Body
                            ])
                            .split(area);

                        let main = vert[1].inner(ratatui::layout::Margin {
                            horizontal: 1,
                            vertical: 1,
                        });

                        let rows = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Min(0),    // ---
                                Constraint::Length(4), // Title
                                Constraint::Length(4), // Title
                                Constraint::Length(4), // Title
                                Constraint::Min(0),    // ---
                            ])
                            .split(main);

                        let lenth = rows[1].height * 2;

                        let rects = [
                            Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints([
                                    Constraint::Min(0),
                                    Constraint::Length(lenth),
                                    Constraint::Length(lenth),
                                    Constraint::Length(lenth),
                                    Constraint::Min(0),
                                ])
                                .split(rows[1]),
                            Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints([
                                    Constraint::Min(0),
                                    Constraint::Length(lenth),
                                    Constraint::Length(lenth),
                                    Constraint::Length(lenth),
                                    Constraint::Min(0),
                                ])
                                .split(rows[2]),
                            Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints([
                                    Constraint::Min(0),
                                    Constraint::Length(lenth),
                                    Constraint::Length(lenth),
                                    Constraint::Length(lenth),
                                    Constraint::Min(0),
                                ])
                                .split(rows[3]),
                        ];

                        let mut index = 0;
                        'outer: for y in 0..3 {
                            for x in 1..4 {
                                if mouse_rect.intersects(rects[y][x]) {
                                    self.clicked = Some((index, Instant::now()));
                                    break 'outer;
                                }
                                index += 1;
                            }
                        }
                    }
                }
                event::Event::Key(key) => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                    _ => (),
                },
                _ => (),
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Filed<'_> for SequenceMemory {
    type SaveState = SMSaveState;

    fn get_save_file() -> Option<String> {
        let dirs = BaseDirs::new()?;
        let dir = dirs.data_dir();
        Some(
            dir.join(format!("{DIR_NAME}/{FILE_NAME}"))
                .to_str()?
                .to_owned(),
        )
    }

    fn get_dir() -> Option<String> {
        let dirs = BaseDirs::new()?;
        let dir = dirs.data_dir();
        Some(dir.join(DIR_NAME).to_str()?.to_owned())
    }

    fn from_savestate(savestate: Self::SaveState) -> Self {
        Self {
            savestate,
            ..Default::default()
        }
    }

    fn get_savestate(&self) -> Self::SaveState {
        SMSaveState {
            avg_score: self.get_avg_score(),
            num_entries: self.savestate.num_entries + self.scores.len() as u32,
        }
    }
}

impl SequenceMemory {
    // lol what is this shit
    fn get_avg_score(&self) -> f32 {
        (self.savestate.avg_score * self.savestate.num_entries as f32
            + self.scores.iter().fold(0.0f32, |acc, x| acc + *x as f32))
            / (self.savestate.num_entries as f32 + self.scores.len() as f32)
    }
}

impl Widget for &SequenceMemory {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Body
            ])
            .split(area);

        let main = vert[1].inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });

        Block::bordered()
            .border_set(border::DOUBLE)
            .render(vert[1], buf);

        let mut clicked = -1;
        if let Some((i, instant)) = self.clicked {
            if (instant.elapsed().as_millis() as u64) < FADE_OUT {
                clicked = i as i8;
            }
        }

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // ---
                Constraint::Length(4), // Title
                Constraint::Length(4), // Title
                Constraint::Length(4), // Title
                Constraint::Min(0),    // ---
            ])
            .split(main);

        let lenth = rows[1].height * 2;

        let rects = [
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(lenth),
                    Constraint::Length(lenth),
                    Constraint::Length(lenth),
                    Constraint::Min(0),
                ])
                .split(rows[1]),
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(lenth),
                    Constraint::Length(lenth),
                    Constraint::Length(lenth),
                    Constraint::Min(0),
                ])
                .split(rows[2]),
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(lenth),
                    Constraint::Length(lenth),
                    Constraint::Length(lenth),
                    Constraint::Min(0),
                ])
                .split(rows[3]),
        ];

        let mut index = 0;
        for y in rects {
            for x in 1..4 {
                if index == clicked {
                    let inner = y[x].inner(ratatui::layout::Margin {
                        horizontal: 1,
                        vertical: 1,
                    });
                    Block::bordered()
                        .border_set(border::QUADRANT_INSIDE)
                        .set_style(Style::default().fg(Color::White))
                        .render(y[x], buf);
                    Block::new()
                        .set_style(Style::default().bg(Color::White))
                        .render(inner, buf);
                } else {
                    Block::bordered()
                        .border_set(border::THICK)
                        .render(y[x], buf);
                }
                index += 1;
            }
        }
    }
}
