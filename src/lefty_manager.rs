#[rustfmt::skip]
const ASCII_BANNER: [(&str, &str, &str); 6] = [
    ("#88c0d0", "#1b1b1b", "██╗     ███████╗███████╗████████╗██╗   ██╗██████╗ ███████╗"),
    ("#e5e9f0", "#1b1b1b", "██║     ██╔════╝██╔════╝╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝"),
    ("#8fbcbb", "#1b1b1b", "██║     █████╗  █████╗     ██║    ╚████╔╝ ██████╔╝███████╗"),
    ("#81a1c1", "#1b1b1b", "██║     ██╔══╝  ██╔══╝     ██║     ╚██╔╝  ██╔══██╗╚════██║"),
    ("#68809a", "#1b1b1b", "███████╗███████╗██║        ██║      ██║██╗██║  ██║███████║"),
    ("#373e4d", "#1b1b1b", "╚══════╝╚══════╝╚═╝        ╚═╝      ╚═╝╚═╝╚═╝  ╚═╝╚══════╝")
];
mod util;
use crate::util::event::{Event, Events};
use clap::Parser;
use leftwm_theme_dark_nord::Result;
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};
use util::TabsState;
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
            util::ui::draw(f, &mut app);
        })?;

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char(c) => {
                    app.on_key(c);
                }
                Key::Esc => {
                    app.on_esc();
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
    pub show_modal: bool,
    pub main_tabs: TabsState<'a>,
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
            show_modal: false,
            main_tabs: TabsState::new(vec!["General", "Bar"]),
        }
    }

    pub fn on_esc(&mut self) {
        self.show_modal = !self.show_modal;
        // self.tasks.previous();
    }

    pub fn on_up(&mut self) {
        // self.tasks.previous();
    }

    pub fn on_down(&mut self) {
        // self.tasks.next();
    }

    pub fn on_right(&mut self) {
        self.main_tabs.next();
    }

    pub fn on_left(&mut self) {
        self.main_tabs.previous();
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
