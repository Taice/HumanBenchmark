mod mode;

use super::{Filed, Game, render_graph, savestate::SaveState};
use mode::Mode;

use rand::{Rng, rng};
use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode, KeyEvent, MouseEvent, MouseEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Styled, Stylize},
    symbols::{Marker, border},
    text::Span,
    widgets::{Block, Dataset, GraphType, Paragraph, Widget},
};

const FILE_NAME: &str = "SequenceMemory";
const FADE_OUT: u64 = 500;

pub struct SequenceMemory {
    exit: bool,

    curr: Vec<u8>,
    scramble: Vec<u8>,
    mode: Mode,
    clicked: Option<(u8, Instant)>,
    savestate: SaveState,
}

impl Default for SequenceMemory {
    fn default() -> Self {
        Self {
            exit: false,
            curr: vec![],
            scramble: vec![rng().random_range(0..9)],
            mode: Mode::Waiting,
            clicked: None,
            savestate: SaveState::default(),
        }
    }
}

impl SequenceMemory {
    fn mouse_input(&mut self, e: MouseEvent, terminal: &mut DefaultTerminal) -> io::Result<()> {
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
            'outer: for y in rects {
                for el in &y[1..4] {
                    if mouse_rect.intersects(*el) {
                        self.clicked = Some((index, Instant::now()));
                        self.curr.push(index);
                        if self.check_validity() {
                            self.mode = Mode::Pause(Instant::now());
                            let old = self.scramble.last().unwrap();
                            let mut rng = rng();
                            let mut new = rng.random_range(0..9);
                            while new == *old {
                                new = rng.random_range(0..9);
                            }
                            self.scramble.push(new);
                            self.curr.clear();
                        }
                        break 'outer;
                    }
                    index += 1;
                }
            }
        }
        Ok(())
    }

    fn check_validity(&mut self) -> bool {
        for x in 0..self.curr.len() {
            if self.curr[x] != self.scramble[x] {
                self.mode = Mode::Results;
                self.savestate.update(self.get_score() as f32);
                self.curr.clear();
                return false;
            }
        }

        self.curr.len() == self.scramble.len()
    }

    fn get_score(&self) -> u32 {
        self.scramble.len().saturating_sub(1) as u32
    }

    fn reset(&mut self) {
        let st = self.savestate;
        *self = Self {
            savestate: st,
            scramble: vec![rng().random_range(0..9)],
            ..Default::default()
        };
    }
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
        match self.mode {
            Mode::Waiting => {
                if event::poll(Duration::MAX)? {
                    let event = event::read()?;

                    match event {
                        event::Event::Key(key) => match key.code {
                            KeyCode::Enter | KeyCode::Char(' ') => self.mode = Mode::Watching(0),
                            KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                            _ => (),
                        },
                        event::Event::Mouse(mouse) => {
                            if let MouseEventKind::Down(_) = mouse.kind {
                                self.mode = Mode::Watching(0);
                            }
                        }
                        _ => (),
                    }
                }
            }
            Mode::Watching(step) => {
                let mut dur = Duration::from_millis(FADE_OUT + 100);
                while dur.as_millis() != 0 {
                    let then = Instant::now();
                    if event::poll(dur)? {
                        let event = event::read()?;
                        if let event::Event::Key(key) = event {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    self.exit = true;
                                    return Ok(());
                                }
                                KeyCode::Char('r') => {
                                    self.reset();
                                    return Ok(());
                                }
                                _ => (),
                            }
                        }
                    }
                    dur = dur.saturating_sub(then.elapsed());
                }

                self.mode = if step == self.scramble.len().saturating_sub(1) as u32 {
                    Mode::Clicking
                } else {
                    Mode::Watching(step + 1)
                };
            }
            Mode::Pause(instant) => {
                let dur = Duration::from_millis(FADE_OUT * 2).saturating_sub(instant.elapsed());
                if event::poll(dur / 2)? {
                    match event::read()? {
                        event::Event::Key(KeyEvent {
                            code: KeyCode::Esc, ..
                        })
                        | event::Event::Key(KeyEvent {
                            code: KeyCode::Char('q'),
                            ..
                        }) => {
                            self.exit = true;
                        }
                        _ => (),
                    }
                }
                if dur.is_zero() {
                    self.mode = Mode::Watching(0);
                }
            }
            Mode::Clicking => {
                let dur = if let Some(val) = self.clicked {
                    Duration::from_millis(FADE_OUT)
                        .checked_sub(val.1.elapsed())
                        .unwrap_or(Duration::MAX)
                } else {
                    Duration::MAX
                };
                if event::poll(dur)? {
                    let event = event::read()?;
                    match event {
                        event::Event::Mouse(e) => {
                            self.mouse_input(e, terminal)?;
                        }
                        event::Event::Key(key) => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            Mode::Results => {
                if event::poll(Duration::from_secs(10))? {
                    let event = event::read()?;
                    if let event::Event::Key(key) = event {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                self.exit = true;
                                return Ok(());
                            }
                            KeyCode::Enter | KeyCode::Char('r') => {
                                self.reset();
                            }
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

impl Filed<'_> for SequenceMemory {
    const NAME: &'static str = FILE_NAME;
    type SaveState = SaveState;

    fn from_savestate(savestate: Self::SaveState) -> Self {
        Self {
            savestate,
            ..Default::default()
        }
    }

    fn get_savestate(&self) -> Self::SaveState {
        self.savestate
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

        Paragraph::new(Span::from("Sequence Memory Test").fg(Color::Red))
            .centered()
            .block(Block::bordered().border_set(border::DOUBLE))
            .render(vert[0], buf);

        let block = Block::bordered().border_set(border::DOUBLE);

        match self.mode {
            Mode::Waiting => {
                block.title("╡ Menu ╞").render(vert[1], buf);

                let constraints = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(main);

                Paragraph::new("Click/Enter to start playing")
                    .centered()
                    .render(constraints[1], buf);
            }
            Mode::Clicking | Mode::Pause(_) => {
                block.title("╡ Playing ╞").render(vert[1], buf);

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
                        Constraint::Length(1), // Title
                        Constraint::Length(4), // Title
                        Constraint::Length(4), // Title
                        Constraint::Length(4), // Title
                        Constraint::Min(0),    // ---
                    ])
                    .split(main);

                Paragraph::new(format!("Score: {}", self.get_score()));

                let lenth = rows[2].height * 2;

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
                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(lenth),
                            Constraint::Length(lenth),
                            Constraint::Length(lenth),
                            Constraint::Min(0),
                        ])
                        .split(rows[4]),
                ];

                let mut index = 0;
                for y in rects {
                    for element in &y[1..4] {
                        if index == clicked {
                            let inner = element.inner(ratatui::layout::Margin {
                                horizontal: 1,
                                vertical: 1,
                            });
                            Block::bordered()
                                .border_set(border::QUADRANT_INSIDE)
                                .set_style(Style::default().fg(Color::White))
                                .render(*element, buf);
                            Block::new()
                                .set_style(Style::default().bg(Color::White))
                                .render(inner, buf);
                        } else {
                            Block::bordered()
                                .border_set(border::THICK)
                                .render(*element, buf);
                        }
                        index += 1;
                    }
                }
            }
            Mode::Watching(step) => {
                block.title("╡ Watching ╞").render(vert[1], buf);

                let rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),    // ---
                        Constraint::Length(1), // Title
                        Constraint::Length(4), // Title
                        Constraint::Length(4), // Title
                        Constraint::Length(4), // Title
                        Constraint::Min(0),    // ---
                    ])
                    .split(main);

                Paragraph::new(format!("Score: {}", self.get_score()));

                let lenth = rows[2].height * 2;

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
                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(lenth),
                            Constraint::Length(lenth),
                            Constraint::Length(lenth),
                            Constraint::Min(0),
                        ])
                        .split(rows[4]),
                ];

                let mut index = 0;
                for y in rects {
                    for element in &y[1..4] {
                        if index == self.scramble[step as usize] {
                            let inner = element.inner(ratatui::layout::Margin {
                                horizontal: 1,
                                vertical: 1,
                            });
                            Block::bordered()
                                .border_set(border::QUADRANT_INSIDE)
                                .set_style(Style::default().fg(Color::White))
                                .render(*element, buf);
                            Block::new()
                                .set_style(Style::default().bg(Color::White))
                                .render(inner, buf);
                        } else {
                            Block::bordered()
                                .border_set(border::THICK)
                                .render(*element, buf);
                        }
                        index += 1;
                    }
                }
            }
            Mode::Results => {
                block.title("╡ Results ╞").render(vert[1], buf);

                let dataset = Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .cyan()
                    .data(&[
                        (0.0, (0.0 / 280.0)),
                        (1.0, (50.0 / 280.0)),
                        (2.0, (95.0 / 280.0)),
                        (3.0, (40.0 / 280.0)),
                        (4.0, (40.0 / 280.0)),
                        (5.0, (66.0 / 280.0)),
                        (6.0, (130.0 / 280.0)),
                        (7.0, (211.0 / 280.0)),
                        (8.0, (265.0 / 280.0)),
                        (9.0, (265.0 / 280.0)),
                        (10.0, (242.0 / 280.0)),
                        (11.0, (210.0 / 280.0)),
                        (12.0, (170.0 / 280.0)),
                        (13.0, (130.0 / 280.0)),
                        (14.0, (100.0 / 280.0)),
                        (15.0, (75.0 / 280.0)),
                        (16.0, (60.0 / 280.0)),
                        (17.0, (40.0 / 280.0)),
                        (18.0, (30.0 / 280.0)),
                        (19.0, (30.0 / 280.0)),
                        (20.0, (20.0 / 280.0)),
                        (21.0, (17.0 / 280.0)),
                        (22.0, (15.0 / 280.0)),
                        (23.0, (14.0 / 280.0)),
                        (24.0, (13.0 / 280.0)),
                        (25.0, (10.0 / 280.0)),
                        (26.0, (7.0 / 280.0)),
                        (27.0, (5.0 / 280.0)),
                        (28.0, (0.0 / 280.0)),
                        (29.0, (0.0 / 280.0)),
                        (30.0, (0.0 / 280.0)),
                    ]);
                render_graph(
                    self.savestate.avg_score as f64,
                    self.get_score() as f64,
                    dataset,
                    [0.0, 30.0],
                    main,
                    buf,
                );
            }
        }
    }
}
