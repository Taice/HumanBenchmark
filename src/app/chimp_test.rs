mod mode;

use mode::Mode;
use rand::{Rng, rng};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode, KeyEvent, MouseEvent, MouseEventKind},
    layout::{Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Color, Style, Styled, Stylize},
    symbols::{Marker, border},
    text::Span,
    widgets::{Block, Dataset, GraphType, Paragraph, Widget},
};
use std::time::Duration;

use super::{Filed, Game, render_graph, savestate::SaveState};

const FILE_NAME: &str = "ChimpTest";
const HEIGHT: u16 = 5 * TARGET_SIZE;
const WIDTH: u16 = 8 * TARGET_SIZE * 2;
const TARGET_SIZE: u16 = 3;
const LIVES: u32 = 0;
const DEFAULT_NUMBERS: u32 = 4;

#[derive(Debug, Clone)]
pub struct ChimpTest {
    exit: bool,
    lives: u32,
    current_number: usize,
    numbers: u32,

    target_vec: Vec<Position>,
    savestate: SaveState,
    mode: Mode,
}

impl Default for ChimpTest {
    fn default() -> Self {
        Self {
            lives: LIVES,
            numbers: DEFAULT_NUMBERS,
            current_number: 0,
            exit: false,
            target_vec: Vec::default(),
            savestate: SaveState::default(),
            mode: Mode::default(),
        }
    }
}

impl ChimpTest {
    fn reset(&mut self) {
        let new = Self {
            savestate: self.savestate,
            ..Default::default()
        };
        *self = new;
    }
    fn key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Char('r') => self.reset(),
            _ => (),
        }
    }
    fn mouse_event(&mut self, mouse: MouseEvent, terminal: &mut DefaultTerminal) {
        match mouse.kind {
            MouseEventKind::Down(event::MouseButton::Left) => (),
            _ => return,
        }

        match self.mode {
            Mode::Waiting => {
                self.mode = Mode::Playing;
                self.populate_vec();
            }
            Mode::Playing => {
                let main = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(0)])
                    .split(terminal.get_frame().area())[1]
                    .inner(Margin {
                        horizontal: 1,
                        vertical: 1,
                    });

                let pf_vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(HEIGHT),
                        Constraint::Min(0),
                    ])
                    .split(main)[1];

                let pf = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(WIDTH),
                        Constraint::Min(0),
                    ])
                    .split(pf_vert)[1];

                let mouse_rect = Rect::new(mouse.column, mouse.row, 1, 1);

                for i in self.current_number..self.target_vec.len() {
                    let pos = self.target_vec[i];
                    let target_vert = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(pos.y * TARGET_SIZE),
                            Constraint::Length(TARGET_SIZE),
                        ])
                        .split(pf)[1];

                    let target = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(pos.x * TARGET_SIZE * 2),
                            Constraint::Length(TARGET_SIZE * 2),
                            Constraint::Min(0),
                        ])
                        .split(target_vert)[1];

                    if mouse_rect.intersects(target) {
                        if i == self.current_number {
                            self.current_number += 1;
                            if self.current_number == self.target_vec.len() {
                                self.current_number = 0;
                                self.numbers += 1;
                                self.target_vec.clear();
                                self.populate_vec();
                            }
                        } else {
                            self.lose_life();
                        }
                    }
                }
            }
            Mode::Results => {
                self.reset();
            }
        }
    }
    fn populate_vec(&mut self) {
        let mut bools: [[bool; WIDTH as usize]; HEIGHT as usize] =
            [[false; WIDTH as usize]; HEIGHT as usize];

        let mut rng = rng();
        for _ in 1..=self.numbers {
            loop {
                let pos: Position = Position {
                    x: rng.random_range(0..8),
                    y: rng.random_range(0..5),
                };

                if !bools[pos.y as usize][pos.x as usize] {
                    bools[pos.y as usize][pos.x as usize] = true;
                    self.target_vec.push(pos);
                    break;
                }
            }
        }
    }
    fn lose_life(&mut self) {
        if let Some(val) = self.lives.checked_sub(1) {
            self.lives = val;
            self.current_number = 0;
            self.target_vec.clear();
            self.populate_vec();
        } else {
            self.mode = Mode::Results;
            self.savestate.update(self.numbers as f32);
        }
    }
}

