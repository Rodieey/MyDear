use colored::*;

use crate::game::GameState;
use crate::game_object::{COMBAT_SELECTIONS, CombatPhase, GameEvent};
use crate::map::Map;
use crate::vector2::Vector2;

pub struct Renderer {
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
    pub combat_message: String,
    /// distance between the top of the screen and the characters that is in combat
    pub combat_character_padding_y: usize,
    /// distance between the right side of the screen and the first character
    pub combat_character_padding_x: usize,
    /// distance between the characters
    pub combat_characters_distance: usize,
    /// distance between the separators (-) and the characters in the y axis
    pub combat_separator_padding_y: usize,
    /// distance between the separators and the combat selections
    pub combat_selection_separator_padding: usize,
    /// distance between the top of the screen and the characters health indicator
    pub combat_health_padding_y: usize,
}

impl Renderer {
    pub fn new(
        screen_size: Vector2,
        screen_margins: Vector2,
        dialogue_padding: usize,
        dialogue_text_padding: usize,
        dialogue_selection_text_padding: usize,
        dialogue_max_character_count: usize,
        combat_character_padding_y: usize,
        combat_character_padding_x: usize,
        combat_characters_distance: usize,
        combat_separator_padding_y: usize,
        combat_selection_separator_padding: usize,
        combat_health_padding_y: usize,
    ) -> Self {
        Renderer {
            screen_size,
            screen_margins,
            dialogue_padding,
            dialogue_text_padding,
            dialogue_selection_text_padding,
            dialogue_max_character_count,
            combat_message: String::from(""),
            combat_character_padding_y,
            combat_character_padding_x,
            combat_characters_distance,
            combat_separator_padding_y,
            combat_selection_separator_padding,
            combat_health_padding_y,
        }
    }

    pub fn render(&self, map: &Map, camera: &Vector2, state: &GameState) {
        let capacity = (self.screen_size.x * self.screen_size.y * 15) as usize;
        let mut buffer = String::with_capacity(capacity);

        for y in 0..self.screen_size.y {
            match state {
                GameState::Normal => {
                    self.render_map_line(map, camera, &mut buffer, y);
                    buffer.push_str(
                        &" ".repeat(
                            self.dialogue_padding * 2 + 1 + self.dialogue_max_character_count,
                        ),
                    );
                }
                GameState::Combat => {
                    self.render_combat_line(map, &mut buffer, y);
                }
                GameState::Dialogue => {
                    self.render_map_line(map, camera, &mut buffer, y);
                    self.render_dialogue_line(map, &mut buffer, y);
                }
                _ => {}
            }

            buffer.push_str("\r\n");
        }

        print!("{}", buffer);
    }

    fn render_map_line(&self, map: &Map, camera: &Vector2, buffer: &mut String, y: i32) {
        for x in 0..self.screen_size.x {
            let current_point = get_point_from_world_to_screen(camera, &Vector2::new(x, y));
            if map.is_out_of_bounds(current_point) {
                buffer.push_str(" ");
                continue;
            }
            if let Some(id) = map.positions_hashmap.get(&current_point) {
                buffer.push_str(&map.objects[*id].icon.to_string());
            } else {
                buffer.push_str(&map.ground_icon.to_string());
            }
        }
    }

