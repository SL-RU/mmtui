mod drives;
mod mountpoints;

use std::{io::stderr, sync::Arc, time::Duration};

use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    prelude::CrosstermBackend,
    style::Color,
    text::{Line, Text},
    widgets::{
        Block, BorderType, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget, Wrap,
    },
    Frame, Terminal,
};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> udisks2::Result<()> {
    enable_raw_mode().unwrap();
    execute!(stderr(), EnterAlternateScreen).unwrap();
    let mut terminal = Terminal::new(CrosstermBackend::new(stderr())).unwrap();
    let mut ts = TableState::new();

    let period = Duration::from_secs_f32(1.0 / 10.0);
    let mut interval = tokio::time::interval(period);
    let mut events = EventStream::new();

    let state: Arc<Mutex<Vec<drives::Drive>>> = Arc::new(Mutex::new(Vec::new()));
    let s = state.clone();
    tokio::spawn(async move {
        loop {
            let drv = drives::collect_all().await.unwrap();
            s.lock().await.clone_from(&drv);
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    let mut output = String::new();
    let mut last_status = String::new();

    loop {
        let drv = state.lock().await.clone();
        let mut selected: Option<drives::Block> = None;
        terminal
            .draw(|f| draw(f, &mut ts, &drv, &mut selected, &last_status))
            .expect("failed to draw frame");

        tokio::select! {
            _ = interval.tick() => { },
            Some(Ok(event)) = events.next() => {
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Up | KeyCode::Char('k') => ts.select_previous(),
                            KeyCode::Down | KeyCode::Char('j') => ts.select_next(),
                            KeyCode::Char('m') => if let Some(b) = &selected {
                                last_status = format!("{:?}", drives::mount(b).await);
                            }
                            KeyCode::Char('u') => if let Some(b) = &selected {
                                last_status = format!("{:?}", drives::unmount(b).await);
                            }
                            KeyCode::Esc | KeyCode::Char('q') => break,
                            KeyCode::Enter => {
                                output = selected.unwrap().mount.unwrap();
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            },
        }
    }

    disable_raw_mode().unwrap();
    execute!(stderr(), LeaveAlternateScreen).unwrap();
    println!("{output}");
    Ok(())
}

fn draw(
    frame: &mut Frame,
    state: &mut TableState,
    drv: &[drives::Drive],
    selected: &mut Option<drives::Block>,
    last_status: &str,
) {
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

    let rows: Vec<drives::Block> = drv.iter().flat_map(|d| d.blocks.clone()).collect();

    state
        .selected()
        .and_then(|n| rows.get(n).cloned())
        .clone_into(selected);

    let rows = rows.iter().map(|i| {
        Row::new(vec![
            i.dev.clone(),
            i.label.clone(),
            i.mount.clone().unwrap_or_default(),
            if i.mounted {
                "M".to_owned()
            } else {
                "O".to_owned()
            },
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
        Paragraph::new(format!(
            "j - UP, k - DOWN, l - Goto mountpoint, m - Mount, u - Unmount, e - Eject\n{selected:?} {last_status:?}"
        ))
        .wrap(Wrap { trim: true }),
        layout[1],
    );
}
