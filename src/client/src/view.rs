use crate::input::{GameInput, Input, Mode};
use crate::model::ChatMessage;
use plugins::{ClientState, PluginType};
use ratatui::style::Stylize;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
    widgets::{Paragraph, Widget},
};

fn render_chat_input(frame: &mut Frame, area: Rect, chat_input: &[String], is_editing: bool) {
    let cursor = if is_editing { "|" } else { "" };

    let input_lines_len = chat_input.len();
    let input_lines: Vec<ListItem> = if chat_input.is_empty() {
        let content = Line::from(Span::raw(cursor));
        vec![ListItem::new(content)]
    } else {
        chat_input
            .iter()
            .enumerate()
            .flat_map(|(i, m)| {
                let mut lines = m.split('\n').map(str::to_string).collect::<Vec<_>>();
                if (i + 1) == input_lines_len
                    && let Some(last) = lines.last_mut()
                {
                    (*last) += cursor;
                }

                lines.into_iter().map(|line| {
                    let content = Line::from(Span::raw(line));
                    ListItem::new(content)
                })
            })
            .collect()
    };

    frame.render_widget(
        List::new(input_lines).block(Block::bordered().title("Chat input")),
        area,
    );
}

fn render_chat_history(frame: &mut Frame, area: Rect, chat_history: &[ChatMessage]) {
    let messages: Vec<ListItem> = chat_history
        .iter()
        .map(|m| {
            let content = Line::from(Span::raw(format!("{}: {}", m.username, m.message)));
            ListItem::new(content)
        })
        .collect();
    let messages = List::new(messages).block(Block::bordered().title("Messages"));

    frame.render_widget(messages, area);
}

struct TickTackToeGrid {
    selected_idx: usize,
    can_place_at_idx: bool,
    plugins: [plugins::tick_tack_toe::Cell; plugins::tick_tack_toe::CELL_COUNT],
}

impl Widget for TickTackToeGrid {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            / 300;

        let col_constraints = (0..3).map(|_| Constraint::Length(9));
        let row_constraints = (0..3).map(|_| Constraint::Length(3));
        let horizontal = Layout::horizontal(col_constraints).spacing(1);
        let vertical = Layout::vertical(row_constraints).spacing(1);

        let cells = area
            .layout_vec(&vertical)
            .into_iter()
            .flat_map(|row| row.layout_vec(&horizontal));

        for (i, (rect, cell)) in cells.zip(self.plugins.iter()).enumerate() {
            use plugins::tick_tack_toe::Cell;

            let c = match cell {
                Cell::Cross => "X",
                Cell::Nought => "O",
                Cell::Empty => " ",
            };

            let mut p = Paragraph::new(c.to_string()).block(Block::bordered());

            if i == self.selected_idx && (secs % 2) == 0 {
                use ratatui::style::Color;
                let colour = if self.can_place_at_idx {
                    Color::White
                } else {
                    Color::Red
                };
                p = p.bg(colour);
            }

            p.render(rect, buf);
        }
    }
}

fn render_game(frame: &mut Frame, area: Rect, game_input: &GameInput, game: &ClientState) {
    let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]);
    let [player_cell_area, game_area] = vertical.areas(area);

    match game {
        ClientState::TickTackToe(ttt) => {
            let selected_idx = if let GameInput::TickTackToe { selected_cell } = game_input {
                *selected_cell
            } else {
                0 // shouldn't happen
            };

            let cell = match ttt.player_cell() {
                plugins::tick_tack_toe::Cell::Nought => 'O',
                plugins::tick_tack_toe::Cell::Cross => 'X',
                plugins::tick_tack_toe::Cell::Empty => ' ',
            };

            let content = Line::from(Span::raw(format!("Your piece: {cell}")));
            frame.render_widget(content, player_cell_area);
            frame.render_widget(
                TickTackToeGrid {
                    plugins: ttt.get_cells(),
                    can_place_at_idx: ttt.can_place_at(selected_idx),
                    selected_idx,
                },
                game_area,
            );
        }
    }
}

fn main_view(input: &Input, frame: &mut Frame) {
    let model = &input.model;

    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Min(10),
        Constraint::Max(5),
        Constraint::Max(1),
    ]);
    let [
        player_area,
        game_area,
        messages_area,
        input_area,
        msg_error_area,
    ] = vertical.areas(frame.area());

    frame.render_widget(
        ratatui::widgets::Paragraph::new(format!("User: {}", input.model.username)),
        player_area,
    );

    frame.render_widget(
        ratatui::widgets::Paragraph::new(model.chat_error.clone()),
        msg_error_area,
    );

    if let Some((_, current_game)) = &model.current_game {
        render_game(frame, game_area, &input.game_input_state, current_game);
    }

    render_chat_input(
        frame,
        input_area,
        &model.chat_input,
        matches!(input.mode, Mode::Chat),
    );
    render_chat_history(frame, messages_area, &model.chat_history);
}

fn new_game_view(current_plugin_type: PluginType, frame: &mut Frame) {
    let all_plugins = PluginType::get_all();

    let list = all_plugins.iter().map(|plugin_type| {
        if *plugin_type == current_plugin_type {
            format!("* {plugin_type}")
        } else {
            plugin_type.to_string()
        }
    });

    let list = List::new(list).block(Block::bordered().title("Create game"));

    frame.render_widget(list, frame.area());
}

fn game_listing_view(input: &Input, selected_game_id: u64, frame: &mut Frame) {
    let list = input
        .model
        .active_games
        .iter()
        .map(|(game_id, plugin_type)| {
            if *game_id == selected_game_id {
                format!("* {game_id}: {plugin_type}")
            } else {
                format!("{game_id}: {plugin_type}")
            }
        });

    let list = List::new(list).block(Block::bordered().title("Game Listing"));

    frame.render_widget(list, frame.area());
}

pub fn view(input: &Input, frame: &mut Frame) {
    match input.mode {
        Mode::NewGame(plugin_type) => new_game_view(plugin_type, frame),
        Mode::GameListing { selected_game_id } => game_listing_view(input, selected_game_id, frame),
        _ => main_view(input, frame),
    }
}
