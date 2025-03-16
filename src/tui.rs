use std::{fs, mem};

use crate::{
    recording::{CHANNEL_NUM, SAMPLE_RATE},
    uauauiua::Uauauiua,
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use hound::{SampleFormat, WavSpec, WavWriter};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::Widget,
};
use uiua::Value;

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
    stack: Option<Vec<Value>>,
    input: String,
    exiting: bool,
}

const RECORD_KEY: KeyCode = KeyCode::Enter;
const RELOAD_KEY: KeyCode = KeyCode::Tab;
const STOP_PLAYBACK_KEY: KeyCode = KeyCode::Backspace;
const EXIT_KEY: KeyCode = KeyCode::Esc;

const RECORDINGS_DIR: &str = "recordings";
fn save_recording(recording: &[f32], name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        return Ok(());
    }

    let spec = WavSpec {
        channels: CHANNEL_NUM,
        sample_rate: *SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };

    let _ = fs::create_dir(RECORDINGS_DIR);

    let mut writer = WavWriter::create(format!("{RECORDINGS_DIR}/{name}.wav"), spec)?;

    recording
        .iter()
        .copied()
        .try_for_each(|x| writer.write_sample(x))?;
    writer.finalize()?;

    Ok(())
}

impl Default for Tui {
    fn default() -> Self {
        Self {
            uauauiua: Uauauiua::new(),
            mode: Mode::Jam,
            last_error: None,
            stack: None,
            input: String::new(),
            exiting: false,
        }
    }
}

impl Tui {
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
                        if let Err(e) = self.handle_key_press(key, modifiers, &mut terminal) {
                            self.last_error = Some(e);
                        }
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
        match self.uauauiua.load() {
            Ok(v) => {
                self.stack = Some(v);
            }
            Err(e) => {
                self.stack = None;
                self.last_error = Some(e);
            }
        }
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
        if modifiers.contains(KeyModifiers::CONTROL) && key == KeyCode::Char('c') {
            self.exiting = true;
            return Ok(());
        }

        match (&self.mode, key) {
            (_, key) if key == RELOAD_KEY => {
                self.load(terminal);
            }
            (Mode::Jam, key) if key == RECORD_KEY => {
                self.uauauiua.start_recording();
                self.mode = Mode::Record;
            }
            (Mode::Jam, key) if key == EXIT_KEY => {
                self.uauauiua.stop_recording_and_playback();
                self.exiting = true;
            }
            (Mode::Record, key) if key == RECORD_KEY => {
                self.mode = Mode::Save(self.uauauiua.stop_recording_and_playback());
            }
            (Mode::Jam | Mode::Record, key) if key == STOP_PLAYBACK_KEY => {
                self.uauauiua.stop_playback();
            }
            (Mode::Jam | Mode::Record, key) => {
                self.uauauiua
                    .add_to_mixer(key, modifiers.contains(KeyModifiers::SHIFT))?;
            }
            (Mode::Save(v), KeyCode::Enter) => {
                save_recording(v, &mem::take(&mut self.input))?;
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
        // TODO: more detailed explanations
        let mut t = match self.mode {
            Mode::Loading => Text::raw("Loading..."),
            Mode::Jam => Text::raw(format!(
                "Press {RECORD_KEY} to start recording, {RELOAD_KEY} to reload the file, {STOP_PLAYBACK_KEY} to stop playback, or {EXIT_KEY} to exit"
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
        if let Some(s) = &self.stack {
            if s.is_empty() {
                t += Line::raw("Stack is empty");
            } else {
                t = t + Text::raw(format!(
                    "Stack:\n{}",
                    s.iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            }
        }

        t.render(area, buf);
    }
}
