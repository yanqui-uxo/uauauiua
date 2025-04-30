use std::mem;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use indexmap::IndexSet;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::Widget,
};

use crate::uauauiua::Uauauiua;

const MAIN_RECORD_KEY: KeyCode = KeyCode::Enter;
const SECONDARY_RECORD_KEY: KeyCode = KeyCode::Char('\\');
const RELOAD_KEY: KeyCode = KeyCode::Tab;
const STOP_PLAYBACK_KEY: KeyCode = KeyCode::End;
const EXIT_KEY: KeyCode = KeyCode::Esc;
const REINIT_AUDIO_KEY: KeyCode = KeyCode::Home;
const CLEAR_STACK_KEY: KeyCode = KeyCode::Backspace;
//const CLEAR_RECORDINGS_KEY: KeyCode = KeyCode::Delete;
const HOLD_MODIFIER: KeyModifiers = KeyModifiers::SHIFT;

enum Mode {
    Loading,
    Jam,
    SaveMain(Vec<f32>),
    SaveSecondary(Vec<f32>),
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
    fn draw(&self, terminal: &mut DefaultTerminal) {
        terminal
            .draw(|f| f.render_widget(self, f.area()))
            .expect("should have drawn terminal");
    }

    fn handle_result<T>(&mut self, r: Result<T, anyhow::Error>) {
        if let Err(e) = r {
            self.last_error = Some(e);
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) {
        self.load_uiua(&mut terminal);

        'main: loop {
            self.draw(&mut terminal);

            self.last_error = None;

            loop {
                if let Event::Key(e) = event::read().expect("should have handled terminal event") {
                    let key = e.code;
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

    fn load_uiua(&mut self, terminal: &mut DefaultTerminal) {
        let current_mode = mem::replace(&mut self.mode, Mode::Loading);
        self.draw(terminal);
        let r = self.uauauiua.load();
        self.handle_result(r);
        self.mode = current_mode;
    }

    fn handle_key_press(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        terminal: &mut DefaultTerminal,
    ) -> anyhow::Result<()> {
        let lower_key = if let KeyCode::Char(c) = key {
            KeyCode::Char(c.to_ascii_lowercase())
        } else {
            key
        };
        match (&mut self.mode, key) {
            (Mode::SaveMain(_) | Mode::SaveSecondary(_), key) if key == EXIT_KEY => {
                self.mode = Mode::Jam;
            }
            (Mode::SaveMain(v), KeyCode::Enter) => {
                if self.input.is_empty() {
                    return Ok(());
                }
                let input = mem::take(&mut self.input);
                let recording = mem::take(v);
                self.uauauiua.save_main_recording(&recording, &input)?;
                self.mode = Mode::Jam;
            }
            (Mode::SaveSecondary(v), KeyCode::Enter) => {
                if self.input.is_empty() {
                    return Ok(());
                }
                let input = mem::take(&mut self.input);
                let recording = mem::take(v);
                self.uauauiua.save_secondary_recording(&recording, &input)?;
                self.mode = Mode::Jam;
            }
            (Mode::SaveMain(_) | Mode::SaveSecondary(_), KeyCode::Char(c)) => {
                self.input.push(c);
            }
            (Mode::SaveMain(_) | Mode::SaveSecondary(_), KeyCode::Backspace) => {
                self.input.pop();
            }
            (_, key) if key == RELOAD_KEY => {
                self.load_uiua(terminal);
            }
            (_, key) if key == REINIT_AUDIO_KEY => {
                self.uauauiua.reinit_audio();
            }
            (_, key) if key == CLEAR_STACK_KEY => {
                self.uauauiua.clear_stack();
            }
            /*
            (_, key) if key == CLEAR_RECORDINGS_KEY => {
                self.uauauiua.clear_recordings();
            }
            */
            (Mode::Jam, key) if key == MAIN_RECORD_KEY && self.uauauiua.is_recording_main() => {
                self.mode = Mode::SaveMain(self.uauauiua.stop_main_recording()?);
            }
            (Mode::Jam, key)
                if key == SECONDARY_RECORD_KEY && self.uauauiua.is_recording_secondary() =>
            {
                self.mode = Mode::SaveSecondary(self.uauauiua.stop_secondary_recording()?);
            }
            (_, key) if key == MAIN_RECORD_KEY => {
                self.uauauiua.start_main_recording()?;
            }
            (_, key) if key == SECONDARY_RECORD_KEY => {
                self.uauauiua.start_secondary_recording()?;
            }
            (_, key) if key == EXIT_KEY => {
                self.exiting = true;
                self.uauauiua.stop_main_recording()?;
            }
            (_, key) if key == STOP_PLAYBACK_KEY => {
                self.uauauiua.stop_playback()?;
            }
            (_, _) => {
                self.uauauiua
                    .add_to_mixer(lower_key, modifiers.contains(HOLD_MODIFIER))?;
            }
        }
        Ok(())
    }
}

impl Widget for &Tui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        fn join_set(x: &IndexSet<impl ToString>) -> String {
            x.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        }

        let main_text = Text::raw(format!("Press {MAIN_RECORD_KEY} to stop main recording"));
        let secondary_text = Text::raw(format!(
            "Press {SECONDARY_RECORD_KEY} to stop secondary recording"
        ));
        let mut t = match self.mode {
            Mode::Loading => Text::raw("Loading..."),
            Mode::Jam => {
                let main = self.uauauiua.is_recording_main();
                let secondary = self.uauauiua.is_recording_secondary();

                if !main && !secondary {
                    Text::raw(format!(
                        "Press {MAIN_RECORD_KEY} to start main recording, \
                        {SECONDARY_RECORD_KEY} to start secondary recording,\n\
                        {RELOAD_KEY} to reload the file, \
                        {STOP_PLAYBACK_KEY} to stop playback,\n\
                        {REINIT_AUDIO_KEY} to reinitialize audio, \
                        {CLEAR_STACK_KEY} to clear the stack,\n\
                        or {EXIT_KEY} to exit\n\n"
                    ))
                } else if main && secondary {
                    main_text + secondary_text
                } else if main {
                    main_text
                } else {
                    secondary_text
                }
            }
            Mode::SaveMain(_) | Mode::SaveSecondary(_) => Text::raw(format!(
                "Enter name (press {EXIT_KEY} to discard): {}_",
                self.input
            )),
        };

        t += Line::raw(format!(
            "Defined sources: [{}]",
            join_set(&self.uauauiua.defined_sources())
        ));
        t += Line::raw(format!(
            "Held sources: [{}]",
            join_set(self.uauauiua.held_sources())
        ));
        /*
        t += Line::raw(format!(
            "Recordings: [{}]",
            join_set(&self.uauauiua.secondary_recording_names())
        ));
        */

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
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        t.render(area, buf);
    }
}
