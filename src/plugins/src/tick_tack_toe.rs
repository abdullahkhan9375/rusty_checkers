use crate::EvaluateResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Cell {
    Nought,
    Cross,
    Empty,
}

pub const CELL_COUNT: usize = 9;
pub const GRID_WIDTH: usize = 3;

#[derive(Serialize, Deserialize)]
enum Message {
    GameState {
        player_cell: Cell,
        cells: [Cell; CELL_COUNT],
    },
    FillCell {
        idx: usize,
        content: Cell,
    },
}

impl Message {
    fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn deserialize(msg_str: &str) -> Result<Self, String> {
        match serde_json::from_str::<Message>(msg_str) {
            Ok(m) => Ok(m),
            Err(e) => Err(format!("Failed to decode msg: {e}")),
        }
    }
}

struct GameState {
    cells: [Cell; CELL_COUNT],
}

impl GameState {
    fn new() -> Self {
        Self {
            cells: [Cell::Empty; CELL_COUNT],
        }
    }
}

pub struct ClientState {
    player_cell: Cell,
    game_state: GameState,
}

impl ClientState {
    pub fn new(player_cell: Cell) -> Self {
        Self {
            player_cell,
            game_state: GameState::new(),
        }
    }

    pub fn player_cell(&self) -> Cell {
        self.player_cell
    }

    pub fn recv_msg(&mut self, msg_str: &str) -> Result<(), String> {
        let msg = Message::deserialize(msg_str)?;
        match msg {
            Message::GameState { player_cell, cells } => {
                self.player_cell = player_cell;
                self.game_state.cells = cells;
            }
            Message::FillCell { idx, content } => {
                if idx < CELL_COUNT {
                    self.game_state.cells[idx] = content;
                }
            }
        }
        Ok(())
    }

    pub fn get_cells(&self) -> [Cell; CELL_COUNT] {
        self.game_state.cells
    }

    pub fn request_move_msg(&self, cell_idx: usize) -> String {
        Message::FillCell {
            idx: cell_idx,
            content: self.player_cell,
        }
        .serialize()
    }

    pub fn can_place_at(&self, cell_idx: usize) -> bool {
        self.game_state
            .cells
            .get(cell_idx)
            .is_some_and(|cell| *cell == Cell::Empty)
    }
}

struct Server {
    players: Vec<(String, Cell)>,
    game_state: GameState,
}

impl crate::ServerPlugin for Server {
    fn has_user(&self, username: &str) -> bool {
        self.players.iter().any(|(user, _)| user == username)
    }

    fn get_state_msg_for_user(&mut self, username: &str) -> Result<String, String> {
        let found = self.players.iter().find(|(name, _)| name == username);

        let (_, player_cell) = if let Some(found) = found {
            found
        } else if self.players.len() < 2 {
            let cells = [Cell::Nought, Cell::Cross];
            let player_cell = cells[self.players.len()];
            self.players.push((username.to_string(), player_cell));
            self.players.last().unwrap()
        } else {
            return Err("Game is full".to_string());
        };

        Ok(Message::GameState {
            player_cell: *player_cell,
            cells: self.game_state.cells,
        }
        .serialize())
    }

    fn recv_msg(&mut self, msg_str: &str) -> Result<String, String> {
        let msg = Message::deserialize(msg_str)?;
        match msg {
            Message::FillCell { idx, content } => {
                if idx < CELL_COUNT {
                    self.game_state.cells[idx] = content;
                }
            }
            _ => {}
        }

        Ok(msg_str.to_string())
    }

    fn evaluate_state(&self) -> EvaluateResult {
        EvaluateResult::Running
    }
}

pub fn new_server() -> Box<dyn crate::ServerPlugin> {
    Box::new(Server {
        players: vec![],
        game_state: GameState::new(),
    })
}
