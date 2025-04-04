use std::mem;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::Widget,
};

use crate::uauauiua::Uauauiua;

const RECORD_KEY: KeyCode = KeyCode::Enter;
const RELOAD_KEY: KeyCode = KeyCode::Tab;
const STOP_PLAYBACK_KEY: KeyCode = KeyCode::Backspace;
const EXIT_KEY: KeyCode = KeyCode::Esc;
const REINIT_AUDIO_KEY: KeyCode = KeyCode::Home;
const CLEAR_STACK_KEY: KeyCode = KeyCode::Delete;
const HOLD_MODIFIER: KeyModifiers = KeyModifiers::SHIFT;

enum Mode {
    Loading,
    Jam,
    Record,
    Save(Vec<f32>),
}

pub struct Tui {
    uauauiua: Uauauiua,
    mode: Mode,
    last_error: Option<anyhow::Error>,
    input: String,
    exiting: bool,
}

impl Default for Tui {
    fn default() -> Self {
        Self {
            uauauiua: Uauauiua::default(),
            mode: Mode::Jam,
            last_error: None,
            input: String::new(),
            exiting: false,
        }
    }
}

impl Tui {
    pub fn handle_result<T>(&mut self, r: Result<T, anyhow::Error>) {
        if let Err(e) = r {
            self.last_error = Some(e);
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) {
        self.load(&mut terminal);

        'main: loop {
            self.draw(&mut terminal);

            self.last_error = None;

            loop {
                if let Event::Key(e) = event::read().expect("should have handled terminal event") {
                    let key = if let KeyCode::Char(c) = e.code {
                        KeyCode::Char(c.to_ascii_lowercase())
                    } else {
                        e.code
                    };
                    let modifiers = e.modifiers;

                    if let KeyEventKind::Press = e.kind {
                        let r = self.handle_key_press(key, modifiers, &mut terminal);
                        self.handle_result(r);
                        break;
                    }
                }
                if self.exiting {
                    break 'main;
                }
            }
        }
    }

    fn load(&mut self, terminal: &mut DefaultTerminal) {
        let current_mode = mem::replace(&mut self.mode, Mode::Loading);
        self.draw(terminal);
        let r = self.uauauiua.load();
        self.handle_result(r);
        self.mode = current_mode;
    }

    fn draw(&self, terminal: &mut DefaultTerminal) {
        terminal
            .draw(|f| f.render_widget(self, f.area()))
            .expect("should have drawn terminal");
    }

    fn handle_key_press(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        terminal: &mut DefaultTerminal,
    ) -> anyhow::Result<()> {
        match (&self.mode, key) {
            (_, key) if key == RELOAD_KEY => {
                self.load(terminal);
            }
            (_, key) if key == REINIT_AUDIO_KEY => {
                self.uauauiua.reinit_audio();
            }
            (_, key) if key == CLEAR_STACK_KEY => {
                self.uauauiua.clear_stack();
            }
            (Mode::Jam, key) if key == RECORD_KEY => {
                self.uauauiua.start_recording()?;
                self.mode = Mode::Record;
            }
            (Mode::Jam, key) if key == EXIT_KEY => {
                self.uauauiua.stop_recording_and_playback()?;
                self.exiting = true;
            }
            (Mode::Record, key) if key == RECORD_KEY => {
                self.mode = Mode::Save(self.uauauiua.stop_recording_and_playback()?);
            }
            (Mode::Jam | Mode::Record, key) if key == STOP_PLAYBACK_KEY => {
                self.uauauiua.stop_playback()?;
            }
            (Mode::Jam | Mode::Record, key) => {
                self.uauauiua
                    .add_to_mixer(key, modifiers.contains(HOLD_MODIFIER))?;
            }
            (Mode::Save(v), KeyCode::Enter) => {
                self.uauauiua
                    .save_recording(v, &mem::take(&mut self.input))?;
                self.mode = Mode::Jam;
            }
            (Mode::Save(_), KeyCode::Char(c)) => {
                self.input.push(c);
            }
            (Mode::Save(_), KeyCode::Backspace) => {
                self.input.pop();
            }
            _ => {}
        }
        Ok(())
    }
}

impl Widget for &Tui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut t = match self.mode {
            Mode::Loading => Text::raw("Loading..."),
            Mode::Jam => Text::raw(format!(
                "Press {RECORD_KEY} to start recording, \
                {RELOAD_KEY} to reload the file, \
                {STOP_PLAYBACK_KEY} to stop playback,\n\
                {REINIT_AUDIO_KEY} to reinitialize audio, \
                {CLEAR_STACK_KEY} to clear the stack, \
                or {EXIT_KEY} to exit\n\n"
            )),
            Mode::Record => Text::raw(format!("Press {RECORD_KEY} to stop recording")),
            Mode::Save(_) => Text::raw(format!(
                "Enter name (leave blank to discard): {}_",
                self.input
            )),
        };
        t += Line::raw(format!(
            "Held sources: [{}]",
            self.uauauiua
                .held_sources()
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        ));
        if let Some(e) = &self.last_error {
            t += Line::raw(format!("Error: {e}"));
        }
        let stack = self.uauauiua.stack();
        if stack.is_empty() {
            t += Line::raw("Stack is empty");
        } else {
            t = t + Text::raw(format!(
                "Stack:\n{}",
                stack
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        t.render(area, buf);
    }
}
