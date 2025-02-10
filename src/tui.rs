use std::mem;
use std::sync::mpsc::{channel, Sender};

use crate::uauauiua::Uauauiua;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::Widget,
    DefaultTerminal,
};

enum Mode {
    Start,
    Loading,
    Record,
    Jam,
    Save(Vec<f32>),
}

enum TuiEvent {
    LoadingTransition,
    Reload,
}
pub struct Tui {
    uauauiua: Uauauiua,
    mode: Mode,
    last_error: Option<anyhow::Error>,
    input: String,
    exiting: bool,
}

const RELOAD_KEY: KeyCode = KeyCode::Char('r');
const START_RECORDING_KEY: KeyCode = KeyCode::Enter;
const JAM_KEY: KeyCode = KeyCode::Char('j');
const STOP_KEY: KeyCode = KeyCode::Esc;
const EXIT_KEY: KeyCode = KeyCode::Esc;

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
        let (event_tx, event_rx) = channel();

        self.reload();

        loop {
            self.draw(&mut terminal);
            if let Event::Key(e) = event::read().expect("should have handled terminal event") {
                if e.kind == KeyEventKind::Press {
                    self.handle_key_press(e, &event_tx);
                }
            }

            self.last_error = None;

            while let Ok(e) = event_rx.try_recv() {
                match e {
                    TuiEvent::LoadingTransition => {
                        self.mode = Mode::Loading;
                        self.draw(&mut terminal);
                    }
                    TuiEvent::Reload => {
                        self.reload();
                        self.mode = Mode::Start;
                    }
                }
            }

            if self.exiting {
                break;
            }
        }
    }

    fn reload(&mut self) {
        if let Err(e) = self.uauauiua.load() {
            self.last_error = Some(e);
        }
    }

    fn draw(&self, terminal: &mut DefaultTerminal) {
        terminal
            .draw(|f| f.render_widget(self, f.area()))
            .expect("should have drawn terminal");
    }

    fn handle_key_press(&mut self, key_event: KeyEvent, event_tx: &Sender<TuiEvent>) {
        let key = key_event.code;

        if key_event.modifiers.contains(KeyModifiers::CONTROL) && key == KeyCode::Char('c') {
            self.exiting = true;
        }

        match (&self.mode, key) {
            (Mode::Start, key) if key == RELOAD_KEY => {
                event_tx.send(TuiEvent::LoadingTransition).unwrap();
                event_tx.send(TuiEvent::Reload).unwrap();
            }
            (Mode::Start, key) if key == START_RECORDING_KEY => {
                self.uauauiua.start_recording();
                self.mode = Mode::Record;
            }
            (Mode::Start, key) if key == JAM_KEY => {
                self.mode = Mode::Jam;
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
                // TODO: do something with potential error
                let _ = self.uauauiua.add_key_source_to_mixer(key);
            }
            (Mode::Save(v), KeyCode::Enter) => {
                self.uauauiua.new_values().insert(
                    mem::take(&mut self.input),
                    v.iter().map(|&x| f64::from(x)).collect(),
                );
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
    }
}

impl Widget for &Tui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let l = match self.mode {
            Mode::Start => Line::raw(format!(
                "Press {START_RECORDING_KEY} to start recording, {RELOAD_KEY} to reload the file, {JAM_KEY} to enter jam mode, or {EXIT_KEY} to exit"
            )),
            Mode::Loading => Line::raw("Loading..."),
            Mode::Record => Line::raw(format!("Press {STOP_KEY} to stop recording")),
            Mode::Jam => Line::raw(format!("Press {STOP_KEY} to stop jamming")),
            Mode::Save(_) => Line::raw(format!("Enter name: {}_", self.input)),
        };

        match &self.last_error {
            Some(e) => Text::from(vec![l, Line::raw(e.to_string())]),
            None => Text::from(l),
        }
        .render(area, buf);
    }
}
