use crate::drives;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::Color,
    text::Line,
    widgets::{
        Block, BorderType, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget, Wrap,
    },
    Frame,
};

pub enum InputResult {
    None,
    Quit,
    QuitChangeDirectory(String),
}

#[derive(Default)]
pub struct Tui {
    ts: TableState,
    pub drv: Vec<drives::Drive>,
    pub selected: Option<drives::Block>,
    pub last_status: String,
}

impl Tui {
    pub async fn input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.ts.select_previous();
                InputResult::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.ts.select_next();
                InputResult::None
            }
            KeyCode::Char('m') => {
                if let Some(b) = &self.selected {
                    self.last_status = format!("{:?}", drives::mount(b).await);
                }
                InputResult::None
            }
            KeyCode::Char('u') => {
                if let Some(b) = &self.selected {
                    self.last_status = format!("{:?}", drives::unmount(b).await);
                }
                InputResult::None
            }
            KeyCode::Esc | KeyCode::Char('q') => InputResult::Quit,
            KeyCode::Enter => {
                let output = self.selected.clone().unwrap().mount.unwrap();
                InputResult::QuitChangeDirectory(output)
            }
            _ => InputResult::None,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
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

        let rows: Vec<drives::Block> = self.drv.iter().flat_map(|d| d.blocks.clone()).collect();

        if self.ts.selected().is_none() && !self.drv.is_empty() {
            self.ts.select(Some(0));
        }

        self.ts
            .selected()
            .and_then(|n| rows.get(n).cloned())
            .clone_into(&mut self.selected);

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

        StatefulWidget::render(
            table,
            block.inner(frame.area()),
            frame.buffer_mut(),
            &mut self.ts,
        );

        let descr = match &self.selected {
            Some(s) => {
                let mnt_point = match &s.mount {
                    None => "",
                    Some(m) => m,
                };
                let mounted = if s.mounted {
                    format!("mounted to {mnt_point:?}")
                } else {
                    format!("not mounted {mnt_point:?}")
                };

                format!(
                    "dev: {:?} label: {:?} type: {:?} {mounted} ",
                    s.dev, s.label, s.fstype
                )
            }
            None => String::new(),
        };

        let info = format!(
            "j - UP, k - DOWN, l - Goto mountpoint, m - Mount, u - Unmount, e - Eject\n{descr} {:?}",
            self.last_status
        );

        let info = Paragraph::new(info).wrap(Wrap { trim: true });
        frame.render_widget(info, layout[1]);
    }
}
