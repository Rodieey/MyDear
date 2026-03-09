use colored::*;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute, terminal};
use std::io::{Write, stdout};
use std::{i32, io};

use crate::game_object::{COMBAT_SELECTIONS, Combat, EventStep, GameEvent};
use crate::map::*;
use crate::renderer::Renderer;
use crate::vector2::*;

use colored::control;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Normal,
    Dialogue,
    Combat,
    Menu,
}

pub struct Game {
    pub map: Map,
    pub camera: Vector2,
    pub audio_manager: AudioManager,
    pub state: GameState,
    pub renderer: Renderer,
}

impl Game {
    pub fn new() -> Self {
        Game {
            map: Map::new(
                Vector2::new(500, 500),
                String::from("#"),
                CustomColor::new(0, 255, 0),
            ),
            camera: Vector2::zero(),
            audio_manager: generate_audio_manager().expect("Failed to initialize audio"),
            state: GameState::Normal,
            renderer: Renderer::new(
                Vector2::new(50, 20),
                Vector2::new(5, 3),
                5,
                2,
                2,
                50,
                7,
                9,
                20,
                5,
                1
            ),
        }
    }

    pub fn setup_objects(&mut self) {
        if let Some(id) = self.map.insert_object(
            Vector2::new(6, 5),
            "♥︎".custom_color(CustomColor::new(255, 0, 0)),
        ) {
            self.map.insert_input_component(id);
            self.map.camera_operator = id;
        }

        if let Some(id) = self.map.insert_object(
            Vector2::new(7, 3),
            "1".custom_color(CustomColor::new(255, 255, 255)),
        ) {
            self.map.insert_moveable_component(id);
        }

        if let Some(id) = self.map.insert_object(
            Vector2::new(3, 3),
            "♥︎".custom_color(CustomColor::new(180, 0, 0)),
        ) {
            let mut events: Vec<EventStep> = Vec::new();
            events.push(EventStep {
                event: crate::game_object::GameEvent::Combat(Combat { current_selection: 0 }),
                requirement: crate::game_object::EventCondition::None,
                repeat: true,
                is_triggered: false,
                next_event: None,
            });
            //events.push(EventStep {
            //    event: crate::game_object::GameEvent::Dialogue(Dialogue {
            //        text: "this is a dialogue".to_string(),
            //        selections: vec![],
            //        selections_pointing_event: vec![],
            //        current_selection: 0,
            //    }),
            //    requirement: crate::game_object::EventCondition::None,
            //    repeat: true,
            //    is_triggered: false,
            //    next_event: None,
            //});
            self.map.insert_event_component(id, events);
        }
    }

    pub fn process_input(&mut self, key: KeyCode) -> bool {
        if key == KeyCode::Char('q') {
            println!("Quitting... \r");
            return false;
        }
        match &self.state {
            GameState::Normal => match key {
                KeyCode::Up => self.move_objects(Vector2::new(0, -1)),
                KeyCode::Down => self.move_objects(Vector2::new(0, 1)),
                KeyCode::Left => self.move_objects(Vector2::new(-1, 0)),
                KeyCode::Right => self.move_objects(Vector2::new(1, 0)),
                KeyCode::Char('e') => {
                    let _ = self.trigger_event_nearby();
                }
                _ => {}
            },
            GameState::Dialogue => match key {
                KeyCode::Up => {
                    let _ = self.change_dialogue_selection(-1);
                }
                KeyCode::Down => {
                    let _ = self.change_dialogue_selection(1);
                }
                KeyCode::Char('e') => {
                    let _ = self.progress_event();
                }
                _ => {}
            },
            GameState::Combat => match key {
                KeyCode::Left => {
                    let _ = self.change_combat_selection(-1);
                }
                KeyCode::Right => {
                    let _ = self.change_combat_selection(1);
                }
                KeyCode::Char('e') => {
                    let _ = self.progress_event();
                }
                _ => {}
            },
            _ => {}
        }
        return true;
    }

    fn change_dialogue_selection(&mut self, direction: i32) -> Option<()> {
        let event = self
            .map
            .event_components
            .get_mut(&self.map.current_event_id)?;

        if let GameEvent::Dialogue(ref mut dialogue) = event.events[event.current_index].event {
            let len = dialogue.selections.len() as i32;
            if len > 0 {
                let new_index = (dialogue.current_selection as i32 + direction + len) % len;
                dialogue.current_selection = new_index as usize;
            }
        }

        return Some(());
    }
    fn change_combat_selection(&mut self, direction: i32) -> Option<()> {
        let event = self
            .map
            .event_components
            .get_mut(&self.map.current_event_id)?;

        if let GameEvent::Combat(ref mut combat) = event.events[event.current_index].event {
            let len = COMBAT_SELECTIONS.len() as i32;
            if len > 0 {
                let new_index = (combat.current_selection as i32 + direction + len) % len;
                combat.current_selection = new_index as usize;
            }
        }

        return Some(());
    }

