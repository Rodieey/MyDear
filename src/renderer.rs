use colored::*;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "editor"))]
use crate::game::GameState;
#[cfg(feature = "editor")]
use crate::game_object::GameEvent;
#[cfg(not(feature = "editor"))]
use crate::game_object::{COMBAT_SELECTIONS, CombatPhase, GameEvent};
use crate::map::Map;
use crate::vector2::Vector2;

#[cfg(feature = "editor")]
use crate::editor::{
    COMPONENT_SELECTIONS, Editor, EditorState, FILE_SELECTIONS, OBJECT_EDIT_SELECTIONS,
};

#[derive(Serialize, Deserialize)]
pub struct ScreenMeasurements {
    // game screen measurements
    pub screen_size: Vector2,
    pub screen_margins: Vector2,
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
impl ScreenMeasurements {
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
        ScreenMeasurements {
            screen_size,
            screen_margins,
            dialogue_padding,
            dialogue_text_padding,
            dialogue_selection_text_padding,
            dialogue_max_character_count,
            combat_character_padding_y,
            combat_character_padding_x,
            combat_characters_distance,
            combat_separator_padding_y,
            combat_selection_separator_padding,
            combat_health_padding_y,
        }
    }
}

pub struct Renderer {
    pub measurements: ScreenMeasurements,
    #[cfg(not(feature = "editor"))]
    pub combat_message: String,
    #[cfg(feature = "editor")]
    pub editor_message: String,
    pub line_length: Vec<usize>,
}

impl Renderer {
    pub fn new(measurements: ScreenMeasurements) -> Self {
        let y = measurements.screen_size.y as usize;
        Renderer {
            measurements,
            #[cfg(not(feature = "editor"))]
            combat_message: String::from(""),
            #[cfg(feature = "editor")]
            editor_message: String::from(""),
            line_length: vec![0; y + 1],
        }
    }

    fn pad_line(&self, buffer: &mut String, y: usize, raw_len: usize) {
        if raw_len < self.line_length[y] {
            buffer.push_str(&" ".repeat(self.line_length[y] - raw_len));
        } else if raw_len > self.line_length[y] {
            buffer.push_str(&" ".repeat(raw_len - self.line_length[y]));
        }
    }

    #[cfg(feature = "editor")]
    pub fn set_editor_message(&mut self, message: &str) {
        self.editor_message = message.to_string();
    }

