mod game;
mod game_object;
mod map;
mod vector2;
mod renderer;
mod level;
#[cfg(feature = "editor")]
mod editor;

fn main() {
    #[cfg(feature = "editor")]
    if let Err(e) = editor::run() {
        eprintln!("Editor exited with error: {}", e);
        std::process::exit(1);
    }

    #[cfg(not(feature = "editor"))]
    if let Err(e) = game::run() {
        eprintln!("Game exited with error: {}", e);
        std::process::exit(1);
    }
}