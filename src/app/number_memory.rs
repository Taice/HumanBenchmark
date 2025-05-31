mod mode;
mod number;

use std::{
    cmp::Ordering,
    time::{Duration, Instant},
};

use mode::Mode;
use number::Number;
use rand::{Rng, rng};
use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Stylize},
    symbols::{Marker, border},
    text::Span,
    widgets::{Block, Dataset, GraphType, Paragraph, Widget},
};

use super::{Filed, Game, render_graph, savestate::SaveState};

const FILE_NAME: &str = "NumberMemory";
const FADE_OUT: u64 = 2000;
const ADDED_FADE: u64 = 600;

#[derive(Default, Debug, Clone)]
pub struct NumberMemory {
    exit: bool,
    mode: Mode,
    score: u32,

    number: Number,
    actual_number: Number,
    savestate: SaveState,
}

impl NumberMemory {
    fn reset(&mut self) {
        let new = Self {
            savestate: self.savestate,

            ..Default::default()
        };
        *self = new;
    }

    fn add_ch(&mut self, ch: char) {
        if (self.number.len() == self.actual_number.len()) || (self.number.is_empty() && ch == '0')
        {
            return;
        }

        self.number.push(ch);
    }

    fn new_number(&mut self) {
        self.score += 1;

        let mut rng = rng();

        self.number.clear();
        self.actual_number.clear();

        self.actual_number
            .push((rng.random_range(1..=9) + b'0') as char);

        for _ in 1..self.score {
            self.actual_number
                .push((rng.random_range(0..=9) + b'0') as char)
        }
    }

    fn process_number(&mut self) {
        if self.number.len() < self.actual_number.len() {
            return;
        }
        if self.number == self.actual_number {
            self.new_number();
            self.mode = Mode::Watching(Instant::now());
        } else {
            self.mode = Mode::Results;
            self.savestate.update(self.score as f32);
        }
    }

    fn get_dur(&self) -> u64 {
        FADE_OUT + (ADDED_FADE * (self.actual_number.len().saturating_sub(1)) as u64)
    }
}

impl Game for NumberMemory {
    fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        let mut game = Self::load().unwrap_or_default();

        while !game.exit {
            terminal.draw(|frame| game.draw(frame))?;
            game.handle_input(terminal)?;
        }