    #[cfg(feature = "editor")]
    pub fn render_editor(&self, editor: &Editor) -> Vec<usize> {
        let mut buffer = String::with_capacity(
            (self.measurements.screen_size.x * self.measurements.screen_size.y * 15) as usize,
        );
        let mut line_lengths = vec![0; (self.measurements.screen_size.y + 1) as usize];

        match &editor.state {
            EditorState::SelectingFile {
                file_selection,
                file_input,
                file_message,
                recent_projects,
                recent_selection,
            } => {
                for y in 0..self.measurements.screen_size.y {
                    let (line, raw_len): (String, usize) = match y as usize {
                        2 => {
                            let mut s = String::new();
                            let mut len = 0;
                            for (i, selection) in FILE_SELECTIONS.iter().enumerate() {
                                if i == *file_selection {
                                    s.push_str(
                                        &selection
                                            .custom_color(CustomColor::new(255, 0, 0))
                                            .to_string(),
                                    );
                                } else {
                                    s.push_str(selection);
                                }
                                s.push_str("  ");
                                len += selection.len() + 2;
                            }
                            (s, len)
                        }
                        4 => {
                            if FILE_SELECTIONS[*file_selection] == "Recent Projects" {
                                (String::new(), 0)
                            } else {
                                let text = format!("location: {}", file_input);
                                let len = "location: ".len() + file_input.len();
                                (text, len)
                            }
                        }
                        5 => {
                            if FILE_SELECTIONS[*file_selection] != "Recent Projects" {
                                let len = file_message.len();
                                (file_message.clone(), len)
                            } else {
                                (String::new(), 0)
                            }
                        }
                        _ => {
                            if FILE_SELECTIONS[*file_selection] == "Recent Projects" && y >= 4 {
                                let list_index = y as usize - 6;
                                if let Some(path) = recent_projects.get(list_index) {
                                    let len = path.len();
                                    if list_index == *recent_selection {
                                        (
                                            path.custom_color(CustomColor::new(255, 0, 0))
                                                .to_string(),
                                            len,
                                        )
                                    } else {
                                        (path.clone(), len)
                                    }
                                } else {
                                    (String::new(), 0)
                                }
                            } else {
                                (String::new(), 0)
                            }
                        }
                    };

                    buffer.push_str(&line);
                    self.pad_line(&mut buffer, y as usize, raw_len);
                    line_lengths.push(raw_len);
                    buffer.push_str("\r\n");
                }
            }
            EditorState::EditingObject {
                object_id,
                selection,
                edit_selection,
                selected,
            } => {
                for y in 0..self.measurements.screen_size.y {
                    self.render_editor_map_line(editor, &mut buffer, y);
                    let mut raw_len = self.measurements.screen_size.x as usize;

                    if (y as usize) < OBJECT_EDIT_SELECTIONS.len() {
                        let sel_str = OBJECT_EDIT_SELECTIONS[y as usize];
                        let selection_text = if y as usize == *selection {
                            sel_str
                                .custom_color(CustomColor::new(
                                    if *selected { 255 } else { 127 },
                                    0,
                                    0,
                                ))
                                .to_string()
                        } else {
                            sel_str.to_string()
                        };
                        buffer.push_str("  ");
                        buffer.push_str(&selection_text);
                        raw_len += 2 + sel_str.len();

                        if *selected && y as usize == *selection {
                            match sel_str {
                                "Position" => {
                                    if let Some(object) = editor.map.objects.get(object_id) {
                                        let pos_text = format!(
                                            "  x:{} y:{}",
                                            object.position.x, object.position.y
                                        );
                                        raw_len += pos_text.len();
                                        buffer.push_str(
                                            &pos_text
                                                .custom_color(CustomColor::new(255, 255, 0))
                                                .to_string(),
                                        );
                                    }
                                }
                                "Color" => {
                                    if let Some(object) = editor.map.objects.get(object_id) {
                                        let color = match object.icon.fgcolor {
                                            Some(Color::TrueColor { r, g, b }) => {
                                                CustomColor::new(r, g, b)
                                            }
                                            _ => CustomColor::new(255, 255, 255),
                                        };

                                        let r_text = format!("  r:{} ", color.r);
                                        let g_text = format!("g:{} ", color.g);
                                        let b_text = format!("b:{}", color.b);
                                        raw_len += r_text.len() + g_text.len() + b_text.len();

                                        buffer.push_str(
                                            &r_text
                                                .custom_color(CustomColor::new(
                                                    255,
                                                    if *edit_selection == 0 { 255 } else { 127 },
                                                    0,
                                                ))
                                                .to_string(),
                                        );
                                        buffer.push_str(
                                            &g_text
                                                .custom_color(CustomColor::new(
                                                    255,
                                                    if *edit_selection == 1 { 255 } else { 127 },
                                                    0,
                                                ))
                                                .to_string(),
                                        );
                                        buffer.push_str(
                                            &b_text
                                                .custom_color(CustomColor::new(
                                                    255,
                                                    if *edit_selection == 2 { 255 } else { 127 },
                                                    0,
                                                ))
                                                .to_string(),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    self.pad_line(&mut buffer, y as usize, raw_len);
                    line_lengths.push(raw_len);
                    buffer.push_str("\r\n");
                }
            }
            EditorState::SelectingComponent {
                object_id,
                selection,
                selected,
            } => {
                for y in 0..self.measurements.screen_size.y {
                    self.render_editor_map_line(editor, &mut buffer, y);
                    let mut raw_len = self.measurements.screen_size.x as usize;

                    if (y as usize) < COMPONENT_SELECTIONS.len() {
                        let sel_str = COMPONENT_SELECTIONS[y as usize];
                        let selection_text = if y as usize == *selection {
                            sel_str
                                .custom_color(CustomColor::new(
                                    if *selected { 255 } else { 127 },
                                    0,
                                    0,
                                ))
                                .to_string()
                        } else {
                            sel_str.to_string()
                        };
                        buffer.push_str("  ");
                        buffer.push_str(&selection_text);
                        raw_len += 2 + sel_str.len();

                        if *selected && y as usize == *selection {
                            match sel_str {
                                _ => {}
                            }
                        }
                    }

                    self.pad_line(&mut buffer, y as usize, raw_len);
                    line_lengths.push(raw_len);
                    buffer.push_str("\r\n");
                }
            }
            _ => {
                for y in 0..self.measurements.screen_size.y {
                    self.render_editor_map_line(editor, &mut buffer, y);
                    let raw_len = self.measurements.screen_size.x as usize;
                    self.pad_line(&mut buffer, y as usize, raw_len);
                    line_lengths.push(raw_len);
                    buffer.push_str("\r\n");
                }
            }
        }

        buffer.push_str(&self.editor_message);
        self.pad_line(
            &mut buffer,
            (self.measurements.screen_size.y) as usize,
            self.editor_message.len(),
        );
        line_lengths.push(self.editor_message.len());
        print!("{}", buffer);
        return line_lengths;
    }

    #[cfg(feature = "editor")]
    fn render_editor_map_line(&self, editor: &Editor, buffer: &mut String, y: i32) {
        let cursor_screen_pos = if let EditorState::Browsing { cursor } = &editor.state {
            Some(cursor)
        } else {
            None
        };

        for x in 0..self.measurements.screen_size.x {
            let current_point = get_point_from_world_to_screen(&editor.camera, &Vector2::new(x, y));

            if editor.map.is_out_of_bounds(current_point) {
                buffer.push_str(" ");
                continue;
            }
            if let Some(id) = editor.map.positions_hashmap.get(&current_point)
                && let Some(object) = editor.map.objects.get(id)
            {
                buffer.push_str(&object.icon.to_string());
            } else if cursor_screen_pos.is_some_and(|c| c.x == x && c.y == y) {
                buffer.push_str(&" ".on_white().to_string());
            } else {
                buffer.push_str(&editor.map.ground_icon.to_string());
            }
        }
    }

    #[cfg(not(feature = "editor"))]
    pub fn render(&self, map: &Map, camera: &Vector2, state: &GameState) {
        let Some(cam) = map.objects.get(&map.camera_operator) else {
            return;
        };
        print!("{}\r\n", cam.position);

        let mut buffer = String::with_capacity(
            (self.measurements.screen_size.x * self.measurements.screen_size.y * 15) as usize,
        );

        for y in 0..self.measurements.screen_size.y {
            match state {
                GameState::Normal => {
                    self.render_map_line(map, camera, &mut buffer, y);
                    buffer.push_str(&" ".repeat(
                        self.measurements.dialogue_padding * 2
                            + 1
                            + self.measurements.dialogue_max_character_count,
                    ));
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
        for x in 0..self.measurements.screen_size.x {
            let current_point = get_point_from_world_to_screen(camera, &Vector2::new(x, y));
            if map.is_out_of_bounds(current_point) {
                buffer.push_str(" ");
                continue;
            }
            if let Some(id) = map.positions_hashmap.get(&current_point)
                && let Some(object) = map.objects.get(id)
            {
                buffer.push_str(&object.icon.to_string());
            } else {
                buffer.push_str(&map.ground_icon.to_string());
            }
        }
    }

    fn render_dialogue_line(&self, map: &Map, buffer: &mut String, y: i32) -> Option<()> {
        buffer.push_str(&" ".repeat(self.measurements.dialogue_padding));
        buffer.push_str("|");
        buffer.push_str(&" ".repeat(self.measurements.dialogue_padding));

        let Some(event_id) = map.current_event_id else {
            return None;
        };
        let event = map.event_components.get(&event_id)?;
        let GameEvent::Dialogue(dialogue) = &event.events[event.current_index].event else {
            return None;
        };

        let dialogue_line_index = (y - self.measurements.dialogue_text_padding as i32) as usize;

        let text_chars = dialogue.text.chars().count();
        let text_line_count = (text_chars + self.measurements.dialogue_max_character_count - 1)
            / self.measurements.dialogue_max_character_count;

        if dialogue_line_index < text_line_count {
            let start = dialogue_line_index * self.measurements.dialogue_max_character_count;
            let line_text: String = dialogue
                .text
                .chars()
                .skip(start)
                .take(self.measurements.dialogue_max_character_count)
                .collect();
            buffer.push_str(&line_text);
            buffer.push_str(&" ".repeat(
                self.measurements.dialogue_max_character_count - line_text.chars().count(),
            ));
        } else if dialogue_line_index
            >= text_line_count + self.measurements.dialogue_selection_text_padding
        {
            let selection_line_index = dialogue_line_index
                - text_line_count
                - self.measurements.dialogue_selection_text_padding;
            let Some(selection_text) = dialogue.selections.get(selection_line_index) else {
                buffer.push_str(&" ".repeat(self.measurements.dialogue_max_character_count));
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

    #[cfg(not(feature = "editor"))]
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
        let Some(player_obj) = map.objects.get(&map.camera_operator) else {
            return;
        };
        let Some(enemy_obj) = map.objects.get(&event_id) else {
            return;
        };

        if y == self.measurements.combat_health_padding_y as i32 {
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

            let player_color = to_custom(&player_obj.icon.fgcolor);
            let enemy_color = to_custom(&enemy_obj.icon.fgcolor);

            let enemy_col = self.measurements.combat_character_padding_x
                + self.measurements.combat_characters_distance
                + 1;
            let used = enemy_col + enemy_hp_text.len();

            buffer.push_str(&" ".repeat(self.measurements.combat_character_padding_x));
            buffer.push_str(&player_hp_text.custom_color(player_color).to_string());
            buffer.push_str(&" ".repeat(
                enemy_col - self.measurements.combat_character_padding_x - player_hp_text.len(),
            ));
            buffer.push_str(&enemy_hp_text.custom_color(enemy_color).to_string());
            buffer.push_str(&" ".repeat(self.measurements.screen_size.x as usize - used));
            return;
        }

        match &combat.current_phase {
            CombatPhase::EnemyAttack(enemy_attack) => {
                let base = self.measurements.combat_character_padding_x
                    + self.measurements.combat_characters_distance
                    + 1;
                let line_width = self.measurements.screen_size.x as usize;

                if y == self.measurements.combat_character_padding_y as i32 - 2 {
                    buffer.push_str(&" ".repeat(base));
                    buffer.push_str(&enemy_obj.icon.to_string());
                    let used = base + 1;
                    if used < line_width {
                        buffer.push_str(&" ".repeat(line_width - used));
                    }
                    return;
                }

                let player_col = if y
                    == (self.measurements.combat_character_padding_y - 1 + combat.player_row) as i32
                {
                    Some(self.measurements.combat_character_padding_x)
                } else {
                    None
                };

                let mut row_projectiles: Vec<_> = enemy_attack
                    .projectiles
                    .iter()
                    .filter(|p| {
                        y == (self.measurements.combat_character_padding_y - 1 + p.row) as i32
                    })
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
                            buffer.push_str(&player_obj.icon.to_string());
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
                        buffer.push_str(&player_obj.icon.to_string());
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

        if y == self.measurements.combat_character_padding_y as i32
            && !matches!(combat.current_phase, CombatPhase::EnemyAttack(_))
        {
            buffer.push_str(&" ".repeat(self.measurements.combat_character_padding_x));
            buffer.push_str(&player_obj.icon.to_string());
            buffer.push_str(&" ".repeat(self.measurements.combat_characters_distance));
            buffer.push_str(&enemy_obj.icon.to_string());
        } else if y
            == (self.measurements.combat_character_padding_y
                + self.measurements.combat_separator_padding_y) as i32
        {
            buffer.push_str(&"-".repeat(self.measurements.screen_size.x as usize));
            return;
        } else if y
            == (self.measurements.combat_character_padding_y
                + self.measurements.combat_separator_padding_y
                + self.measurements.combat_selection_separator_padding) as i32
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
            buffer.push_str(&" ".repeat(self.measurements.screen_size.x as usize - raw_len));

            return;
        }
        buffer.push_str(&" ".repeat(self.measurements.screen_size.x as usize));
    }
}

fn get_point_from_world_to_screen(game_origin: &Vector2, screen_coordinate: &Vector2) -> Vector2 {
    return game_origin + screen_coordinate;
}
