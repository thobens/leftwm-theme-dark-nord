#[rustfmt::skip]
const ASCII_BANNER: [&str; 6] = ["██╗     ███████╗███████╗████████╗██╗   ██╗██████╗ ███████╗",
                                 "██║     ██╔════╝██╔════╝╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝",
                                 "██║     █████╗  █████╗     ██║    ╚████╔╝ ██████╔╝███████╗",
                                 "██║     ██╔══╝  ██╔══╝     ██║     ╚██╔╝  ██╔══██╗╚════██║",
                                 "███████╗███████╗██║        ██║      ██║██╗██║  ██║███████║",
                                 "╚══════╝╚══════╝╚═╝        ╚═╝      ╚═╝╚═╝╚═╝  ╚═╝╚══════╝"];
mod util;
use crate::util::event::{Event, Events};
use clap::Parser;
use leftwm_theme_dark_nord::Result;
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Terminal,
};
extern crate rand;

/// Termion demo
#[derive(Debug, Parser)]
struct Cli {
    /// time in ms between two ticks.
    #[clap(long, default_value = "250")]
    tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    #[clap(long)]
    enhanced_graphics: bool,
}

/// helper function to create a centered rect using up
/// certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();
    let mut app = App::new("Lefty prototype", cli.enhanced_graphics);

    loop {
        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(size);

                let s = "Veeeeeeeeeeeeeeeery    loooooooooooooooooong   striiiiiiiiiiiiiiiiiiiiiiiiiing.   ";
                let mut long_line = s.repeat(usize::from(size.width)*usize::from(size.height)/300);
                long_line.push('\n');

            let text = vec![
                Spans::from("This is a line "),
                Spans::from(Span::styled("This is a line   ", Style::default().fg(Color::Red))),
                Spans::from(Span::styled("This is a line", Style::default().bg(Color::Blue))),
                Spans::from(Span::styled(
                    "This is a longer line\n",
                    Style::default().add_modifier(Modifier::CROSSED_OUT),
                )),
                Spans::from(Span::styled(&long_line, Style::default().bg(Color::Green))),
                Spans::from(Span::styled(
                    "This is a line\n",
                    Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC),
                )),
            ];

            let paragraph = Paragraph::new(text.clone())
                .block(Block::default().title("Left Block").borders(Borders::ALL))
                .alignment(Alignment::Left).wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[0]);

            let paragraph = Paragraph::new(text)
                .block(Block::default().title("Right Block").borders(Borders::ALL))
                .alignment(Alignment::Left).wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);

            let block = Block::default().title("Popup").borders(Borders::ALL);
            let area = centered_rect(60, 20, size);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);
        })?;

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char(c) => {
                    app.on_key(c);
                }
                Key::Up => {
                    app.on_up();
                }
                Key::Down => {
                    app.on_down();
                }
                Key::Left => {
                    app.on_left();
                }
                Key::Right => {
                    app.on_right();
                }
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

pub struct Signal<S: Iterator> {
    source: S,
    pub points: Vec<S::Item>,
    tick_rate: usize,
}

impl<S> Signal<S>
where
    S: Iterator,
{
    fn on_tick(&mut self) {
        for _ in 0..self.tick_rate {
            self.points.remove(0);
        }
        self.points
            .extend(self.source.by_ref().take(self.tick_rate));
    }
}

pub struct Signals {
    // pub sig: Signal<SinSignal>,
    pub window: [f64; 2],
}

impl Signals {
    fn on_tick(&mut self) {
        // self.sig.on_tick();
        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub signals: Signals,
    pub enhanced_graphics: bool,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool) -> App<'a> {
        App {
            title,
            should_quit: false,
            enhanced_graphics,
            signals: Signals {
                window: [0.0, 20.0],
            },
        }
    }

    pub fn on_up(&mut self) {
        // self.tasks.previous();
    }

    pub fn on_down(&mut self) {
        // self.tasks.next();
    }

    pub fn on_right(&mut self) {
        // self.tabs.next();
    }

    pub fn on_left(&mut self) {
        // self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            'i' => {}
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        // Update progress

        self.signals.on_tick();
    }
}
