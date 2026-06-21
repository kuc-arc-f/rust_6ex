use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr;
use std::env;

use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use serde::Deserialize;

extern "C" {
    fn todo_add(input: *const c_char) -> *mut c_char;
    fn todo_list() -> *mut c_char;
    fn todo_delete(id: i32) -> i32;

    // 文字列処理（受信→加工→返却）
    fn process_string(input: *const c_char) -> *mut c_char;
    
    // メモリ解放
    fn free_string(ptr: *mut c_char);    
}

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::new().run(terminal))
}
#[derive(Debug, Deserialize)]
struct Item {
    id: i32,
    title: String,
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
    /// Vertical scroll offset for the messages pane
    messages_scroll: u16,
    /// Keep the messages pane pinned to the latest message
    follow_messages: bool,    
    /// Whether the app is currently loading
    is_loading: bool,
    /// The time when loading started
    loading_start: Option<std::time::Instant>,    
}

enum InputMode {
    Normal,
    Editing,
}

impl App {
    const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            messages_scroll: 0,
            follow_messages: true,            
            character_index: 0,
            is_loading: false,
            loading_start: None,            
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_message(&mut self) {
        self.follow_messages = true;
        unsafe {
            let mut input_buff = self.input.clone();
            let bool_add = input_buff.starts_with("add");
            let bool_list = input_buff.starts_with("list");
            //println!("add={}", input_buff.starts_with("add"));
            //println!("list={}", input_buff.starts_with("list"));
            if bool_add == true {
                // 削除に成功した場合は Some(削除後) を返す
                if let Some(stripped) = input_buff.strip_prefix("add ") {
                    //println!("stripped={}", stripped);
                    input_buff = stripped.to_string();
                }                
                let c_input = CString::new(input_buff.clone()).unwrap();
                let result_ptr = todo_add(c_input.as_ptr());
                if !result_ptr.is_null() {
                    let result_cstr = CStr::from_ptr(result_ptr);
                    let result_str = result_cstr.to_str().unwrap();
                    let resp = result_str.to_string();
                    // JSON → Vec<Item>
                    /*
                    let items: Vec<Item> = serde_json::from_str(&resp).unwrap();
                    for item in items {
                        let row_str: String = item.id.to_string();
                        let s3 = format!("id={} , {}", row_str, item.title);
                        self.messages.push(s3);
                    }
                    */
                    self.messages.push(resp);
                    self.input.clear();
                    self.reset_cursor();

                    free_string(result_ptr);
                }
            } 
            if bool_list == true {
                self.messages = vec![]; 
                let c_input = self.input.clone();
                let result_ptr = todo_list();
                if !result_ptr.is_null() {
                    let result_cstr = CStr::from_ptr(result_ptr);
                    let result_str = result_cstr.to_str().unwrap();
                    let resp = result_str.to_string();
                    // JSON → Vec<Item>
                    let items: Vec<Item> = serde_json::from_str(&resp).unwrap();
                    for item in items {
                        let row_str: String = item.id.to_string();
                        let s3 = format!("id={} , {}", row_str, item.title);
                        self.messages.push(s3);
                    }
                    //self.messages.push(resp);
                    self.input.clear();
                    self.reset_cursor();

                    free_string(result_ptr);
                }                
                self.input.clear();
                self.reset_cursor();
            }

        }
        self.is_loading = true;
        self.loading_start = Some(std::time::Instant::now());
    }

    fn scroll_messages_up(&mut self) {
        self.follow_messages = false;
        self.messages_scroll = self.messages_scroll.saturating_sub(1);
    }

    fn scroll_messages_down(&mut self) {
        self.follow_messages = false;
        self.messages_scroll = self.messages_scroll.saturating_add(1);
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if self.is_loading {
                if let Some(start) = self.loading_start {
                    if start.elapsed() >= std::time::Duration::from_secs(1) {
                        self.is_loading = false;
                        self.loading_start = None;
                    }
                }
            }

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Some(key) = event::read()?.as_key_press_event() {
                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('e') => {
                                self.input_mode = InputMode::Editing;
                            }
                            KeyCode::Up => self.scroll_messages_up(),
                            KeyCode::Down => self.scroll_messages_down(),
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {}
                        },
                        InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                            KeyCode::Enter => self.submit_message(),
                            KeyCode::Char(to_insert) => self.enter_char(to_insert),
                            KeyCode::Backspace => self.delete_char(),
                            KeyCode::Left => self.move_cursor_left(),
                            KeyCode::Right => self.move_cursor_right(),
                            KeyCode::Up => self.scroll_messages_up(),
                            KeyCode::Down => self.scroll_messages_down(),
                            KeyCode::Esc => self.input_mode = InputMode::Normal,
                            _ => {}
                        },
                        InputMode::Editing => {}
                    }
                }
            }

        }
    }

    fn render(&self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = frame.area().layout(&layout);

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                    "Up/Down".bold(),
                    " to scroll messages.".into(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message".into(),
                    "Up/Down".bold(),
                    " to scroll messages.".into(),                    
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input_text = if self.is_loading {
            "Please wait..."
        } else {
            self.input.as_str()
        };

        let input = Paragraph::new(input_text)
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));

        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[expect(clippy::cast_possible_truncation)]
            InputMode::Editing => {
                if !self.is_loading {
                    frame.set_cursor_position(Position::new(
                        // Draw the cursor at the current position in the input field.
                        // This position can be controlled via the left and right arrow key
                        input_area.x + self.character_index as u16 + 1,
                        // Move one line down, from the border to the input line
                        input_area.y + 1,
                    ));
                }
            }
        }
        let message_lines: Vec<Line> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| Line::from(Span::raw(format!("{m}"))))
            .collect();
        let viewport_height = messages_area.height.saturating_sub(2);
        let max_scroll = message_lines
            .len()
            .saturating_sub(usize::from(viewport_height)) as u16;
        let scroll = if self.follow_messages {
            max_scroll
        } else {
            self.messages_scroll.min(max_scroll)
        };
        let messages = Paragraph::new(Text::from(message_lines))
            .scroll((scroll, 0))
            .block(Block::bordered().title("Messages"));
        frame.render_widget(messages, messages_area);
    }
}
