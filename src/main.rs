mod limited_backend;
mod recording;
mod tui;
mod uauauiua;
mod uiua_extension;

use tui::Tui;

fn main() {
    let terminal = ratatui::init();
    Tui::default().run(terminal);
    ratatui::restore();
}
