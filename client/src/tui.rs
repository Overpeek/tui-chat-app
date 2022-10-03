use ::tui::{Frame, Terminal};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::{self, Stdout},
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
};

//

pub fn run(tick_rate: Duration, no_unicode: bool) -> Result<(), Box<dyn Error>> {
    // setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = App::new(no_unicode).run(&mut terminal, tick_rate);

    // restore
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

//

struct App {
    no_unicode: bool,
    should_close: bool,

    // input or messages
    focus: Focus,
    input: String,

    messages: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum Focus {
    Input { idx: usize },
    Chat { idx: usize },
}

//

impl App {
    fn new(no_unicode: bool) -> Self {
        Self {
            no_unicode,
            should_close: false,

            focus: Focus::Input { idx: 0 },
            input: String::new(),

            messages: vec![],
        }
    }

    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        tick_rate: Duration,
    ) -> Result<(), Box<dyn Error>> {
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|f| self.draw(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        /* KeyCode::Char(c) => app.on_key(c),
                        KeyCode::Left => app.on_left(),
                        KeyCode::Up => app.on_up(),
                        KeyCode::Right => app.on_right(),
                        KeyCode::Down => app.on_down(), */
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.should_close = true
                        }
                        KeyCode::Esc => self.should_close = true,
                        _ => self.key_event(key),
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                self.tick();
                last_tick = Instant::now();
            }
            if self.should_close {
                return Ok(());
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
        let Rect { width, height, .. } = frame.size();
        let min_width = 96;
        let min_height = 24;
        if width < min_width || height < min_height {
            frame.render_widget(
                Paragraph::new(vec![
                    vec![Span::styled(
                        "Terminal size too small:",
                        Style::default().fg(Color::White),
                    )]
                    .into(),
                    vec![Span::styled(
                        format!("width = {width}, height = {height}"),
                        Style::default().fg(Color::White),
                    )]
                    .into(),
                    vec![].into(),
                    vec![Span::styled(
                        "Minimum terminal size:",
                        Style::default().fg(Color::White),
                    )]
                    .into(),
                    vec![Span::styled(
                        format!("width = {min_width}, height = {min_height}"),
                        Style::default().fg(Color::White),
                    )]
                    .into(),
                ]),
                frame.size(),
            );
            return;
        }

        let split = Layout::default()
            .constraints([
                Constraint::Length(9),
                Constraint::Length(1),
                Constraint::Min(86),
            ])
            .direction(Direction::Horizontal)
            .split(frame.size());
        frame.render_widget(Block::default().borders(Borders::RIGHT), split[1]);

        // server list
        let server_list_view = split[0];
        frame.render_widget(Block::default().title("Servers"), server_list_view);

        // server
        let server_view = split[2];
        self.draw_server(frame, server_view);
    }

    fn draw_server(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, rect: Rect) {
        let split = Layout::default()
            .constraints([
                Constraint::Min(65),
                Constraint::Length(1),
                Constraint::Length(20),
            ])
            .direction(Direction::Horizontal)
            .split(rect);
        frame.render_widget(Block::default().borders(Borders::RIGHT), split[1]);

        // chat
        let chat_view = split[0];
        self.draw_chat(frame, chat_view);

        // member list
        let member_view = split[2];
        frame.render_widget(Block::default().title("Online - 0"), member_view);
    }

    fn draw_chat(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, rect: Rect) {
        let split = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(20),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .direction(Direction::Vertical)
            .split(rect);
        frame.render_widget(Block::default().borders(Borders::BOTTOM), split[1]);
        frame.render_widget(Block::default().borders(Borders::BOTTOM), split[3]);

        // title
        let title_view = split[0];
        frame.render_widget(Block::default().title("Server name"), title_view);

        // messages
        let message_view = split[2];
        frame.render_widget(Block::default(), message_view);

        // input
        let input_view = split[4];
        frame.render_widget(
            Paragraph::new(vec![vec![
                Span::styled("> ", Style::default().fg(Color::White)),
                Span::styled(self.input.as_str(), Style::default().fg(Color::LightGreen)),
            ]
            .into()]),
            input_view,
        );
        if let Focus::Input { idx } = self.focus {
            frame.set_cursor(
                input_view.x + 2 + idx.try_into().unwrap_or(0_u16),
                input_view.y,
            );
        }
    }

    fn tick(&mut self) {}

    fn key_event(&mut self, event: KeyEvent) {
        if let Focus::Input { idx } = &mut self.focus {
            match event.code {
                // Doesn't work in crossterm yet
                //
                // https://github.com/crossterm-rs/crossterm/issues/685#issue-1290596799
                KeyCode::Backspace if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-') =
                        Self::pop_input(&mut self.input, idx, -1)
                    {}
                }
                KeyCode::Backspace => {
                    Self::pop_input(&mut self.input, idx, -1);
                }
                KeyCode::Delete => {
                    Self::pop_input(&mut self.input, idx, 0);
                }
                KeyCode::Char(ch) => {
                    self.input.insert(*idx, ch);
                    *idx = self.input.len().min(idx.saturating_add(1));
                }
                KeyCode::Left => {
                    *idx = self.input.len().min(idx.saturating_sub(1));
                }
                KeyCode::Right => {
                    *idx = self.input.len().min(idx.saturating_add(1));
                }
                KeyCode::Home => {
                    *idx = 0;
                }
                KeyCode::End => {
                    *idx = self.input.len();
                }
                KeyCode::Enter => {
                    if self.input.chars().all(|c| c.is_whitespace()) {
                        // dont send whitespace only messages
                        //
                        // server also blocks these
                        return;
                    }

                    let input = self.input.trim();
                }
                _ => {}
            }
        }
    }

    fn pop_input(input: &mut String, idx: &mut usize, offset: isize) -> Option<char> {
        let pos = (*idx as isize + offset) as usize;
        if input.len() <= pos {
            return None;
        }
        let ch = input.remove(pos);
        *idx = input.len().min(pos);
        Some(ch)
    }
}
