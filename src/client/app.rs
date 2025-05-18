// rsql.rs - Nathanael "NateNateNate" Thevarajah
// <natenatenat3@protonmail.com> - Refer to the license for more
// information.

#![allow(clippy::cognitive_complexity)]

use crate::client::editor::{EditorCommand, EditorState, map_key_event_to_command};

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    DefaultTerminal,
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn run() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let mut editor = EditorState::default();

    let result = draw(&mut terminal, &mut editor);

    ratatui::restore();
    result
}

fn draw(terminal: &mut DefaultTerminal, editor: &mut EditorState) -> color_eyre::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, editor))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let Some(cmd) = map_key_event_to_command(key) {
                    match cmd {
                        EditorCommand::Quit => break Ok(()),
                        _ => {
                            editor.handle_buffer(cmd);
                        }
                    }
                }
            }
        }
    }
}

/// Our editor is wrapped in Widget::Paragraph. This is what we use to
/// render the "editor look" of our editor. Then, we just wrap it in a
/// fucking block xd.
fn render(frame: &mut Frame, editor: &EditorState) {
    let area = frame.area();
    let height = frame.area().height as usize;
    let offset_y = editor.scroll.1;

    let visible_lines: Vec<Line> = (offset_y..offset_y + height)
        .map(|i| {
            let mut line = editor
                .buffer
                .get_line(i)
                .map(|l| Line::from(l.to_string()))
                .unwrap_or_else(|| Line::from("".to_string()));

            if Some(i) == editor.selected_line {
                line = line.style(Style::default().bg(Color::Blue));
            }

            line
        })
        .collect();

    frame.render_widget(
        Paragraph::new(visible_lines)
            .block(
                Block::default()
                    .title(" Editor ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain),
            )
            .scroll((editor.scroll.1 as u16, 0)),
        frame.area(),
    );

    let cursor_x = editor.cursor.0 as u16 + 1;
    let cursor_y = (editor.cursor.1 - editor.scroll.1) as u16 + 1;
    frame.set_cursor_position((area.x + cursor_x, area.y + cursor_y));
}
