mod game;
mod game_object;
mod map;
mod vector2;
mod renderer;

fn main() {
    if let Err(e) = game::run() {
        eprintln!("MyDear exited with error: {}", e);
        std::process::exit(1);
    }
}