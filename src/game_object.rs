use crate::vector2::Vector2;
use colored::*;

pub type GameObjectID = usize;
/// anything with this component should be moved with keyboard input
pub struct InputComponent;
/// this allows an object to move another object with this component
pub struct MoveableComponent;

pub struct Dialogue {
    pub text: String,
    pub selections: Vec<String>,
    /// when a selection is selected the current_index variable in EventComponent will change to the number in the same index in this variable. if it is None the selection wont to anything. if the vector is empty none of them wont to anything
    pub selections_pointing_event: Vec<Option<usize>>,
    pub current_selection: usize
}

pub const COMBAT_SELECTIONS: &[&str] = &["Fight", "Item", "Run"];
pub struct Combat { 
    pub current_selection: usize,
}
pub enum GameEvent {
    Dialogue(Dialogue),
    Combat(Combat),
    TriggerObjectEvent(GameObjectID),
}
pub enum EventCondition {
    None,
}
pub struct EventStep {
    pub event: GameEvent,
    pub requirement: EventCondition,
    pub repeat: bool,
    pub is_triggered: bool,
    /// if None it wont switch to any other event, if not will switch the event on that index (if it is a dialogue with selections that points to an event this value will not be used)
    pub next_event: Option<usize>,
}
pub struct EventComponent {
    pub events: Vec<EventStep>,
    pub current_index: usize,
}
pub struct GameObject {
    pub id: GameObjectID,
    pub icon: ColoredString,
    /// ALWAYS CHANGE POSITION FROM MAP FUNCTION, when the position changed, the hashmap on the map struct that point to the id should be changed, with that we can see if we should render the object
    pub position: Vector2,
}
