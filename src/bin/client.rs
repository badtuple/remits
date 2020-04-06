/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages
mod util;
mod messages;
use std::io::{self, Write};

use termion::cursor::Goto;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, Paragraph, SelectableList, Text, Widget};
use tui::Terminal;
use unicode_width::UnicodeWidthStr;

use crate::util::event::{Event, Events};

enum InputMode {
    Normal,
    Editing,
}

/// App holds the state of the application
struct App<'a> {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
    commands: Vec<&'a str>,
    selected: Option<usize>,
}

impl<'a> Default for App<'a> {
    fn default() -> App<'a> {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            commands: vec!["LOG ADD", "MSG ADD", "LOG DEL"],
            selected: Some(0),
        }
    }
}

impl App<'_> {
    fn update_input_screen(&mut self) {
        self.input = match self.commands[self.selected.unwrap_or_default()] {
            "LOG ADD" => {
                let skeleton = messages::AddLogBody::new("");
                serde_json::to_string(&skeleton).unwrap()
            },
            "MSG ADD" => String::new(),
            "LOG DEL" => String::new(),
            _ => String::new(),
        }
    }
}

fn main() -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor().expect("Couldn't hide cursor");
    // Setup event handlers
    let mut events = Events::new();

    // Create default app state
    let mut app = App::default();

    loop {
        // Draw UI
        terminal.draw(|mut f| {
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(f.size());

            let style = Style::default();
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("List"))
                .items(&app.commands)
                .select(app.selected)
                .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                .highlight_symbol(">")
                .render(&mut f, horizontal_chunks[0]);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Percentage(30),
                        Constraint::Percentage(30),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(horizontal_chunks[1]);

            let help_message = match app.input_mode {
                InputMode::Normal => "Press q to exit, e to start editing.",
                InputMode::Editing => "Press Esc to stop editing, Enter to record the message",
            };
            Paragraph::new([Text::raw(help_message)].iter()).render(&mut f, chunks[0]);

            Paragraph::new([Text::raw(&app.input)].iter())
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Input Structure"),
                )
                .render(&mut f, chunks[1]);

            let messages = app
                .messages
                .iter()
                .enumerate()
                .map(|(i, m)| Text::raw(format!("{}: {}", i, m)));
            List::new(messages)
                .block(Block::default().borders(Borders::ALL).title("Messages"))
                .render(&mut f, chunks[2]);

            let bytes = app
                .messages
                .iter()
                .enumerate()
                .map(|(i, m)| Text::raw(format!("{}: {:?}", i, m.as_bytes())));
            List::new(bytes)
                .block(Block::default().borders(Borders::ALL).title("Bytes"))
                .render(&mut f, chunks[3]);
        })?;

        // Put the cursor back inside the input box
        write!(
            terminal.backend_mut(),
            "{}",
            Goto(4 + app.input.width() as u16, 5)
        )?;
        // stdout is buffered, flush it to see the effect immediately when hitting backspace
        io::stdout().flush().ok();

        // Handle input
        match events.next()? {
            Event::Input(input) => match app.input_mode {
                InputMode::Normal => match input {
                    Key::Char('e') => {
                        app.input_mode = InputMode::Editing;
                        events.disable_exit_key();
                    }
                    Key::Char('q') => {
                        break;
                    }
                    Key::Char('\n') => {
                        app.input_mode = InputMode::Editing;
                        events.disable_exit_key();
                        app.update_input_screen();
                        terminal.show_cursor().expect("Couldnt show cursor");
                    }
                    Key::Down => {
                        app.selected = if let Some(selected) = app.selected {
                            if selected >= app.commands.len() - 1 {
                                Some(0)
                            } else {
                                Some(selected + 1)
                            }
                        } else {
                            Some(0)
                        }
                    }
                    Key::Up => {
                        app.selected = if let Some(selected) = app.selected {
                            if selected > 0 {
                                Some(selected - 1)
                            } else {
                                Some(app.commands.len() - 1)
                            }
                        } else {
                            Some(0)
                        }
                    }
                    _ => {}
                },
                InputMode::Editing => match input {
                    Key::Char('\n') => {
                        app.messages.push(app.input.drain(..).collect());
                    }
                    Key::Char(c) => {
                        app.input.push(c);
                    }
                    Key::Backspace => {
                        app.input.pop();
                    }
                    Key::Esc => {
                        app.input_mode = InputMode::Normal;
                        events.enable_exit_key();
                    }
                    _ => {}
                },
            },
            _ => {}
        }
    }
    Ok(())
}
