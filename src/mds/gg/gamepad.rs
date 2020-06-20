//! Logic for dealing with gamepads
use crate::ggez::event::GamepadId;
use std::collections::HashMap;

pub(super) struct GamepadRegistry {
    ids: Vec<GamepadId>,
    map: HashMap<GamepadId, i64>,
}

impl GamepadRegistry {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn index(&mut self, id: GamepadId) -> i64 {
        match self.map.get(&id) {
            Some(val) => *val,
            None => {
                let index = self.ids.len();
                assert_eq!(index, self.map.len());
                let index = index as u64 as i64;

                self.ids.push(id);
                self.map.insert(id, index);

                index
            }
        }
    }

    #[allow(dead_code)]
    pub fn id(&self, index: i64) -> Option<GamepadId> {
        self.ids.get(index as u64 as usize).cloned()
    }
}
