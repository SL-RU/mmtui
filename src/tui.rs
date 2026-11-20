use crate::drives;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout},
    style::Color,
    text::Line,
    widgets::{Block, BorderType, Clear, Padding, Paragraph, Row, Table, TableState, Widget, Wrap},
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
    passphrase_prompt: Option<PassphrasePrompt>,
}

struct PassphrasePrompt {
    block: drives::Block,
    passphrase: String,
}

const HELP: &str = "j/▲⋮k/▼⋮l/o/▶ - CD⋮m - Mount⋮u - Unmount⋮e - Eject⋮q - Quit";
const PROMPT_HELP: &str = "Enter - Unlock⋮Esc - Cancel";

impl Tui {
    fn set_status(&mut self, res: udisks2::Result<()>) {
        self.last_status = match res {
            Ok(()) => String::from("Ok"),
            Err(e) => format!("Error: {e:?}"),
        }
    }

    pub async fn input(&mut self, key: KeyEvent) -> InputResult {
        if let Some(prompt) = &mut self.passphrase_prompt {
            match key.code {
                KeyCode::Char(character) => prompt.passphrase.push(character),
                KeyCode::Backspace => {
                    prompt.passphrase.pop();
                }
                KeyCode::Esc => self.passphrase_prompt = None,
                KeyCode::Enter => {
                    if let Some(prompt) = self.passphrase_prompt.take() {
                        self.set_status(
                            drives::unlock_and_mount(&prompt.block, &prompt.passphrase).await,
                        );
                    }
                }
                _ => {}
            }
            return InputResult::None;
        }

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
                    if let drives::Encryption::Locked = b.encryption {
                        self.passphrase_prompt = Some(PassphrasePrompt {
                            block: b.clone(),
                            passphrase: String::new(),
                        });
                    } else {
                        self.set_status(drives::mount(b).await);
                    }
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

        if let Some(prompt) = &self.passphrase_prompt {
            let [area] = Layout::vertical([Constraint::Length(3)])
                .flex(Flex::Center)
                .areas(frame.area());
            let [area] = Layout::horizontal([Constraint::Percentage(60)])
                .flex(Flex::Center)
                .areas(area);
            let block = Block::bordered()
                .title(Line::styled(
                    format!("Passphrase for {}", prompt.block.dev),
                    Color::White,
                ))
                .title_bottom(PROMPT_HELP)
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(Color::Yellow);
            let hidden_passphrase = "*".repeat(prompt.passphrase.chars().count());
            frame.render_widget(Clear, area);
            frame.render_widget(Paragraph::new(hidden_passphrase).block(block), area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tui_with_locked_luks_row() -> Tui {
        let block = drives::Block {
            object_path: "/org/freedesktop/UDisks2/block_devices/sdc2".to_owned(),
            drive_path: "/org/freedesktop/UDisks2/drives/disk".to_owned(),
            dev: "/dev/sdc2".to_owned(),
            label: String::new(),
            mount: None,
            fstype: "crypto_LUKS".to_owned(),
            mounted: false,
            size: String::new(),
            encryption: drives::Encryption::Locked,
        };
        Tui {
            selected: Some(block),
            ..Tui::default()
        }
    }

    async fn press(tui: &mut Tui, code: KeyCode) -> InputResult {
        tui.input(KeyEvent::from(code)).await
    }

    async fn type_text(tui: &mut Tui, text: &str) {
        for character in text.chars() {
            press(tui, KeyCode::Char(character)).await;
        }
    }

    // udisks cannot mount a locked device. `m` must ask for the passphrase of
    // the selected device and must not send a mount request.
    #[tokio::test]
    async fn mount_of_locked_device_asks_for_passphrase() {
        let mut tui = tui_with_locked_luks_row();
        press(&mut tui, KeyCode::Char('m')).await;
        let prompt = tui.passphrase_prompt.expect("no passphrase prompt");
        assert_eq!(prompt.block.dev, "/dev/sdc2");
        assert_eq!(tui.last_status, "");
    }

    // A passphrase can contain command keys, for example `q`. In the prompt
    // they must become passphrase text, and must not quit or start a command.
    #[tokio::test]
    async fn prompt_takes_command_keys_as_passphrase_text() {
        let mut tui = tui_with_locked_luks_row();
        press(&mut tui, KeyCode::Char('m')).await;
        for character in "qmuejkl".chars() {
            let result = press(&mut tui, KeyCode::Char(character)).await;
            assert!(matches!(result, InputResult::None));
        }
        press(&mut tui, KeyCode::Backspace).await;
        let prompt = tui.passphrase_prompt.expect("prompt closed");
        assert_eq!(prompt.passphrase, "qmuejk");
    }

    // Esc must cancel: no unlock request (it sets a status), and the normal
    // keys work again.
    #[tokio::test]
    async fn escape_closes_prompt_without_unlock() {
        let mut tui = tui_with_locked_luks_row();
        press(&mut tui, KeyCode::Char('m')).await;
        type_text(&mut tui, "secret").await;
        let result = press(&mut tui, KeyCode::Esc).await;
        assert!(matches!(result, InputResult::None));
        assert!(tui.passphrase_prompt.is_none());
        assert_eq!(tui.last_status, "");
        assert!(matches!(
            press(&mut tui, KeyCode::Char('q')).await,
            InputResult::Quit
        ));
    }
}