impl Game for ChimpTest {
    fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        let mut game = Self::load().unwrap_or_default();

        while !game.exit {
            terminal.draw(|frame| game.draw(frame))?;
            game.handle_input(terminal)?;
        }

        game.save();
        Ok(())
    }

    fn handle_input(&mut self, terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        if event::poll(Duration::MAX)? {
            match event::read()? {
                event::Event::Key(key) => self.key_event(key),
                event::Event::Mouse(mouse) => self.mouse_event(mouse, terminal),
                _ => (),
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Filed<'_> for ChimpTest {
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

impl Widget for &ChimpTest {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        Paragraph::new(Span::from("Chimp Test").fg(Color::Red))
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
                block.title("╡ Playing ╞").render(vert[1], buf);

                let pf_vert = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(HEIGHT),
                        Constraint::Min(0),
                    ])
                    .split(main)[1];

                let pf = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(WIDTH),
                        Constraint::Min(0),
                    ])
                    .split(pf_vert)[1];

                // Block::bordered()
                //     .border_set(border::ROUNDED)
                //     .title(" Playfield ")
                //     .render(pf_outer, buf);

                for i in self.current_number..self.target_vec.len() {
                    let pos = self.target_vec[i];
                    let target_vert = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(pos.y * TARGET_SIZE),
                            Constraint::Length(TARGET_SIZE),
                            Constraint::Min(0),
                        ])
                        .split(pf)[1];

                    let target = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(pos.x * TARGET_SIZE * 2),
                            Constraint::Length(TARGET_SIZE * 2),
                            Constraint::Min(0),
                        ])
                        .split(target_vert)[1];

                    let pg = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(1),
                            Constraint::Min(0),
                        ])
                        .split(target.inner(Margin {
                            horizontal: 1,
                            vertical: 1,
                        }))[1];

                    if self.current_number > 0 {
                        Block::bordered()
                            .border_set(border::ROUNDED)
                            .set_style(Style::default().bg(Color::White))
                            .render(target, buf);
                    } else {
                        Block::bordered()
                            .border_set(border::ROUNDED)
                            .set_style(Color::White)
                            .render(target, buf);
                    }

                    Paragraph::new(num_to_string(i as u32 + 1))
                        .set_style(Color::White)
                        .centered()
                        .render(pg, buf);
                }
            }
            Mode::Results => {
                block.title("╡ Results ╞").render(vert[1], buf);

                let dataset = Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .cyan()
                    .data(&[
                        (4.0, (20.0 / 280.0)),
                        (5.0, (14.0 / 280.0)),
                        (6.0, (18.0 / 280.0)),
                        (7.0, (42.0 / 280.0)),
                        (8.0, (125.0 / 280.0)),
                        (9.0, (240.0 / 280.0)),
                        (10.0, (254.0 / 280.0)),
                        (11.0, (219.0 / 280.0)),
                        (12.0, (121.0 / 280.0)),
                        (13.0, (59.0 / 280.0)),
                        (14.0, (35.0 / 280.0)),
                        (15.0, (20.0 / 280.0)),
                        (16.0, (15.0 / 280.0)),
                        (17.0, (10.0 / 280.0)),
                        (18.0, (10.0 / 280.0)),
                        (19.0, (5.0 / 280.0)),
                        (20.0, (3.0 / 280.0)),
                        (21.0, (2.0 / 280.0)),
                        (22.0, (1.0 / 280.0)),
                        (23.0, (1.0 / 280.0)),
                        (24.0, (1.0 / 280.0)),
                    ]);

                render_graph(
                    self.savestate.avg_score as f64,
                    self.numbers as f64,
                    dataset,
                    [4.0, 24.0],
                    main,
                    buf,
                );
            }
        }
    }
}

fn num_to_string(num: u32) -> String {
    if num < 10 {
        format!("0{}", num)
    } else {
        num.to_string()
    }
}
