use crate::App;
use leftwm_theme_dark_nord::theme::Rgba;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap},
    Terminal,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let tabs = MainTabView {};
    tabs.draw(f, app);

    // let paragraph = Paragraph:1:new(text)
    //     .block(Block::default().title("Right Block").borders(Borders::ALL))
    //     .alignment(Alignment::Left)
    //     .wrap(Wrap { trim: true });
    // f.render_widget(paragraph, chunks[1]);

    if app.show_modal {
        let modal = ModalDialog { title: "" };
        modal.draw(f, app)
    }
}

pub trait Draw {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, app: &mut App);
}

struct ModalDialog<'a> {
    pub title: &'a str,
}

impl<'a> Draw for ModalDialog<'a> {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, _app: &mut App) {
        let area = centered_rect(60, 20, f.size());
        let lines = crate::ASCII_BANNER
            .iter()
            .map(|l| {
                Spans::from(Span::styled(
                    l.2,
                    Style::default().fg(Rgba::from(l.0).into()),
                ))
            })
            .collect::<Vec<Spans>>();
        let paragraph = Paragraph::new(lines)
            .block(Block::default().title(self.title).borders(Borders::ALL))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(paragraph, area);
    }
}

struct MainTabView {}

impl Draw for MainTabView {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, app: &mut App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());

        let titles = app
            .main_tabs
            .titles
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Spans::from(vec![
                    Span::styled(first, Style::default().fg(Color::Yellow)),
                    Span::styled(rest, Style::default().fg(Color::Green)),
                ])
            })
            .collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .select(app.main_tabs.index)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );
        f.render_widget(tabs, chunks[0]);
        let inner = match app.main_tabs.index {
            0 => Block::default().title("Inner 0").borders(Borders::ALL),
            1 => Block::default().title("Inner 1").borders(Borders::ALL),
            2 => Block::default().title("Inner 2").borders(Borders::ALL),
            3 => Block::default().title("Inner 3").borders(Borders::ALL),
            _ => unreachable!(),
        };
        f.render_widget(inner, chunks[1]);
    }
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
