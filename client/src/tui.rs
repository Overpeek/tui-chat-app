use ::tui::{Frame, Terminal};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::{self, Stdout},
    time::{Duration, Instant},
};
use tui::backend::CrosstermBackend;

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
}

//

impl App {
    fn new(no_unicode: bool) -> Self {
        Self {
            no_unicode,
            should_close: false,
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
                        KeyCode::Esc => self.should_close = true,
                        _ => {}
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

    fn draw(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>) {}

    fn tick(&mut self) {}
}
