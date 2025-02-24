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

enum Mode {
    Start,
    Loading,
    Record,
    Jam,
    Save(Vec<f32>),
}

pub struct Tui {
    uauauiua: Uauauiua,
    mode: Mode,
    last_error: Option<anyhow::Error>,
    input: String,
    exiting: bool,
}

const START_RECORDING_KEY: KeyCode = KeyCode::Enter;
const JAM_KEY: KeyCode = KeyCode::Char('j');
const RELOAD_KEY: KeyCode = KeyCode::Char('r');
const STOP_KEY: KeyCode = KeyCode::Esc;
const EXIT_KEY: KeyCode = KeyCode::Esc;

const RECORDINGS_DIR: &str = "recordings";
fn save_recording(recording: &[f32], name: &str) -> anyhow::Result<()> {
    if !name.is_empty() {
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
    }
    Ok(())
}

impl Default for Tui {
    fn default() -> Self {
        Self {
            uauauiua: Uauauiua::new(),
            mode: Mode::Start,
            last_error: None,
            input: String::new(),
            exiting: false,
        }
    }
}

impl Tui {
    pub fn run(mut self, mut terminal: DefaultTerminal) {
        self.reload(&mut terminal);

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

    fn handle_result<T>(&mut self, r: anyhow::Result<T>) {
        if let Err(e) = r {
            self.last_error = Some(e);
        }
    }

    fn reload(&mut self, terminal: &mut DefaultTerminal) {
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
        if modifiers.contains(KeyModifiers::CONTROL) && key == KeyCode::Char('c') {
            self.exiting = true;
            return Ok(());
        }

        match (&self.mode, key) {
            (Mode::Start, key) if key == START_RECORDING_KEY => {
                self.uauauiua.start_recording();
                self.mode = Mode::Record;
            }
            (Mode::Start, key) if key == JAM_KEY => {
                self.mode = Mode::Jam;
            }
            (Mode::Start, key) if key == RELOAD_KEY => {
                self.reload(terminal);
            }
            (Mode::Start, key) if key == EXIT_KEY => {
                self.exiting = true;
            }
            (Mode::Record, key) if key == STOP_KEY => {
                self.mode = Mode::Save(self.uauauiua.stop_recording_and_playback());
            }
            (Mode::Jam, key) if key == STOP_KEY => {
                self.uauauiua.stop_recording_and_playback();
                self.mode = Mode::Start;
            }
            (Mode::Record | Mode::Jam, key) => {
                self.uauauiua
                    .add_to_mixer(key, modifiers.contains(KeyModifiers::SHIFT))?;
            }
            (Mode::Save(v), KeyCode::Enter) => {
                save_recording(v, &mem::take(&mut self.input))?;
                self.mode = Mode::Start;
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
        let l = match self.mode {
            Mode::Start => Line::raw(format!(
                "Press {START_RECORDING_KEY} to start recording, {JAM_KEY} to enter jam mode, {RELOAD_KEY} to reload the file, or {EXIT_KEY} to exit"
            )),
            Mode::Loading => Line::raw("Loading..."),
            Mode::Record => Line::raw(format!("Press {STOP_KEY} to stop recording")),
            Mode::Jam => Line::raw(format!("Press {STOP_KEY} to stop jamming")),
            Mode::Save(_) => Line::raw(format!(
                "Enter name (leave blank to discard): {}_",
                self.input
            )),
        };

        match &self.last_error {
            Some(e) => Text::from(vec![l, Line::raw(format!("Error: {e}"))]),
            None => Text::from(l),
        }
        .render(area, buf);
    }
}
