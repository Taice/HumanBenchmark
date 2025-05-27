use super::DIR_NAME;

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
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Styled},
    symbols::border,
    widgets::{Block, Paragraph, Widget},
};

use serde::{Deserialize, Serialize};

#[derive(Default)]
enum Mode {
    #[default]
    Waiting,
    TooEarly,
    Clicking,
    TimeOut,
    Result,
}

#[derive(Default)]
pub struct ReactionTime {
    exit: bool,
    curr: Option<SystemTime>,
    times: Vec<Duration>,
    savestate: SaveState,
    mode: Mode,
}

#[derive(Default, Serialize, Deserialize)]
struct SaveState {
    avg_time: u64,
    num_entries: u32,
}

impl ReactionTime {
    pub fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut game = Self::load().unwrap_or_default();

        while !game.exit {
            terminal.draw(|frame| game.draw(frame))?;
            game.handle_input()?;
        }

        game.save();
        Ok(())
    }

    fn handle_input(&mut self) -> io::Result<()> {
        match self.mode {
            Mode::Waiting => {
                self.waiting_input()?;
            }
            Mode::Clicking => {
                if event::poll(Duration::from_secs(10))? {
                    let event = event::read()?;
                    match event {
                        event::Event::Key(key) => match key.code {
                            KeyCode::Char('q') => self.exit = true,
                            _ => {
                                self.times.push(self.curr.unwrap().elapsed().unwrap());
                                self.mode = Mode::Result;
                            }
                        },
                        event::Event::Mouse(mouse) => {
                            if let MouseEventKind::Down(_) = mouse.kind {
                                self.times.push(self.curr.unwrap().elapsed().unwrap());
                                self.mode = Mode::Result;
                            }
                        }
                        _ => (),
                    }
                } else {
                    self.mode = Mode::TimeOut;
                }
            }
            Mode::Result | Mode::TimeOut | Mode::TooEarly => {
                if event::poll(Duration::MAX)? {
                    let event = event::read()?;
                    match event {
                        event::Event::Key(key) => match key.code {
                            KeyCode::Char('q') => self.exit = true,
                            KeyCode::Char('r') => self.mode = Mode::Waiting,
                            _ => (),
                        },
                        event::Event::Mouse(mouse) => {
                            if let MouseEventKind::Down(_) = mouse.kind {
                                self.mode = Mode::Waiting;
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
        Ok(())
    }

    fn waiting_input(&mut self) -> io::Result<()> {
        let start = Instant::now();
        let dur = Duration::from_millis(rng().random_range(3000..6000));

        while start.elapsed() < dur {
            let remaining = dur.checked_sub(start.elapsed()).unwrap_or(Duration::ZERO);
            if event::poll(remaining)? {
                let event = event::read()?;
                match event {
                    event::Event::Key(key) => {
                        if let KeyCode::Char('q') = key.code {
                            self.exit = true;
                        } else {
                            self.mode = Mode::TooEarly;
                        }
                        return Ok(());
                    }
                    event::Event::Mouse(mouse) => {
                        if let MouseEventKind::Down(_) = mouse.kind {
                            self.mode = Mode::TooEarly;
                            return Ok(());
                        }
                    }
                    _ => {} // Ignore other events and continue waiting
                }
            }
        }

        self.mode = Mode::Clicking;
        self.curr = Some(SystemTime::now());
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn load() -> Option<Self> {
        let file = get_save_file()?;
        match std::fs::read_to_string(&file) {
            Ok(contents) => {
                let thing: serde_json::Result<SaveState> = serde_json::from_str(&contents);
                match thing {
                    Ok(savestate) => {
                        return Some(Self {
                            savestate,
                            ..Default::default()
                        });
                    }
                    Err(err) => write_log(format!("{err}")),
                }
            }
            Err(err) => write_log(format!("{err}")),
        }
        None
    }

    fn save(&mut self) {
        if let Some(file) = get_save_file() {
            let savestate = SaveState {
                avg_time: self.get_avg_time(),
                num_entries: self.savestate.num_entries + self.times.len() as u32,
            };

            if let Ok(json) = serde_json::to_string(&savestate) {
                if let Err(e) = fs::create_dir_all(get_dir().unwrap()) {
                    write_log(e.to_string() + "bnanaa");
                }
                match fs::write(file, json) {
                    Ok(_) => write_log(String::from("Successuly saved.")),
                    Err(e) => write_log(e.to_string() + "bnanaa"),
                }
            }
        }
    }

    // lol what is this shit
    fn get_avg_time(&self) -> u64 {
        (self.savestate.avg_time * self.savestate.num_entries as u64
            + self
                .times
                .iter()
                .fold(0u64, |acc, x| acc + x.as_millis() as u64))
            / (self.savestate.num_entries as u64 + self.times.len() as u64)
    }
}

impl Widget for &ReactionTime {
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
        let center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(main);

        Paragraph::new("Reaction Time Test")
            .set_style(Color::Blue)
            .centered()
            .block(Block::bordered().border_set(border::DOUBLE))
            .render(vert[0], buf);

        Block::bordered()
            .border_set(border::DOUBLE)
            .render(vert[1], buf);

        match self.mode {
            Mode::Waiting => {
                Block::new()
                    .style(Style::default().bg(Color::Red))
                    .render(main, buf);
                Paragraph::new("Waiting...")
                    .centered()
                    .set_style(Color::Black)
                    .render(center[1], buf);

                Paragraph::new("'q' to quit")
                    .centered()
                    .set_style(Color::Black)
                    .render(center[4], buf);
            }
            Mode::TooEarly => {
                Block::new()
                    .style(Style::default().bg(Color::DarkGray))
                    .render(main, buf);
                Paragraph::new("Too early you loser fuck you early clicker dumbass")
                    .centered()
                    .render(center[1], buf);

                Paragraph::new("'r' to restart and 'q' to quit")
                    .centered()
                    .render(center[4], buf);
            }
            Mode::Clicking => {
                Block::new()
                    .style(Style::default().bg(Color::Green))
                    .render(main, buf);
                Paragraph::new("CLICK NOW FAST OR ELSE YOU'LL DIE NOW CLICK FAST")
                    .centered()
                    .set_style(Color::Black)
                    .render(center[1], buf);

                Paragraph::new("'q' to quit")
                    .centered()
                    .set_style(Color::Black)
                    .render(center[4], buf);
            }
            Mode::TimeOut => {
                Paragraph::new("You're so slow I literally timed out.")
                    .centered()
                    .render(center[1], buf);

                Paragraph::new("'r' to restart and 'q' to quit")
                    .centered()
                    .render(center[4], buf);
            }
            Mode::Result => {
                Paragraph::new(format!(
                    "Your time was: {}ms",
                    self.times.last().unwrap().as_millis()
                ))
                .centered()
                .render(center[1], buf);
                Paragraph::new(format!(
                    "Avg. time across the board: {}ms",
                    self.get_avg_time()
                ))
                .centered()
                .render(center[3], buf);

                Paragraph::new("'r' to restart and 'q' to quit")
                    .centered()
                    .render(center[4], buf);
            }
        }
    }
}

fn get_save_file() -> Option<String> {
    let dirs = BaseDirs::new()?;
    let dir = dirs.data_dir();
    Some(
        dir.join(format!("{DIR_NAME}/ReactionTime.json"))
            .to_str()?
            .to_owned(),
    )
}

fn get_dir() -> Option<String> {
    let dirs = BaseDirs::new()?;
    let dir = dirs.data_dir();
    Some(dir.join(DIR_NAME).to_str()?.to_owned())
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

fn write_log(log: String) {
    if let Some(file) = get_log_file() {
        if let Ok(data_file) = &mut OpenOptions::new().append(true).create(true).open(file) {
            let now: DateTime<Local> = Local::now();
            let log = format!(
                "[{}] ReactionTime: {}",
                now.format("%Y-%m-%d %H:%M:%S"),
                log
            );
            let _ = data_file.write(log.as_bytes());
        }
    }
}
