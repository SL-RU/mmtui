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

        let max_field_length = |field: fn(&drives::Block) -> &str| -> u16 {
            rows.iter()
                .map(|r| field(r).len())
                .max()
                .unwrap_or(0)
                .try_into()
                .unwrap_or(0)
        };
        let table_area = block.inner(layout[0]);
        let max_dev_field_length = max_field_length(|r| &r.dev);
        let max_label_field_length = max_field_length(|r| &r.label);
        let max_fstype_field_length = max_field_length(|r| &r.fstype);
        let max_mount_field_length = max_field_length(|r| r.mount.as_deref().unwrap_or_default());
        let max_size_field_length = max_field_length(|r| &r.size);

        let rows = rows.iter().map(|i| {
            Row::new(vec![
                i.dev.clone(),
                i.label.clone(),
                i.fstype.clone(),
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
            Constraint::Fill(max_dev_field_length.min(table_area.width / 3)),
            Constraint::Fill(max_label_field_length),
            Constraint::Fill(max_fstype_field_length),
            Constraint::Fill(max_mount_field_length),
            Constraint::Length(max_size_field_length),
            Constraint::Length(1),
        ];
        let table = Table::new(rows, widths)
            .row_highlight_style(Color::Green)
            .highlight_symbol(">");

        frame.render_stateful_widget(table, table_area, &mut self.ts);

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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_row(dev: &str, terminal_width: u16) -> String {
        let mut tui = Tui {
            drv: vec![drives::Drive {
                id: String::new(),
                object_path: "/drive".to_owned(),
                model: String::new(),
                ejectable: false,
                blocks: vec![drives::Block {
                    object_path: String::new(),
                    drive_path: "/drive".to_owned(),
                    dev: dev.to_owned(),
                    label: "photos".to_owned(),
                    mount: Some("/run/media/user/photos".to_owned()),
                    fstype: "exfat".to_owned(),
                    mounted: true,
                    size: "64 GB".to_owned(),
                }],
            }],
            ..Default::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(terminal_width, 20)).unwrap();
        terminal.draw(|frame| tui.draw(frame)).unwrap();

        let buffer = terminal.backend().buffer();
        buffer
            .content
            .chunks(buffer.area.width.into())
            .map(|line| line.iter().map(ratatui::buffer::Cell::symbol).collect())
            .find(|line: &String| line.starts_with("│ >"))
            .unwrap()
    }

    fn column(row: &str, text: &str) -> usize {
        row.find(text).expect(text)
    }

    // The user identifies a device by its label and fstype, and the full mount path must stay visible.
    #[test]
    fn row_shows_fstype_between_label_and_full_mount_path() {
        let row = render_row("/dev/sda1", 100);
        assert!(column(&row, "photos") < column(&row, "exfat"));
        assert!(column(&row, "exfat") < column(&row, "/run/media/user/photos"));
    }

    // The free width goes to all columns, in proportion to the length of their content.
    // A short label must not get as much width as a long mount path.
    #[test]
    fn free_width_follows_content_length() {
        let row = render_row("/dev/sda1", 100);
        let dev_width = column(&row, "photos") - column(&row, "/dev/sda1");
        let label_width = column(&row, "exfat") - column(&row, "photos");
        let mount_width = column(&row, "64 GB") - column(&row, "/run/media/user/photos");
        assert!(dev_width > "/dev/sda1 ".len());
        assert!(label_width < dev_width);
        assert!(dev_width < mount_width);
    }

    // Network mounts (sshfs, NFS) have long dev names.
    // The other columns must stay visible, mainly the mount path, because `l` goes to it.
    #[test]
    fn long_dev_name_does_not_hide_other_columns() {
        let row = render_row("user@fileserver.example.com:/srv/share", 80);
        assert!(column(&row, "photos") < column(&row, "exfat"));
        assert!(column(&row, "exfat") < column(&row, "/run/media/user/photos"));
    }
}