    fn render_dialogue_line(&self, map: &Map, buffer: &mut String, y: i32) -> Option<()> {
        buffer.push_str(&" ".repeat(self.dialogue_padding));
        buffer.push_str("|");
        buffer.push_str(&" ".repeat(self.dialogue_padding));

        let Some(event_id) = map.current_event_id else {
            return None;
        };
        let event = map.event_components.get(&event_id)?;
        let GameEvent::Dialogue(dialogue) = &event.events[event.current_index].event else {
            return None;
        };

        let dialogue_line_index = (y - self.dialogue_text_padding as i32) as usize;

        let text_chars = dialogue.text.chars().count();
        let text_line_count = (text_chars + self.dialogue_max_character_count - 1)
            / self.dialogue_max_character_count;

        if dialogue_line_index < text_line_count {
            let start = dialogue_line_index * self.dialogue_max_character_count;
            let line_text: String = dialogue
                .text
                .chars()
                .skip(start)
                .take(self.dialogue_max_character_count)
                .collect();
            buffer.push_str(&line_text);
            buffer.push_str(
                &" ".repeat(self.dialogue_max_character_count - line_text.chars().count()),
            );
        } else if dialogue_line_index >= text_line_count + self.dialogue_selection_text_padding {
            let selection_line_index =
                dialogue_line_index - text_line_count - self.dialogue_selection_text_padding;
            let Some(selection_text) = dialogue.selections.get(selection_line_index) else {
                buffer.push_str(&" ".repeat(self.dialogue_max_character_count));
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

    fn render_combat_line(&self, map: &Map, buffer: &mut String, y: i32) {
        let Some(event_id) = map.current_event_id else {
            return;
        };
        let Some(event) = map.event_components.get(&event_id) else {
            return;
        };
        let GameEvent::Combat(combat) = &event.events[event.current_index].event else {
            return;
        };

        match &combat.current_phase {
            CombatPhase::EnemyAttack(enemy_attack) => {
                let base = self.combat_character_padding_x + self.combat_characters_distance + 1;
                let line_width = self.screen_size.x as usize;

                if y == self.combat_character_padding_y as i32 - 2 {
                    buffer.push_str(&" ".repeat(base));
                    buffer.push_str(&map.objects[event_id].icon.to_string());
                    let used = base + 1;
                    if used < line_width {
                        buffer.push_str(&" ".repeat(line_width - used));
                    }
                    return;
                }

                let player_col =
                    if y == (self.combat_character_padding_y - 1 + combat.player_row) as i32 {
                        Some(self.combat_character_padding_x)
                    } else {
                        None
                    };

                let mut row_projectiles: Vec<_> = enemy_attack
                    .projectiles
                    .iter()
                    .filter(|p| y == (self.combat_character_padding_y - 1 + p.row) as i32)
                    .collect();
                row_projectiles.sort_by(|a, b| b.x.cmp(&a.x));

                let mut last_pos: usize = 0;

                for projectile in &row_projectiles {
                    if projectile.x >= base {
                        continue;
                    }
                    let col = base - projectile.x;
                    if col >= line_width {
                        continue;
                    }

                    if let Some(pcol) = player_col {
                        if pcol >= last_pos && pcol < col {
                            buffer.push_str(&" ".repeat(pcol - last_pos));
                            buffer.push_str(&map.objects[map.camera_operator].icon.to_string());
                            last_pos = pcol + 1;
                        }
                    }

                    if col >= last_pos {
                        buffer.push_str(&" ".repeat(col - last_pos));
                        buffer.push_str(&combat.projectile_icon.to_string());
                        last_pos = col + 1;
                    }
                }

                if let Some(pcol) = player_col {
                    if pcol >= last_pos && pcol < line_width {
                        buffer.push_str(&" ".repeat(pcol - last_pos));
                        buffer.push_str(&map.objects[map.camera_operator].icon.to_string());
                        last_pos = pcol + 1;
                    }
                }

                if last_pos < line_width {
                    buffer.push_str(&" ".repeat(line_width - last_pos));
                }
                return;
            }
            _ => {}
        }

        if y == self.combat_character_padding_y as i32
            && !matches!(
                combat.current_phase,
                CombatPhase::EnemyAttack(_)
            )
        {
            buffer.push_str(&" ".repeat(self.combat_character_padding_x));
            buffer.push_str(&map.objects[map.camera_operator].icon.to_string());
            buffer.push_str(&" ".repeat(self.combat_characters_distance));
            buffer.push_str(&map.objects[event_id].icon.to_string());
        } else if y == (self.combat_character_padding_y + self.combat_separator_padding_y) as i32 {
            buffer.push_str(&"-".repeat(self.screen_size.x as usize));
            return;
        } else if y == (self.combat_health_padding_y) as i32 {
            let player_stats = map.stats_components.get(&map.camera_operator);
            let enemy_stats = map.stats_components.get(&event_id);

            let player_hp_text = format!("hp:{}", player_stats.map_or(0, |s| s.health()));
            let enemy_hp_text = format!("hp:{}", enemy_stats.map_or(0, |s| s.health()));

            let to_custom = |color: &Option<Color>| -> CustomColor {
                match color {
                    Some(Color::TrueColor { r, g, b }) => CustomColor::new(*r, *g, *b),
                    _ => CustomColor::new(255, 255, 255),
                }
            };

            let player_color = to_custom(&map.objects[map.camera_operator].icon.fgcolor);
            let enemy_color = to_custom(&map.objects[event_id].icon.fgcolor);

            let enemy_col = self.combat_character_padding_x + self.combat_characters_distance + 1;
            let used = enemy_col + enemy_hp_text.len();

            buffer.push_str(&" ".repeat(self.combat_character_padding_x));
            buffer.push_str(&player_hp_text.custom_color(player_color).to_string());
            buffer.push_str(
                &" ".repeat(enemy_col - self.combat_character_padding_x - player_hp_text.len()),
            );
            buffer.push_str(&enemy_hp_text.custom_color(enemy_color).to_string());
            buffer.push_str(&" ".repeat(self.screen_size.x as usize - used));
        } else if y
            == (self.combat_character_padding_y
                + self.combat_separator_padding_y
                + self.combat_selection_separator_padding) as i32
        {
            let mut raw_len: usize = 0;

            match &combat.current_phase {
                CombatPhase::PlayerTurn => {
                    let selections_text = COMBAT_SELECTIONS
                        .iter()
                        .enumerate()
                        .map(|(i, selection)| {
                            let text = if i == combat.current_selection {
                                selection
                                    .custom_color(CustomColor::new(255, 0, 0))
                                    .to_string()
                            } else {
                                selection.to_string()
                            };
                            format!("{}  ", text)
                        })
                        .collect::<String>();

                    raw_len = COMBAT_SELECTIONS.iter().map(|s| s.len() + 2).sum::<usize>()
                        + self.combat_message.len();

                    buffer.push_str(&selections_text);
                }
                _ => {}
            }

            buffer.push_str(
                &self
                    .combat_message
                    .custom_color(CustomColor::new(255, 255, 0))
                    .to_string(),
            );
            buffer.push_str(&" ".repeat(self.screen_size.x as usize - raw_len));

            return;
        }
        buffer.push_str(&" ".repeat(self.screen_size.x as usize));
    }
}

fn get_point_from_world_to_screen(game_origin: &Vector2, screen_coordinate: &Vector2) -> Vector2 {
    return game_origin + screen_coordinate;
}
