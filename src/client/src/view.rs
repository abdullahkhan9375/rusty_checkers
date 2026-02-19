use crate::model::{self, Model};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
};

pub fn view(model: &Model, frame: &mut Frame) {
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Max(5),
        Constraint::Max(1),
    ]);
    let [help_area, messages_area, input_area, msg_error_area] = vertical.areas(frame.area());

    frame.render_widget(
        ratatui::widgets::Paragraph::new("Help".to_string()),
        help_area,
    );

    frame.render_widget(
        ratatui::widgets::Paragraph::new(model.chat_error.clone()),
        msg_error_area,
    );

    let cursor = if matches!(model.mode, model::Mode::Chat) {
        "|"
    } else {
        ""
    };
    let input_lines_len = model.chat_input.len();
    let input_lines: Vec<ListItem> = if model.chat_input.is_empty() {
        let content = Line::from(Span::raw(cursor));
        vec![ListItem::new(content)]
    } else {
        model
            .chat_input
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
        List::new(input_lines).block(Block::bordered().title("Doda")),
        input_area,
    );

    let messages: Vec<ListItem> = model
        .chat_history
        .iter()
        .map(|m| {
            let content = Line::from(Span::raw(format!("{}: {}", m.username, m.message)));
            ListItem::new(content)
        })
        .collect();
    let messages = List::new(messages).block(Block::bordered().title("Messages"));

    frame.render_widget(
        messages, //ratatui::widgets::Paragraph::new(model.chat_input.clone()),
        messages_area,
    );
}
