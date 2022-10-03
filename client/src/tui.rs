use ::tui::{Frame, Terminal};
use chrono::{DateTime, Local};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify_rust::Notification;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    io::{self, Stdout},
    mem::swap,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::{Receiver, Sender};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};
use tui_chat_app_common::{
    client::{ClientChatPacket, ClientPacket},
    server::{ServerChatPacket, ServerPacket},
};
use uuid::Uuid;

//

pub async fn run(
    tick_rate: Duration,
    no_unicode: bool,
    recv: Receiver<ServerPacket>,
    send: Sender<ClientPacket>,
) -> Result<(), Box<dyn Error>> {
    // setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = App::new(no_unicode, recv, send)
        .run(&mut terminal, tick_rate)
        .await;

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

    messages: Vec<Message>,
    all_messages: HashMap<Uuid, HashMap<Uuid, String>>,
    self_id: SelfUuid,

    recv: Receiver<ServerPacket>,
    send: Sender<ClientPacket>,
}

#[derive(Debug, Clone, Copy)]
enum Focus {
    Input { idx: usize },
    Chat { idx: usize },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelfUuid {
    Some(Uuid),
    Pending(Instant),
    None,
}

struct Message {
    sender_id: Uuid,
    message_id: Uuid,
    timestamp: DateTime<Local>,
}

//

impl Display for SelfUuid {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SelfUuid::Some(uuid) => write!(f, "{uuid}"),
            SelfUuid::Pending(_) => write!(f, "name pending"),
            SelfUuid::None => write!(f, "name none"),
        }
    }
}

impl App {
    fn new(no_unicode: bool, recv: Receiver<ServerPacket>, send: Sender<ClientPacket>) -> Self {
        Self {
            no_unicode,
            should_close: false,

            focus: Focus::Input { idx: 0 },
            input: String::new(),

            messages: vec![],
            all_messages: HashMap::new(),
            self_id: SelfUuid::None,

            recv,
            send,
        }
    }

    async fn run(
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
                        _ => self.key_event(key).await,
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                self.tick().await;
                last_tick = Instant::now();
            }
            if self.should_close {
                return Ok(());
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
        let Rect { width, height, .. } = frame.size();
        let min_width = 50;
        let min_height = 14;
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
                Constraint::Min(40),
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
                Constraint::Min(19),
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
                Constraint::Min(10),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .direction(Direction::Vertical)
            .split(rect);
        frame.render_widget(Block::default().borders(Borders::BOTTOM), split[1]);
        frame.render_widget(Block::default().borders(Borders::BOTTOM), split[3]);

        // title
        let title_view = split[0];
        frame.render_widget(
            Block::default().title(format!("Server name - {}", self.self_id)),
            title_view,
        );

        // messages
        let message_view = split[2];
        frame.render_widget(Block::default(), message_view);

        let mut messages = self
            .messages
            .iter()
            .rev()
            .filter_map(|m| Some((m, self.all_messages.get(&m.sender_id)?)))
            .filter_map(|(m, sender)| Some((m, sender.get(&m.message_id)?)));

        let mut message_buffer: Vec<Spans> = vec![];
        let mut last_sender = None;
        while let Some((message, message_str)) = messages.next() {
            if last_sender != Some(message.sender_id) {
                message_buffer.push(vec![].into());
                message_buffer.push(
                    vec![
                        Span::styled(
                            format!("{}", message.sender_id),
                            Style::default().fg(Color::LightCyan),
                        ),
                        Span::styled(
                            format!(" {}", message.timestamp.format("%H:%M:%S")),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::ITALIC)
                                .add_modifier(Modifier::DIM),
                        ),
                    ]
                    .into(),
                );
            }
            last_sender = Some(message.sender_id);

            message_buffer.push(
                vec![Span::styled(
                    message_str.as_str(),
                    Style::default().fg(Color::White),
                )]
                .into(),
            );
        }

        frame.render_widget(Paragraph::new(message_buffer), message_view);

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

    async fn tick(&mut self) {
        match self.recv.try_recv() {
            Ok(ServerPacket::Chat(ServerChatPacket::NewMessage {
                sender_id,
                message_id,
                message,
            })) => {
                if self.self_id != SelfUuid::Some(sender_id) {
                    let notify = format!("{sender_id}:\n{message}");
                    let _ = Notification::new()
                        .summary("Message")
                        .body(notify.as_str())
                        .show();
                }

                self.all_messages
                    .entry(sender_id)
                    .or_default()
                    .insert(message_id, message);
                self.messages.push(Message {
                    sender_id,
                    message_id,
                    timestamp: Local::now(),
                });
            }
            Ok(ServerPacket::Chat(ServerChatPacket::SelfMember { member_id })) => {
                self.self_id = SelfUuid::Some(member_id);
            }
            _ => (),
        }

        match self.self_id {
            SelfUuid::None => {
                self.self_id = SelfUuid::Pending(Instant::now());
                let _ = self
                    .send
                    .send(ClientPacket::Chat(ClientChatPacket::RequestSelfMember))
                    .await;
            }
            SelfUuid::Pending(i) if i.elapsed() >= Duration::SECOND => {
                self.self_id = SelfUuid::Pending(Instant::now());
                let _ = self
                    .send
                    .send(ClientPacket::Chat(ClientChatPacket::RequestSelfMember))
                    .await;
            }
            _ => {}
        }
    }

    async fn key_event(&mut self, event: KeyEvent) {
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

                    self.focus = Focus::Input { idx: 0 };
                    let mut input = String::new();
                    swap(&mut input, &mut self.input);
                    let message_id = Uuid::new_v4();

                    let _ = self
                        .send
                        .send(ClientPacket::Chat(ClientChatPacket::SendMessage {
                            message_id,
                            message: input.to_string(),
                        }))
                        .await;
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
