use colored::*;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute, terminal};
use std::io::{Write, stdout};
use std::os::linux::raw::stat;
use std::{i32, io, string, vec};

use crate::game_object::{Dialogue, EventStep, GameEvent, GameObjectID};
use crate::map::*;
use crate::vector2::*;

use kira::{
    AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Tween,
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
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
    // game screen measurements
    pub screen_size: Vector2,
    pub screen_margins: Vector2,
    // padding measurements
    // Dialogue
    /// distance between the game world and the seperators (|) and the distance between seperators and the dialogue text
    pub dialogue_padding: usize,
    /// distance between the top of the screen and the dialogue text
    pub dialogue_text_padding: usize,
    /// distance between the dialogue text and the selections
    pub dialogue_selection_text_padding: usize,
    /// max number of character to render while in dialogue
    pub dialogue_max_character_count: usize,
    // Combat
    // distance between the top of the screen and the characters that is in combat
    pub combat_character_padding_y: usize,
    // distance between the right side of the screen and the first character
    pub combat_character_padding_x: usize,
    // distance between the characters
    pub combat_characters_distance: usize,
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    control::set_override(true);

    let mut game: Game = Game {
        map: Map::new(
            Vector2::new(500, 500),
            String::from("#"),
            CustomColor::new(0, 255, 0),
        ),
        camera: Vector2::zero(),
        audio_manager: generate_audio_manager().expect("Failed to initialize audio"),
        state: GameState::Normal,
        screen_size: Vector2::new(50, 20),
        screen_margins: Vector2::new(5, 3),
        dialogue_padding: 5,
        dialogue_text_padding: 2,
        dialogue_selection_text_padding: 2,
        dialogue_max_character_count: 50,
        combat_character_padding_y: 7,
        combat_character_padding_x: 5,
        combat_characters_distance: 20,
    };

    if let Some(id) = game.map.insert_object(
        Vector2::new(6, 5),
        "♥︎".custom_color(CustomColor::new(255, 0, 0)),
    ) {
        game.map.insert_input_component(id);
        game.map.camera_operator = id;
    }

    if let Some(id) = game.map.insert_object(
        Vector2::new(7, 3),
        "1".custom_color(CustomColor::new(255, 255, 255)),
    ) {
        game.map.insert_moveable_component(id);
    }

    if let Some(id) = game.map.insert_object(
        Vector2::new(3, 3),
        "♥︎".custom_color(CustomColor::new(180, 0, 0)),
    ) {
        let mut events: Vec<EventStep> = Vec::new();
        events.push(EventStep {
            event: crate::game_object::GameEvent::Combat(id),
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
        game.map.insert_event_component(id, events);
    }

    let mut frame_number: i32 = 0;
    loop {
        execute!(stdout, cursor::MoveTo(0, 0))?;

        print!("{}\r\n", frame_number);
        frame_number += 1;

        render(&game);

        stdout.flush()?;

        if event::poll(Duration::from_millis(0))?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            if process_input(code, &mut game) == false {
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(32)); // 30 fps
    }

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}

fn generate_audio_manager() -> Result<AudioManager, Box<dyn std::error::Error>> {
    let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
    Ok(manager)
}

fn process_input(key: KeyCode, game: &mut Game) -> bool {
    if key == KeyCode::Char('q') {
        println!("Quitting... \r");
        return false;
    }
    match &game.state {
        GameState::Normal => match key {
            KeyCode::Up => move_objects(Vector2::new(0, -1), game),
            KeyCode::Down => move_objects(Vector2::new(0, 1), game),
            KeyCode::Left => move_objects(Vector2::new(-1, 0), game),
            KeyCode::Right => move_objects(Vector2::new(1, 0), game),
            KeyCode::Char('e') => {
                let _ = trigger_event_nearby(game);
            }
            _ => {}
        },
        GameState::Dialogue => match key {
            KeyCode::Up => {
                let _ = change_dialogue_selection(game, -1);
            }
            KeyCode::Down => {
                let _ = change_dialogue_selection(game, 1);
            }
            KeyCode::Char('e') => {
                let _ = progress_event(game);
            }
            _ => {}
        },
        _ => {}
    }
    return true;
}

fn change_dialogue_selection(game: &mut Game, direction: i32) -> Option<()> {
    let event = game
        .map
        .event_components
        .get_mut(&game.map.current_event_id)?;

    if let GameEvent::Dialogue(ref mut dialogue) = event.events[event.current_index].event {
        let len = dialogue.selections.len() as i32;
        if len > 0 {
            let new_index = (dialogue.current_selection as i32 + direction + len) % len;
            dialogue.current_selection = new_index as usize;
        }
    }

    return Some(());
}

fn progress_event(game: &mut Game) -> Option<()> {
    let event = game
        .map
        .event_components
        .get_mut(&game.map.current_event_id)?;

    match event.events[event.current_index].requirement {
        crate::game_object::EventCondition::None => 'none: {
            match event.events[event.current_index].event {
                GameEvent::Dialogue(ref dialogue) => {
                    game.state = GameState::Normal;
                    if dialogue.selections_pointing_event.is_empty() {
                        let Some(next_index) = event.events[event.current_index].next_event else {
                            break 'none;
                        };
                        event.current_index = next_index;
                        trigger_event_nearby(game);
                        break 'none;
                    }

                    let Some(next_index) =
                        dialogue.selections_pointing_event[dialogue.current_selection]
                    else {
                        let Some(next_index) = event.events[event.current_index].next_event else {
                            break 'none;
                        };
                        event.current_index = next_index;
                        trigger_event_nearby(game);
                        break 'none;
                    };
                    event.current_index = next_index;
                    trigger_event_nearby(game);
                }
                GameEvent::Combat(ref id) => {}
                GameEvent::TriggerObjectEvent(ref id) => {}
            }
        }
    }

    return Some(());
}

fn trigger_event_nearby(game: &mut Game) -> Option<()> {
    let id = game
        .map
        .get_event_around_this_position(game.map.objects[game.map.camera_operator].position)?;
    let event = game.map.event_components.get_mut(&id)?;

    if event.events[event.current_index].is_triggered && !event.events[event.current_index].repeat {
        return Some(());
    }

    match &event.events[event.current_index].event {
        GameEvent::Dialogue(_text) => {
            game.state = GameState::Dialogue;
            game.map.current_event_id = id;
        }
        GameEvent::Combat(id) => {
            game.state = GameState::Combat;
            game.map.current_event_id = id.clone();
        }
        GameEvent::TriggerObjectEvent(id) => {}
    }

    event.events[event.current_index].is_triggered = true;

    Some(())
}

fn move_objects(direction: Vector2, game: &mut Game) {
    let ids: Vec<usize> = game.map.input_components.keys().cloned().collect();

    for id in ids {
        let next_position: Vector2 = game.map.objects[id].position + direction;

        if game.map.is_out_of_bounds(next_position) {
            continue;
        }

        if let Some(moveable_id) = game.map.positions_hashmap.get(&next_position)
            && game.map.moveable_components.contains_key(moveable_id)
        {
            if game.map.change_object_position(
                *moveable_id,
                Vector2::new(direction.x, direction.y) + next_position,
            ) {
                game.map.change_object_position(id, next_position);
            }
        } else {
            game.map.change_object_position(id, next_position);
        }
    }

    let rel_x = game.map.objects[game.map.camera_operator].position.x - game.camera.x;
    if direction.x < 0 && rel_x < game.screen_margins.x {
        game.camera.x += direction.x;
    } else if direction.x > 0 && rel_x >= game.screen_size.x - game.screen_margins.x {
        game.camera.x += direction.x;
    }

    let rel_y = game.map.objects[game.map.camera_operator].position.y - game.camera.y;
    if direction.y < 0 && rel_y < game.screen_margins.y {
        game.camera.y += direction.y;
    } else if direction.y > 0 && rel_y >= game.screen_size.y - game.screen_margins.y {
        game.camera.y += direction.y;
    }
}

fn render(game: &Game) {
    let capacity = (game.screen_size.x * game.screen_size.y * 15) as usize;
    let mut buffer = String::with_capacity(capacity);

    for y in 0..game.screen_size.y {
        match &game.state {
            GameState::Normal => {
                render_map_line(game, &mut buffer, y);
                buffer.push_str(
                    &" ".repeat(game.dialogue_padding * 2 + 1 + game.dialogue_max_character_count),
                );
            }
            GameState::Combat => {
                render_combat_line(game, &mut buffer, y);
            }
            GameState::Dialogue => {
                render_map_line(game, &mut buffer, y);
                render_dialogue_line(game, &mut buffer, y);
            }
            _ => {}
        }

        buffer.push_str("\r\n");
    }

    print!("{}", buffer);
}

fn render_map_line(game: &Game, buffer: &mut String, y: i32) {
    for x in 0..game.screen_size.x {
        let current_point = get_point_from_world_to_screen(&game.camera, &Vector2::new(x, y));
        if game.map.is_out_of_bounds(current_point) {
            buffer.push_str(" ");
            continue;
        }
        if let Some(id) = game.map.positions_hashmap.get(&current_point) {
            buffer.push_str(&game.map.objects[*id].icon.to_string());
        } else {
            buffer.push_str(&game.map.ground_icon.to_string());
        }
    }
}

fn render_dialogue_line(game: &Game, buffer: &mut String, y: i32) -> Option<()> {
    buffer.push_str(&" ".repeat(game.dialogue_padding));
    buffer.push_str("|");
    buffer.push_str(&" ".repeat(game.dialogue_padding));

    let event = game.map.event_components.get(&game.map.current_event_id)?;
    let GameEvent::Dialogue(dialogue) = &event.events[event.current_index].event else {
        return None;
    };

    let dialogue_line_index = (y - game.dialogue_text_padding as i32) as usize;

    let text_chars = dialogue.text.chars().count();
    let text_line_count =
        (text_chars + game.dialogue_max_character_count - 1) / game.dialogue_max_character_count;

    if dialogue_line_index < text_line_count {
        let start = dialogue_line_index * game.dialogue_max_character_count;
        let line_text: String = dialogue
            .text
            .chars()
            .skip(start)
            .take(game.dialogue_max_character_count)
            .collect();
        buffer.push_str(&line_text);
        buffer.push_str(&" ".repeat(game.dialogue_max_character_count - line_text.chars().count()));
    } else if dialogue_line_index >= text_line_count + game.dialogue_selection_text_padding {
        let selection_line_index =
            dialogue_line_index - text_line_count - game.dialogue_selection_text_padding;
        let Some(selection_text) = dialogue.selections.get(selection_line_index) else {
            buffer.push_str(&" ".repeat(game.dialogue_max_character_count));
            return None;
        };

        if dialogue.current_selection == selection_line_index {
            buffer.push_str(
                &selection_text
                    .custom_color(CustomColor::new(255, 0, 0))
                    .to_string(),
            );
        } else {
            buffer.push_str(&selection_text);
        }
    }

    return None;
}

fn render_combat_line(game: &Game, buffer: &mut String, y: i32) {
    if y == game.combat_character_padding_y as i32 {
        buffer.push_str(&" ".repeat(game.combat_character_padding_x));

        buffer.push_str(&game.map.objects[game.map.camera_operator].icon.to_string());

        buffer.push_str(
            &" ".repeat(game.combat_character_padding_x + game.combat_characters_distance),
        );

        buffer.push_str(&game.map.objects[game.map.current_event_id].icon.to_string());
    }
    buffer.push_str(&" ".repeat(game.screen_size.x as usize));
}

fn get_point_from_world_to_screen(game_origin: &Vector2, screen_coordinate: &Vector2) -> Vector2 {
    return game_origin + screen_coordinate;
}
