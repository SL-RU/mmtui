mod drives;
mod mountpoints;
mod tui;

use crossterm::{
    event::{Event, EventStream, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::{io::stderr, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tui::{InputResult, Tui};

#[tokio::main]
async fn main() -> udisks2::Result<()> {
    enable_raw_mode().unwrap();
    execute!(stderr(), EnterAlternateScreen).unwrap();
    let mut terminal = Terminal::new(CrosstermBackend::new(stderr())).unwrap();

    let period = Duration::from_secs_f32(1.0 / 10.0);
    let mut interval = tokio::time::interval(period);
    let mut events = EventStream::new();

    let state: Arc<Mutex<Vec<drives::Drive>>> = Arc::new(Mutex::new(Vec::new()));
    let s = state.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(drv) = drives::collect_all().await {
                s.lock().await.clone_from(&drv);
            };
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    let mut output_change_path = String::new();
    let mut tui = Tui::default();
    loop {
        tui.drv = state.lock().await.to_vec();
        tui.selected = None;
        terminal
            .draw(|f| tui.draw(f))
            .expect("failed to draw frame");

        let key = tokio::select! {
            _ = interval.tick() => continue,
            Some(Ok(Event::Key(key))) = events.next() => key,
        };

        if key.kind == KeyEventKind::Press {
            match tui.input(key).await {
                InputResult::None => continue,
                InputResult::Quit => break,
                InputResult::QuitChangeDirectory(p) => {
                    output_change_path = p;
                    break;
                }
            }
        }
    }

    disable_raw_mode().unwrap();
    execute!(stderr(), LeaveAlternateScreen).unwrap();
    println!("{output_change_path}");
    Ok(())
}
