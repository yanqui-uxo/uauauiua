use std::mem;

use crate::uauauiua::Uauauiua;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{buffer::Buffer, layout::Rect, text::Text, widgets::Widget, DefaultTerminal};

enum UauauiuaMode {
    Start,
    Recording,
    Save(Vec<f32>),
}
pub struct Tui {
    uauauiua: Uauauiua,
    mode: UauauiuaMode,
    input: String,
    exiting: bool,
}

const START_RECORDING_KEY: KeyCode = KeyCode::Char('r');
const STOP_RECORDING_KEY: KeyCode = KeyCode::Esc;
const EXIT_KEY: KeyCode = KeyCode::Esc;

impl Tui {
    pub fn run(mut terminal: DefaultTerminal) {
        let mut uauauiua = Uauauiua::new();

        // TODO: handle error properly
        uauauiua.load().unwrap();
        let mut tui = Self {
            uauauiua,
            mode: UauauiuaMode::Start,
            input: String::new(),
            exiting: false,
        };

        loop {
            terminal
                .draw(|f| f.render_widget(&tui, f.area()))
                .expect("should have drawn terminal");
            if let Event::Key(e) = event::read().expect("should have handled terminal event") {
                if e.kind == KeyEventKind::Press {
                    tui.handle_key_press(e);
                }
            }
            if tui.exiting {
                break;
            }
        }
    }

    fn handle_key_press(&mut self, key_event: KeyEvent) {
        let key = key_event.code;

        if key_event.modifiers.contains(KeyModifiers::CONTROL) && key == KeyCode::Char('c') {
            self.exiting = true;
        }

        match (&self.mode, key) {
            (UauauiuaMode::Start, key) if key == START_RECORDING_KEY => {
                self.uauauiua.start_recording();
                self.mode = UauauiuaMode::Recording;
            }
            (UauauiuaMode::Start, key) if key == EXIT_KEY => {
                self.exiting = true;
            }
            (UauauiuaMode::Recording, key) if key == STOP_RECORDING_KEY => {
                self.mode = UauauiuaMode::Save(self.uauauiua.stop_recording());
            }
            (UauauiuaMode::Recording, key) => {
                let _ = self.uauauiua.add_key_source_to_mixer(key);
            }
            (UauauiuaMode::Save(v), KeyCode::Enter) => {
                self.uauauiua.new_values().insert(
                    mem::take(&mut self.input),
                    v.iter().map(|&x| f64::from(x)).collect(),
                );
                self.mode = UauauiuaMode::Start;
            }
            (UauauiuaMode::Save(_), KeyCode::Char(c)) => {
                self.input.push(c);
            }
            (UauauiuaMode::Save(_), KeyCode::Backspace) => {
                self.input.pop();
            }
            _ => {}
        }
    }
}

impl Widget for &Tui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.mode {
            UauauiuaMode::Start => Text::raw(format!(
                "Press {START_RECORDING_KEY} to start recording or {EXIT_KEY} to exit"
            )),
            UauauiuaMode::Recording => {
                Text::raw(format!("Press {STOP_RECORDING_KEY} to stop recording"))
            }
            UauauiuaMode::Save(_) => Text::raw(format!("Enter name: {}_", self.input)),
        }
        .render(area, buf);
    }
}
