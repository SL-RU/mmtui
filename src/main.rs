mod drives;
mod mountpoints;

use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::{OsStrExt, OsStringExt},
    sync::{Arc, Mutex},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use mountpoints::MountPoint;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::Color,
    text::{Line, Text},
    widgets::{
        Block, BorderType, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget, Wrap,
    },
    Frame,
};

#[tokio::main]
async fn main() -> udisks2::Result<()> {
    let mut terminal = ratatui::init();
    let mut ts = TableState::new();
    loop {
        let drv = drives::collect_all().await?;
        terminal
            .draw(|f| draw(f, &mut ts, &drv))
            .expect("failed to draw frame");

        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Up => ts.select_previous(),
                    KeyCode::Down => ts.select_next(),
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }
    ratatui::restore();
    Ok(())
}

fn draw(frame: &mut Frame, state: &mut TableState, drv: &[drives::Drive]) {
    let text = Text::raw("Hello World!");
    frame.render_widget(text, frame.area());

    let layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints(vec![Constraint::Fill(1), Constraint::Length(5)])
        .split(frame.area());

    let block = Block::bordered()
        .title(Line::styled("Mount", Color::White))
        .title_alignment(Alignment::Center)
        .padding(Padding::symmetric(1, 1))
        .border_type(BorderType::Rounded)
        .border_style(Color::Yellow);
    block.clone().render(layout[0], frame.buffer_mut());

    let rows = drv.iter().flat_map(|d| &d.blocks).map(|i| {
        Row::new(vec![
            i.dev.clone(),
            i.label.clone(),
            i.mount.clone().unwrap_or_default(),
            "M".to_owned(),
        ])
    });
    let widths = [
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Length(3),
    ];
    let table = Table::new(rows, widths)
        .row_highlight_style(Color::Green)
        .highlight_symbol(">");

    StatefulWidget::render(table, block.inner(frame.area()), frame.buffer_mut(), state);
    frame.render_widget(
        Paragraph::new("j - UP, k - DOWN, l - Goto mountpoint\nm - Mount, u - Unmount, e - Eject")
            .wrap(Wrap { trim: true }),
        layout[1],
    );
}
