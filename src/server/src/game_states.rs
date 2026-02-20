use plugins::{PluginType, ServerPlugin};
use std::collections::HashMap;

pub type State = Box<dyn ServerPlugin>;
pub struct GameStates {
    states: HashMap<u64, (PluginType, State)>,
    next_id: u64,
}

impl GameStates {
    pub fn new() -> Self {
        Self {
            states: Default::default(),
            next_id: 0,
        }
    }

    pub fn new_game(&mut self, plugin_type: PluginType) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        self.states
            .insert(id, (plugin_type, plugin_type.new_server()));

        id
    }

    pub fn get_game(&self, id: u64) -> Option<(PluginType, &Box<dyn ServerPlugin>)> {
        self.states
            .get(&id)
            .map(|(plugin_type, plugin)| (*plugin_type, plugin))
    }

    pub fn get_game_mut(&mut self, id: u64) -> Option<(PluginType, &mut Box<dyn ServerPlugin>)> {
        self.states
            .get_mut(&id)
            .map(|(plugin_type, plugin)| (*plugin_type, plugin))
    }

    pub fn list_games(&self, username: Option<&str>) -> Vec<(u64, &dyn ServerPlugin)> {
        self.states
            .iter()
            .filter(|(_, (_, state))| username.is_none_or(|u| state.has_user(u)))
            .map(|(k, (_, v))| (*k, v.as_ref()))
            .collect::<Vec<_>>()
    }
}
