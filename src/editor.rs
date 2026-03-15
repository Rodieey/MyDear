use crate::{
    game_object::GameObjectID,
    level::{data_to_map, load_map, load_measurements, map_to_data, save_map, save_measurements},
    map::Map,
    renderer::{Renderer, ScreenMeasurements},
    vector2::Vector2,
};
use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::time::{Duration, Instant};
use std::{
    io::{self, Write, stdout},
    path::Path,
};

pub const FILE_SELECTIONS: &[&str] = &["New Project", "Open Project"];
pub const OBJECT_EDIT_SELECTIONS: &[&str] = &["Position", "Icon", "Color"];
pub enum EditorState {
    SelectingFile {
        file_selection: usize,
        file_input: String,
        file_message: String,
    },
    Browsing {
        cursor: Vector2,
    },
    EditingObject {
        object_id: GameObjectID,
        edit_selection: usize,
        selected: bool,
    },
}

pub struct Editor {
    pub map: Map,
    pub camera: Vector2,
    pub renderer: Renderer,
    pub state: EditorState,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            map: Map::new(
                Vector2::new(500, 500),
                "#".custom_color(CustomColor::new(0, 255, 0)),
            ),
            camera: Vector2::zero(),
            renderer: Renderer::new(ScreenMeasurements::new(
                Vector2::new(50, 20),
                Vector2::new(5, 3),
                5,
                2,
                2,
                50,
                7,
                9,
                30,
                5,
                1,
                3,
            )),
            state: EditorState::SelectingFile {
                file_selection: 0,
                file_input: "".to_string(),
                file_message: "".to_string(),
            },
        }
    }

    pub fn process_input(&mut self, key: KeyCode) -> bool {
        if key == KeyCode::Char('q') {
            return false;
        }
        match &mut self.state {
            EditorState::SelectingFile {
                file_selection,
                file_input,
                file_message,
            } => match key {
                KeyCode::Left => {
                    let new_index = (*file_selection as i32 - 1 + 2) % 2;
                    *file_selection = new_index as usize;
                }
                KeyCode::Right => {
                    let new_index = (*file_selection as i32 + 1 + 2) % 2;
                    *file_selection = new_index as usize;
                }
                KeyCode::Char(c) => {
                    file_input.push(c);
                }
                KeyCode::Backspace => {
                    file_input.pop();
                }
                KeyCode::Enter => match FILE_SELECTIONS[*file_selection] {
                    "New Project" => {
                        let path = Path::new(file_input.as_str());
                        if path.exists() {
                            let is_empty = path
                                .read_dir()
                                .map(|mut d| d.next().is_none())
                                .unwrap_or(false);
                            if is_empty {
                                let path_str = file_input.clone() + "/";
                                let _ = save_map(&map_to_data(&self.map), path_str.clone());
                                let _ = save_measurements(&self.renderer.measurements, path_str);
                                self.state = EditorState::Browsing {
                                    cursor: Vector2::new(
                                        self.renderer.measurements.screen_size.x / 2,
                                        self.renderer.measurements.screen_size.y / 2,
                                    ),
                                };
                            }
                        } else {
                            if std::fs::create_dir_all(path).is_ok() {
                                let path_str = file_input.clone() + "/";
                                let _ = save_map(&map_to_data(&self.map), path_str.clone());
                                let _ = save_measurements(&self.renderer.measurements, path_str);
                                self.state = EditorState::Browsing {
                                    cursor: Vector2::new(
                                        self.renderer.measurements.screen_size.x / 2,
                                        self.renderer.measurements.screen_size.y / 2,
                                    ),
                                };
                            }
                        }
                    }
                    "Open Project" => {
                        let path_str = file_input.clone() + "/";
                        let map_path = Path::new(file_input.as_str()).join("map.ron");
                        if map_path.is_file() {
                            self.map = data_to_map(&load_map(&(path_str.clone() + "map.ron")));
                            self.renderer =
                                Renderer::new(load_measurements(&(path_str + "measurements.ron")));
                            self.state = EditorState::Browsing {
                                cursor: Vector2::new(
                                    self.renderer.measurements.screen_size.x / 2,
                                    self.renderer.measurements.screen_size.y / 2,
                                ),
                            };
                        } else {
                            *file_message = format!("Filepath {} is not valid", file_input);
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            EditorState::Browsing { cursor } => match key {
                KeyCode::Up => self.camera.y -= 1,
                KeyCode::Down => self.camera.y += 1,
                KeyCode::Left => self.camera.x -= 1,
                KeyCode::Right => self.camera.x += 1,
                KeyCode::Delete => {}
                KeyCode::Char('e') => {
                    let current_pos = self.camera + *cursor;
                    if let Some(object_id) = self.map.positions_hashmap.get(&current_pos) {
                        self.state = EditorState::EditingObject {
                            object_id: *object_id,
                            edit_selection: 0,
                            selected: false,
                        }
                    } else {
                        self.map.insert_object(
                            current_pos,
                            "♥︎".custom_color(CustomColor::new(255, 0, 0)),
                        );
                    }
                }
                KeyCode::Char('s') => {}
                _ => {}
            },
            EditorState::EditingObject {
                object_id,
                edit_selection,
                selected,
            } => match key {
                KeyCode::Up => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*edit_selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x,
                                        self.map.objects.get(object_id).unwrap().position.y - 1,
                                    ),
                                );
                            }
                            _ => {}
                        }
                    } else {
                        let new_index = (*edit_selection as i32 - 1).max(0) as usize;
                        *edit_selection = new_index;
                    }
                }
                KeyCode::Down => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*edit_selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x,
                                        self.map.objects.get(object_id).unwrap().position.y + 1,
                                    ),
                                );
                            }
                            _ => {}
                        }
                    } else {
                        let new_index = (*edit_selection as i32 + 1)
                            .min(OBJECT_EDIT_SELECTIONS.len() as i32 - 1)
                            as usize;
                        *edit_selection = new_index;
                    }
                }
                KeyCode::Left => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*edit_selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x - 1,
                                        self.map.objects.get(object_id).unwrap().position.y,
                                    ),
                                );
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Right => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*edit_selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x + 1,
                                        self.map.objects.get(object_id).unwrap().position.y,
                                    ),
                                );
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Enter => *selected = !*selected,
                KeyCode::Delete => {}
                KeyCode::Esc => {
                    self.state = EditorState::Browsing {
                        cursor: Vector2::new(
                            self.renderer.measurements.screen_size.x / 2,
                            self.renderer.measurements.screen_size.y / 2,
                        ),
                    };
                }
                _ => {}
            },
            _ => {}
        }
        return true;
    }
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    control::set_override(true);

    let mut editor = Editor::new();
    //let mut last_frame = Instant::now();

    loop {
        //let delta_ms = last_frame.elapsed().as_millis() as usize;
        //last_frame = Instant::now();

        execute!(stdout, cursor::MoveTo(0, 0))?;

        editor.renderer.render_editor(&editor);

        stdout.flush()?;

        if event::poll(Duration::from_millis(0))?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            if editor.process_input(code) == false {
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(32));
    }

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}
