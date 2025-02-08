use std::mem;

use crate::uauauiua::Uauauiua;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Text},
    widgets::Widget,
    DefaultTerminal,
};

enum UauauiuaMode {
    Start,
    Record,
    Jam,
    Save(Vec<f32>),
}
pub struct Tui {
    uauauiua: Uauauiua,
    mode: UauauiuaMode,
    last_error: Option<anyhow::Error>,
    input: String,
    exiting: bool,
}

const RELOAD_KEY: KeyCode = KeyCode::Char('r');
const START_RECORDING_KEY: KeyCode = KeyCode::Enter;
const JAM_KEY: KeyCode = KeyCode::Char('j');
const STOP_KEY: KeyCode = KeyCode::Esc;
const EXIT_KEY: KeyCode = KeyCode::Esc;

impl Tui {
    fn reload(&mut self) {
        if let Err(e) = self.uauauiua.load() {
            self.last_error = Some(e);
        }
    }

    pub fn run(mut terminal: DefaultTerminal) {
        let mut tui = Self {
            uauauiua: Uauauiua::new(),
            mode: UauauiuaMode::Start,
            last_error: None,
            input: String::new(),
            exiting: false,
        };
        tui.reload();

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
            (UauauiuaMode::Start, key) if key == RELOAD_KEY => {
                self.reload();
            }
            (UauauiuaMode::Start, key) if key == START_RECORDING_KEY => {
                self.uauauiua.start_recording();
                self.mode = UauauiuaMode::Record;
            }
            (UauauiuaMode::Start, key) if key == JAM_KEY => {
                self.mode = UauauiuaMode::Jam;
            }
            (UauauiuaMode::Start, key) if key == EXIT_KEY => {
                self.exiting = true;
            }
            (UauauiuaMode::Record, key) if key == STOP_KEY => {
                self.mode = UauauiuaMode::Save(self.uauauiua.stop_recording());
            }
            (UauauiuaMode::Jam, key) if key == STOP_KEY => {
                self.mode = UauauiuaMode::Start;
            }
            (UauauiuaMode::Record | UauauiuaMode::Jam, key) => {
                // TODO: do something with potential error
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
        let l = match self.mode {
            UauauiuaMode::Start => Line::raw(format!(
                "Press {START_RECORDING_KEY} to start recording, {RELOAD_KEY} to reload the file, or {EXIT_KEY} to exit"
            )),
            UauauiuaMode::Record => Line::raw(format!("Press {STOP_KEY} to stop recording")),
            UauauiuaMode::Jam => Line::raw(format!("Press {STOP_KEY} to stop jamming")),
            UauauiuaMode::Save(_) => Line::raw(format!("Enter name: {}_", self.input)),
        };

        match &self.last_error {
            Some(e) => Text::from(vec![l, Line::raw(e.to_string())]),
            None => Text::from(l),
        }
        .render(area, buf);
    }
}