        game.save();
        Ok(())
    }

    fn handle_input(&mut self, _: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        match self.mode {
            Mode::Waiting => {
                if event::poll(Duration::MAX)? {
                    if let event::Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char(' ') | KeyCode::Enter => {
                                self.mode = Mode::Watching(Instant::now());
                                self.new_number();
                            }
                            KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                            KeyCode::Char('r') => self.reset(),
                            _ => (),
                        }
                    }
                }
            }
            Mode::Watching(instant) => {
                if instant.elapsed().as_millis() as u64 >= self.get_dur() {
                    self.mode = Mode::Playing;
                }
                if event::poll(Duration::from_millis(self.get_dur() / 10))? {
                    if let event::Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                            KeyCode::Char('r') => self.reset(),
                            KeyCode::Char(' ') => self.mode = Mode::Playing,
                            _ => (),
                        }
                    }
                }
            }
            Mode::Playing => {
                if event::poll(Duration::MAX)? {
                    if let event::Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Esc => self.exit = true,
                            KeyCode::Char(ch) => match ch {
                                'q' => self.exit = true,
                                'r' => self.reset(),
                                '0'..='9' => self.add_ch(ch),
                                _ => (),
                            },
                            KeyCode::Backspace => {
                                let _ = self.number.pop();
                            }
                            KeyCode::Enter => self.process_number(),
                            _ => (),
                        }
                    }
                }
            }
            Mode::Results => {
                if event::poll(Duration::MAX)? {
                    if let event::Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                            KeyCode::Enter | KeyCode::Char('r') => self.reset(),
                            _ => (),
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Filed<'_> for NumberMemory {
    type SaveState = SaveState;
    const NAME: &'static str = FILE_NAME;

    fn get_savestate(&self) -> Self::SaveState {
        self.savestate
    }
    fn from_savestate(savestate: Self::SaveState) -> Self {
        Self {
            savestate,
            ..Default::default()
        }
    }
}

impl Widget for &NumberMemory {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        Paragraph::new(Span::from("Number Memory Test").fg(Color::Red))
            .centered()
            .block(Block::bordered().border_set(border::DOUBLE))
            .render(vert[0], buf);

        let main = vert[1].inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        let block = Block::bordered().border_set(border::DOUBLE);

        match self.mode {
            Mode::Waiting => {
                block.title("╡ Menu ╞").render(vert[1], buf);

                let thing = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(main)[1];

                Paragraph::new("Press ENTER to start game...")
                    .centered()
                    .render(thing, buf);
            }
            Mode::Watching(instant) => {
                block.clone().title("╡ Watching.. ╞").render(vert[1], buf);

                let mut string = String::from("╡");
                let percent = ((instant.elapsed().as_millis() as f32 / (self.get_dur()) as f32)
                    * 10.0)
                    .round();
                for i in 1..=10 {
                    match percent.total_cmp(&(i as f32)) {
                        Ordering::Greater => string += "=",
                        Ordering::Equal => string += ">",
                        Ordering::Less => string += " ",
                    }
                }
                string += "╞";

                block.title("╡ Watching.. ╞").render(vert[1], buf);

                let thing = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ])
                    .split(main)[1];

                let thong = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length((self.actual_number.len() as u16 + 4).max(14)),
                        Constraint::Min(0),
                    ])
                    .split(thing)[1];

                Block::bordered()
                    .border_set(border::DOUBLE)
                    .title(string)
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .render(thong, buf);

                Paragraph::new(self.actual_number.to_string()).render(
                    thong.inner(Margin {
                        horizontal: 2,
                        vertical: 1,
                    }),
                    buf,
                );
            }
            Mode::Playing => {
                block.title("╡ Playing ╞").render(vert[1], buf);

                let thing = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ])
                    .split(main)[1];

                let thong = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length((self.actual_number.len() as u16 + 4).max(14)),
                        Constraint::Min(0),
                    ])
                    .split(thing)[1];

                Block::bordered()
                    .border_set(border::DOUBLE)
                    .title("╡ Number ╞")
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .render(thong, buf);

                Paragraph::new(self.number.to_string()).render(
                    thong.inner(Margin {
                        horizontal: 2,
                        vertical: 1,
                    }),
                    buf,
                );
            }
            Mode::Results => {
                block.title("╡ Results ╞").render(vert[1], buf);

                let dataset = Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .cyan()
                    .data(&[
                        (0.0, (0.0 / 280.0)),
                        (1.0, (18.0 / 280.0)),
                        (2.0, (5.0 / 280.0)),
                        (3.0, (5.0 / 280.0)),
                        (4.0, (7.0 / 280.0)),
                        (5.0, (15.0 / 280.0)),
                        (6.0, (42.0 / 280.0)),
                        (7.0, (112.0 / 280.0)),
                        (8.0, (206.0 / 280.0)),
                        (9.0, (255.0 / 280.0)),
                        (10.0, (200.0 / 280.0)),
                        (11.0, (135.0 / 280.0)),
                        (12.0, (65.0 / 280.0)),
                        (13.0, (35.0 / 280.0)),
                        (14.0, (20.0 / 280.0)),
                        (15.0, (10.0 / 280.0)),
                        (16.0, (7.0 / 280.0)),
                        (17.0, (5.0 / 280.0)),
                        (18.0, (4.0 / 280.0)),
                        (19.0, (3.0 / 280.0)),
                        (20.0, (2.0 / 280.0)),
                        (21.0, (1.0 / 280.0)),
                        (22.0, (0.0 / 280.0)),
                        (23.0, (0.0 / 280.0)),
                    ]);

                render_graph(
                    self.savestate.avg_score as f64,
                    self.score as f64,
                    dataset,
                    [0.0, 23.0],
                    main,
                    buf,
                );
            }
        }
    }
}
