mod mode;
mod texts;

use mode::Mode;
use rand::{Rng, rng};
use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode, KeyEvent, MouseEventKind},
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Stylize},
    symbols::{Marker, border},
    text::{Line, Span},
    widgets::{Block, Dataset, GraphType, Paragraph, Widget},
};
use std::time::{Duration, Instant};
use texts::TEXTS;

use super::{Filed, Game, render_graph, savestate::SaveState};

const FILE_NAME: &str = "TypingTest";

#[derive(Default, Debug, Clone)]
pub struct TypingTest {
    exit: bool,

    wpm: Option<f32>,
    instant: Option<Instant>,
    text: String,
    text_index: usize,
    savestate: SaveState,
    mode: Mode,
}

impl TypingTest {
    fn reset(&mut self) {
        let new = Self {
            savestate: self.savestate,
            ..Default::default()
        };
        *self = new;
    }
    fn key_event(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Waiting | Mode::Results => match key.code {
                KeyCode::Char(' ') | KeyCode::Enter => self.play(),
                KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
                KeyCode::Char('r') => self.reset(),
                _ => (),
            },
            Mode::Playing => match key.code {
                KeyCode::Esc => self.exit = true,
                KeyCode::Enter => self.go(),
                KeyCode::Char(c) => self.add_ch(c),
                KeyCode::Backspace => {
                    let _ = self.text.pop();
                }
                _ => (),
            },
        }
    }

    fn go(&mut self) {
        if self.text.len() == TEXTS[self.text_index].len() {
            self.results();
        }
    }
    fn add_ch(&mut self, c: char) {
        if self.text.is_empty() {
            self.instant = Some(Instant::now());
        }
        if self.text.len() < TEXTS[self.text_index].len() {
            let last = self
                .text
                .chars()
                .last()
                .unwrap_or(/*because if none dont put space*/ ' ');
            if c == ' ' && last == ' ' {
                return;
            }
            self.text += &c.to_string();
        }
        if self.text.len() == TEXTS[self.text_index].len() {
            self.results();
        }
    }

    fn play(&mut self) {
        self.mode = Mode::Playing;
        self.text_index = rng().random_range(0..TEXTS.len());
        self.text = String::new();
        self.instant = Some(Instant::now());
    }

    fn get_wpm(&self) -> Option<f32> {
        Some(
            (self.text.len() as f32 * self.get_acc() / 5.0)
                / (self.instant?.elapsed().as_millis() as f32 / 1000.0)
                * 60.0,
        )
    }

    fn get_acc(&self) -> f32 {
        let total = self.text.len();
        let matches = self
            .text
            .chars()
            .zip(TEXTS[self.text_index].chars())
            .filter(|(c1, c2)| c1 == c2)
            .count();

        matches as f32 / total as f32
    }

    fn get_text(&self) -> Line {
        let mut text = Line::default();
        let mut iterator = self.text.chars();

        for rc in TEXTS[self.text_index].chars() {
            if let Some(uc) = iterator.next() {
                if rc == uc {
                    text += rc.to_string().on_green().black()
                } else {
                    text += rc.to_string().on_red()
                }
            } else {
                text += Span::raw(rc.to_string());
            }
        }

        text
    }

    fn results(&mut self) {
        self.mode = Mode::Results;
        self.wpm = self.get_wpm();
        self.savestate.update(self.wpm.unwrap());
    }
}

impl Game for TypingTest {
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
        if event::poll(Duration::MAX)? {
            match event::read()? {
                event::Event::Key(key) => self.key_event(key),
                event::Event::Mouse(mouse) => {
                    if let Mode::Playing = self.mode {
                    } else if let MouseEventKind::Down(_) = mouse.kind {
                        self.play();
                    }
                }
                _ => (),
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Filed<'_> for TypingTest {
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

impl Widget for &TypingTest {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        Paragraph::new(Span::from("Typing Test").fg(Color::Red))
            .centered()
            .block(Block::bordered().border_set(border::DOUBLE))
            .render(vert[0], buf);

        let block = Block::bordered().border_set(border::DOUBLE);

        let main = vert[1].inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        match self.mode {
            Mode::Waiting => {
                block.title("╡ Menu ╞").render(vert[1], buf);

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(main);

                Paragraph::new("Click to start the game")
                    .centered()
                    .render(layout[1], buf);
            }
            Mode::Playing => {
                block.title("╡ Typing ╞").render(vert[1], buf);
                let text = self.get_text();

                Paragraph::new(text)
                    .wrap(ratatui::widgets::Wrap { trim: true })
                    .render(main, buf);
            }
            Mode::Results => {
                block.title("╡ Results ╞").render(vert[1], buf);

                let dataset = Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .cyan()
                    .data(&[
                        (0.0, (30.0 / 300.0)),
                        (10.0, (70.0 / 300.0)),
                        (20.0, (160.0 / 300.0)),
                        (30.0, (233.0 / 300.0)),
                        (40.0, (275.0 / 300.0)),
                        (50.0, (247.0 / 300.0)),
                        (60.0, (213.0 / 300.0)),
                        (70.0, (160.0 / 300.0)),
                        (80.0, (131.0 / 300.0)),
                        (90.0, (75.0 / 300.0)),
                        (100.0, (57.0 / 300.0)),
                        (110.0, (27.0 / 300.0)),
                        (120.0, (17.0 / 300.0)),
                        (130.0, (5.0 / 300.0)),
                        (140.0, (0.0 / 300.0)),
                        (150.0, (0.0 / 300.0)),
                        (160.0, (0.0 / 300.0)),
                        (170.0, (0.0 / 300.0)),
                        (180.0, (0.0 / 300.0)),
                        (190.0, (0.0 / 300.0)),
                        (200.0, (0.0 / 300.0)),
                    ]);

                render_graph(
                    self.savestate.avg_score as f64,
                    self.wpm.unwrap() as f64,
                    dataset,
                    [0.0, 200.0],
                    main,
                    buf,
                );
            }
        }
    }
}