    fn progress_event(&mut self) -> Option<()> {
        let event = self
            .map
            .event_components
            .get_mut(&self.map.current_event_id)?;

        match event.events[event.current_index].requirement {
            crate::game_object::EventCondition::None => 'none: {
                match event.events[event.current_index].event {
                    GameEvent::Dialogue(ref dialogue) => {
                        self.state = GameState::Normal;
                        if dialogue.selections_pointing_event.is_empty() {
                            let Some(next_index) = event.events[event.current_index].next_event
                            else {
                                break 'none;
                            };
                            event.current_index = next_index;
                            self.trigger_event_nearby();
                            break 'none;
                        }

                        let Some(next_index) =
                            dialogue.selections_pointing_event[dialogue.current_selection]
                        else {
                            let Some(next_index) = event.events[event.current_index].next_event
                            else {
                                break 'none;
                            };
                            event.current_index = next_index;
                            self.trigger_event_nearby();
                            break 'none;
                        };
                        event.current_index = next_index;
                        self.trigger_event_nearby();
                    }
                    GameEvent::Combat(ref id) => {}
                    GameEvent::TriggerObjectEvent(ref id) => {}
                }
            }
        }

        return Some(());
    }

    fn trigger_event_nearby(&mut self) -> Option<()> {
        let id = self
            .map
            .get_event_around_this_position(self.map.objects[self.map.camera_operator].position)?;
        let event = self.map.event_components.get_mut(&id)?;

        if event.events[event.current_index].is_triggered
            && !event.events[event.current_index].repeat
        {
            return Some(());
        }

        match &event.events[event.current_index].event {
            GameEvent::Dialogue(_text) => {
                self.state = GameState::Dialogue;
                self.map.current_event_id = id;
            }
            GameEvent::Combat(combat) => {
                self.state = GameState::Combat;
                self.map.current_event_id = id;
            }
            GameEvent::TriggerObjectEvent(id) => {}
        }

        event.events[event.current_index].is_triggered = true;

        Some(())
    }

    fn move_objects(&mut self, direction: Vector2) {
        let ids: Vec<usize> = self.map.input_components.keys().cloned().collect();

        for id in ids {
            let next_position: Vector2 = self.map.objects[id].position + direction;

            if self.map.is_out_of_bounds(next_position) {
                continue;
            }

            if let Some(moveable_id) = self.map.positions_hashmap.get(&next_position)
                && self.map.moveable_components.contains_key(moveable_id)
            {
                if self.map.change_object_position(
                    *moveable_id,
                    Vector2::new(direction.x, direction.y) + next_position,
                ) {
                    self.map.change_object_position(id, next_position);
                }
            } else {
                self.map.change_object_position(id, next_position);
            }
        }

        let rel_x = self.map.objects[self.map.camera_operator].position.x - self.camera.x;
        if direction.x < 0 && rel_x < self.renderer.screen_margins.x {
            self.camera.x += direction.x;
        } else if direction.x > 0
            && rel_x >= self.renderer.screen_size.x - self.renderer.screen_margins.x
        {
            self.camera.x += direction.x;
        }

        let rel_y = self.map.objects[self.map.camera_operator].position.y - self.camera.y;
        if direction.y < 0 && rel_y < self.renderer.screen_margins.y {
            self.camera.y += direction.y;
        } else if direction.y > 0
            && rel_y >= self.renderer.screen_size.y - self.renderer.screen_margins.y
        {
            self.camera.y += direction.y;
        }
    }
}

fn generate_audio_manager() -> Result<AudioManager, Box<dyn std::error::Error>> {
    let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
    Ok(manager)
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    control::set_override(true);

    let mut game = Game::new();
    game.setup_objects();

    let mut frame_number: i32 = 0;
    loop {
        execute!(stdout, cursor::MoveTo(0, 0))?;

        print!("{}\r\n", frame_number);
        frame_number += 1;

        game.renderer.render(&game.map, &game.camera, &game.state);

        stdout.flush()?;

        if event::poll(Duration::from_millis(0))?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            if game.process_input(code) == false {
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(32)); // 30 fps
    }

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}
