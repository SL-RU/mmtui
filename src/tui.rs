use crate::drives;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::Color,
    text::Line,
    widgets::{Block, BorderType, Padding, Paragraph, Row, Table, TableState, Widget, Wrap},
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

const HELP: &str = "j/▲⋮k/▼⋮l/o/▶ - CD⋮m - Mount⋮u - Unmount⋮e - Eject⋮q - Quit";

impl Tui {
    fn set_status(&mut self, res: udisks2::Result<()>) {
        self.last_status = match res {
            Ok(()) => String::from("Ok"),
            Err(e) => format!("Error: {e:?}"),
        }
    }

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
                    self.set_status(drives::mount(b).await);
                }
                InputResult::None
            }
            KeyCode::Char('u') => {
                if let Some(b) = &self.selected {
                    self.set_status(drives::unmount(b).await);
                }
                InputResult::None
            }
            KeyCode::Char('e') => {
                if let Some(b) = &self.selected {
                    self.set_status(drives::eject(b).await);
                }
                InputResult::None
            }
            KeyCode::Enter | KeyCode::Char('l' | 'o') => {
                if let Some(s) = &self.selected {
                    let output = s.clone().mount.unwrap_or_default();
                    InputResult::QuitChangeDirectory(output)
                } else {
                    InputResult::Quit
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => InputResult::Quit,
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

        let max_size_field_length: u16 = rows
            .iter()
            .map(|r| r.size.len())
            .max()
            .unwrap_or(0)
            .try_into()
            .unwrap_or(0);

        let rows = rows.iter().map(|i| {
            Row::new(vec![
                i.dev.clone(),
                i.label.clone(),
                i.mount.clone().unwrap_or_default(),
                i.size.clone(),
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
            Constraint::Length(max_size_field_length),
            Constraint::Length(1),
        ];
        let table = Table::new(rows, widths)
            .row_highlight_style(Color::Green)
            .highlight_symbol(">");

        frame.render_stateful_widget(table, block.inner(layout[0]), &mut self.ts);

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
                    "dev: {:?} label: {:?} type: {:?} size {} {mounted}",
                    s.dev, s.label, s.fstype, s.size
                )
            }
            None => String::new(),
        };

        let info = format!("{} | {HELP}\n{descr}", self.last_status);
        let info = Paragraph::new(info).wrap(Wrap { trim: true });
        frame.render_widget(info, layout[1]);
    }
}
