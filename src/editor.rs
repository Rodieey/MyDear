use crate::{
    game_object::{GameObjectID, StatsComponent},
    level::{
        add_recent_project, data_to_map, load_map, load_measurements, load_recent_projects,
        map_to_data, remove_recent_project, save_map, save_measurements,
    },
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
use std::time::Duration;
use std::{
    io::{self, Write, stdout},
    path::Path,
};

pub const OBJECT_EDIT_SELECTIONS: &[&str] = &["Position", "Icon", "Color", "Components"];
pub const FILE_SELECTIONS: &[&str] = &["New Project", "Open Project", "Recent Projects"];
pub const COMPONENT_SELECTIONS: &[&str] = &[
    "MoveableComponent",
    "InputComponent",
    "EventComponent",
    "StatsComponent",
];
pub const STATS_COMPONENT_SELECTIONS: &[&str] =
    &["strength", "agility", "defense", "luck", "max_health"];

pub const BROSWING_MESSAGE: &str = "←↑→↓:Move, e:Insert/Edit object, s:Save, q:Quit";
pub const EDITING_OBJECT_MESSAGE: &str =
    "↑↓:Move selection, ENTER:Select/DeSelect property, DELETE:Delete Object, ESC:Go back";
pub const EDITING_COMPONENT_MESSAGE: &str =
    "↑↓:Move selection, ENTER:Add component, DELETE:Remove component, ESC:Go back";
pub const EDITING_STATS_COMPONENT_MESSAGE: &str = "↑↓:Move selection, ←→:Change value, ESC:Go back";

pub enum EditorState {
    SelectingFile {
        file_selection: usize,
        file_input: String,
        file_message: String,
        recent_projects: Vec<String>,
        recent_selection: usize,
    },
    Browsing {
        cursor: Vector2,
    },
    EditingObject {
        object_id: GameObjectID,
        selection: usize,
        edit_selection: usize,
        selected: bool,
    },
    SelectingComponent {
        object_id: GameObjectID,
        selection: usize,
    },
    EditingEventComponent {
        object_id: GameObjectID,
        selection: usize,
    },
    EditingStatsComponent {
        object_id: GameObjectID,
        selection: usize,
    },
}

pub struct Editor {
    pub map: Map,
    pub camera: Vector2,
    pub renderer: Renderer,
    pub state: EditorState,
    current_folder: String,
    current_map: String,
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
                recent_projects: load_recent_projects().paths,
                recent_selection: 0,
            },
            current_folder: "".to_string(),
            current_map: "".to_string(),
        }
    }

    fn save(&self) {
        let _ = save_map(
            &map_to_data(&self.map),
            &self.current_folder,
            &self.current_map,
        );
        let _ = save_measurements(&self.renderer.measurements, &self.current_folder);
    }

    fn open_project(&mut self, path: &str) -> bool {
        let folder = if path.ends_with('/') {
            path.to_string()
        } else {
            path.to_string() + "/"
        };

        let map_path = folder.clone() + "map.ron";
        let measurements_path = folder.clone() + "measurements.ron";

        if !Path::new(&map_path).is_file() {
            return false;
        }
        if !Path::new(&measurements_path).is_file() {
            return false;
        }

        self.map = data_to_map(&load_map(&map_path));
        self.renderer = Renderer::new(load_measurements(&measurements_path));
        self.current_folder = folder;
        self.current_map = String::from("map.ron");
        add_recent_project(path);
        self.renderer.set_editor_message(BROSWING_MESSAGE);
        self.state = EditorState::Browsing {
            cursor: Vector2::new(
                self.renderer.measurements.screen_size.x / 2,
                self.renderer.measurements.screen_size.y / 2,
            ),
        };
        return true;
    }

    pub fn process_input(&mut self, key: KeyCode) -> bool {
        if key == KeyCode::Char('q') {
            return false;
        }
        if let EditorState::SelectingFile {
            file_selection,
            file_input,
            recent_projects,
            recent_selection,
            ..
        } = &self.state
        {
            if key == KeyCode::Enter {
                if file_input == "" && FILE_SELECTIONS[*file_selection] != "Recent Projects" {
                    return true;
                }
                let selection = FILE_SELECTIONS[*file_selection];
                let input = file_input.clone();
                let recent = recent_projects.get(*recent_selection).cloned();
                // all borrows of self.state dropped here
                match selection {
                    "New Project" => {
                        let path = Path::new(input.as_str());
                        let can_create = if path.exists() {
                            path.read_dir()
                                .map(|mut d| d.next().is_none())
                                .unwrap_or(false)
                        } else {
                            std::fs::create_dir_all(path).is_ok()
                        };
                        if can_create {
                            self.current_folder = input.clone() + "/";
                            add_recent_project(&input);
                            self.save();
                            self.renderer.set_editor_message(BROSWING_MESSAGE);
                            self.state = EditorState::Browsing {
                                cursor: Vector2::new(
                                    self.renderer.measurements.screen_size.x / 2,
                                    self.renderer.measurements.screen_size.y / 2,
                                ),
                            };
                        } else {
                            if let EditorState::SelectingFile { file_message, .. } = &mut self.state
                            {
                                *file_message = format!("Cannot create project at {}", input);
                            }
                        }
                    }
                    "Open Project" => {
                        if !self.open_project(&input) {
                            if let EditorState::SelectingFile { file_message, .. } = &mut self.state
                            {
                                *file_message = format!("Filepath {} is not valid", input);
                            }
                        }
                    }
                    "Recent Projects" => {
                        if let Some(path) = recent {
                            if !self.open_project(&path) {
                                if let EditorState::SelectingFile { file_message, .. } =
                                    &mut self.state
                                {
                                    *file_message = format!("Project at {} no longer exists", path);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                return true;
            }
        }

        match &mut self.state {
            EditorState::SelectingFile {
                file_selection,
                file_input,
                file_message,
                recent_projects,
                recent_selection,
            } => match key {
                KeyCode::Left => {
                    *file_selection =
                        (*file_selection + FILE_SELECTIONS.len() - 1) % FILE_SELECTIONS.len();
                }
                KeyCode::Right => {
                    *file_selection = (*file_selection + 1) % FILE_SELECTIONS.len();
                }
                KeyCode::Up => {
                    if FILE_SELECTIONS[*file_selection] == "Recent Projects"
                        && !recent_projects.is_empty()
                    {
                        *recent_selection =
                            (*recent_selection + recent_projects.len() - 1) % recent_projects.len();
                    }
                }
                KeyCode::Down => {
                    if FILE_SELECTIONS[*file_selection] == "Recent Projects"
                        && !recent_projects.is_empty()
                    {
                        *recent_selection = (*recent_selection + 1) % recent_projects.len();
                    }
                }
                KeyCode::Char(c) => {
                    if FILE_SELECTIONS[*file_selection] != "Recent Projects" {
                        file_input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if FILE_SELECTIONS[*file_selection] != "Recent Projects" {
                        file_input.pop();
                    }
                }
                KeyCode::Delete => {
                    if FILE_SELECTIONS[*file_selection] == "Recent Projects"
                        && !recent_projects.is_empty()
                    {
                        remove_recent_project(&recent_projects[*recent_selection]);
                        self.state = EditorState::SelectingFile {
                            file_selection: *file_selection,
                            file_input: file_input.clone(),
                            file_message: file_message.clone(),
                            recent_projects: load_recent_projects().paths,
                            recent_selection: *recent_selection,
                        };
                    }
                }
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
                        self.renderer.set_editor_message(EDITING_OBJECT_MESSAGE);
                        self.state = EditorState::EditingObject {
                            object_id: *object_id,
                            selection: 0,
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
                KeyCode::Char('s') => {
                    self.save();
                }
                _ => {}
            },
            EditorState::EditingObject {
                object_id,
                selection,
                edit_selection,
                selected,
            } => match key {
                KeyCode::Up => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x,
                                        self.map.objects.get(object_id).unwrap().position.y - 1,
                                    ),
                                );
                            }
                            "Color" => {
                                if let Some(object) = self.map.objects.get_mut(object_id) {
                                    if let Some(Color::TrueColor { r, g, b }) =
                                        &mut object.icon.fgcolor
                                    {
                                        match *edit_selection {
                                            0 => *r = r.wrapping_add(1),
                                            1 => *g = g.wrapping_add(1),
                                            2 => *b = b.wrapping_add(1),
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        *selection = (*selection + OBJECT_EDIT_SELECTIONS.len() - 1)
                            % OBJECT_EDIT_SELECTIONS.len();
                    }
                }
                KeyCode::Down => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x,
                                        self.map.objects.get(object_id).unwrap().position.y + 1,
                                    ),
                                );
                            }
                            "Color" => {
                                if let Some(object) = self.map.objects.get_mut(object_id) {
                                    if let Some(Color::TrueColor { r, g, b }) =
                                        &mut object.icon.fgcolor
                                    {
                                        match *edit_selection {
                                            0 => *r = r.wrapping_sub(1),
                                            1 => *g = g.wrapping_sub(1),
                                            2 => *b = b.wrapping_sub(1),
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        *selection = (*selection + 1) % OBJECT_EDIT_SELECTIONS.len();
                    }
                }
                KeyCode::Left => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x - 1,
                                        self.map.objects.get(object_id).unwrap().position.y,
                                    ),
                                );
                            }
                            "Color" => {
                                *edit_selection = (*edit_selection + 3 - 1) % 3;
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Right => {
                    if *selected {
                        match OBJECT_EDIT_SELECTIONS[*selection] {
                            "Position" => {
                                self.map.change_object_position(
                                    *object_id,
                                    Vector2::new(
                                        self.map.objects.get(object_id).unwrap().position.x + 1,
                                        self.map.objects.get(object_id).unwrap().position.y,
                                    ),
                                );
                            }
                            "Color" => {
                                *edit_selection = (*edit_selection + 1) % 3;
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Enter => {
                    *selected = !*selected;
                    *edit_selection = 0;
                    if OBJECT_EDIT_SELECTIONS[*selection] == "Components" {
                        self.renderer.set_editor_message(EDITING_COMPONENT_MESSAGE);
                        self.state = EditorState::SelectingComponent {
                            object_id: object_id.clone(),
                            selection: 0,
                        }
                    }
                }
                KeyCode::Delete => {
                    self.map.delete_object(*object_id);
                    self.renderer.set_editor_message(BROSWING_MESSAGE);
                    self.state = EditorState::Browsing {
                        cursor: Vector2::new(
                            self.renderer.measurements.screen_size.x / 2,
                            self.renderer.measurements.screen_size.y / 2,
                        ),
                    };
                }
                KeyCode::Esc => {
                    self.renderer.set_editor_message(BROSWING_MESSAGE);
                    self.state = EditorState::Browsing {
                        cursor: Vector2::new(
                            self.renderer.measurements.screen_size.x / 2,
                            self.renderer.measurements.screen_size.y / 2,
                        ),
                    };
                }
                KeyCode::Char(c) => {
                    if *selected && OBJECT_EDIT_SELECTIONS[*selection] == "Icon" {
                        if let Some(object) = self.map.objects.get_mut(object_id) {
                            let color = object.icon.fgcolor.clone();
                            object.icon = c.to_string().custom_color(match color {
                                Some(Color::TrueColor { r, g, b }) => CustomColor::new(r, g, b),
                                _ => CustomColor::new(255, 255, 255),
                            });
                            *selected = false;
                        }
                    }
                }
                _ => {}
            },
            EditorState::SelectingComponent {
                object_id,
                selection,
            } => match key {
                KeyCode::Up => {
                    *selection =
                        (*selection + COMPONENT_SELECTIONS.len() - 1) % COMPONENT_SELECTIONS.len();
                }
                KeyCode::Down => {
                    *selection = (*selection + 1) % COMPONENT_SELECTIONS.len();
                }
                KeyCode::Enter => match COMPONENT_SELECTIONS[*selection] {
                    "MoveableComponent" => {
                        self.map.insert_moveable_component(*object_id);
                    }
                    "InputComponent" => {
                        self.map.insert_input_component(*object_id);
                    }
                    "StatsComponent" => {
                        self.map
                            .insert_stats_component(*object_id, StatsComponent::new(0, 0, 0, 0, 0));
                        self.renderer
                            .set_editor_message(EDITING_STATS_COMPONENT_MESSAGE);
                        self.state = EditorState::EditingStatsComponent {
                            object_id: *object_id,
                            selection: 0,
                        }
                    }
                    _ => {}
                },
                KeyCode::Delete => match COMPONENT_SELECTIONS[*selection] {
                    "MoveableComponent" => {
                        self.map.moveable_components.remove(object_id);
                    }
                    "InputComponent" => {
                        self.map.input_components.remove(object_id);
                    }
                    "StatsComponent" => {
                        self.map.stats_components.remove(object_id);
                    }
                    _ => {}
                },
                KeyCode::Esc => {
                    self.renderer.set_editor_message(EDITING_OBJECT_MESSAGE);
                    self.state = EditorState::EditingObject {
                        object_id: *object_id,
                        selection: 3,
                        edit_selection: 0,
                        selected: false,
                    }
                }
                _ => {}
            },
            EditorState::EditingEventComponent {
                object_id,
                selection,
            } => match key {
                _ => {}
            },
            EditorState::EditingStatsComponent {
                object_id,
                selection,
            } => match key {
                KeyCode::Left => match STATS_COMPONENT_SELECTIONS[*selection] {
                    "strength" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.strength = stats.strength.saturating_sub(1);
                        }
                    }
                    "agility" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.agility = stats.agility.saturating_sub(1);
                        }
                    }
                    "defense" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.defense = stats.defense.saturating_sub(1);
                        }
                    }
                    "luck" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.luck = stats.luck.saturating_sub(1);
                        }
                    }
                    "max_health" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.max_health = stats.max_health.saturating_sub(1);
                        }
                    }
                    _ => {}
                },
                KeyCode::Right => match STATS_COMPONENT_SELECTIONS[*selection] {
                    "strength" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.strength += 1;
                        }
                    }
                    "agility" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.agility += 1;
                        }
                    }
                    "defense" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.defense += 1;
                        }
                    }
                    "luck" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.luck += 1;
                        }
                    }
                    "max_health" => {
                        if let Some(stats) = self.map.stats_components.get_mut(object_id) {
                            stats.max_health += 1;
                        }
                    }
                    _ => {}
                },
                KeyCode::Up => {
                    *selection = (*selection + STATS_COMPONENT_SELECTIONS.len() - 1)
                        % STATS_COMPONENT_SELECTIONS.len();
                }
                KeyCode::Down => {
                    *selection = (*selection + 1) % STATS_COMPONENT_SELECTIONS.len();
                }
                KeyCode::Esc => {
                    self.renderer.set_editor_message(EDITING_COMPONENT_MESSAGE);
                    self.state = EditorState::SelectingComponent {
                        object_id: *object_id,
                        selection: 3,
                    }
                }
                _ => {}
            },
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

        editor.renderer.line_length = editor.renderer.render_editor(&editor).clone();

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
