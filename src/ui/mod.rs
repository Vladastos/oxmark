use crate::sqlite::SqliteRepository;
use ratatui::{
    crossterm::{
        cursor,
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Flex,
    prelude::{Constraint, CrosstermBackend, Layout},
};
use std::{
    cmp::{max, min},
    io,
    time::Duration,
};
use tui_textarea::TextArea;

mod deleting_popup;
mod fuzzy_filter;
mod list;
mod updating_popup;

use super::Bookmark;

#[derive(Debug)]
pub struct App {
    // The sqlite repository
    sqlite_repository: SqliteRepository,

    // The running state
    running_state: RunningState,

    // The search query
    search_bar: Option<TextArea<'static>>,

    // The selected bookmark
    selected_bookmark: Option<Bookmark>,

    // The list of bookmarks
    bookmarks: Option<Vec<Bookmark>>,

    // The list of filtered bookmarks
    filtered_bookmarks: Option<Vec<Bookmark>>,

    // The index of the selected bookmark
    selection_index: Option<i32>,

    // Running-state specific data
    deleting_state: Option<DeletingState>,
}

#[derive(Debug, Clone)]
struct DeletingState {
    // The bookmark to delete
    bookmark: Option<Bookmark>,

    // The selected option (yes/no)
    selection: Option<bool>,
}

#[derive(Debug, PartialEq)]
pub enum RunningState {
    Listing,
    Deleting,
    Updating,
    Done,
    Exited,
}

impl App {
    pub fn new() -> App {
        let result = SqliteRepository::new();
        if let Err(e) = result {
            println!("{}", e);
            std::process::exit(1);
        }

        App {
            sqlite_repository: result.unwrap(),
            running_state: RunningState::Listing,
            search_bar: Option::from(TextArea::new(vec![])),
            selected_bookmark: None,
            bookmarks: None,
            filtered_bookmarks: None,
            selection_index: Option::from(0),
            deleting_state: None,
        }
    }

    pub fn run(&mut self, print_command: bool) -> Result<(), AppError> {
        let mut terminal = self.init_terminal();

        // Get the list of bookmarks from the sqlite repository
        let bookmark_list_result = self.sqlite_repository.get_all_bookmarks();
        if let Err(_) = bookmark_list_result {
            self.restore(terminal);
            return Err(AppError::InternalError(
                "Error while getting bookmarks".to_string(),
            ));
        }

        self.bookmarks = Option::from(bookmark_list_result.unwrap());

        // Initialize the filtered bookmarks
        self.filtered_bookmarks = self.bookmarks.clone();

        // Initialize the selected bookmark
        self.update_selected_bookmark();

        // Start the main loop
        while self.running_state != RunningState::Done && self.running_state != RunningState::Exited
        {
            let result = terminal.draw(|frame| self.render(frame));
            if let Err(_) = result {
                self.restore(terminal);
                return Err(AppError::InternalError("Error while rendering".to_string()));
            }

            let key_event_result = self.poll_event();
            if let Err(e) = key_event_result {
                self.restore(terminal);
                return Err(e);
            }
            self.handle_event(key_event_result.unwrap());

            // After handling the event, update the filtered bookmarks, the selection index and the selected bookmark
            self.refresh_bookmarks();
            self.update_filtered_bookmarks();
            self.update_selection_index();
            self.update_selected_bookmark();
        }
        self.restore(terminal);

        // Manage the exit state
        self.handle_exit(print_command);

        Ok(())
    }

    //
    // Rendering
    //

    fn render(&self, frame: &mut ratatui::Frame) {
        let popup_area = get_popup_area(frame.area(), 20, 40);

        list::render(self, frame);

        if self.running_state == RunningState::Deleting {
            deleting_popup::render(self, popup_area, frame);
        }

        if self.running_state == RunningState::Updating {
            updating_popup::render(self, popup_area, frame);
        }
    }

    //
    // Event handling
    //

    fn poll_event(&self) -> Result<event::KeyEvent, AppError> {
        if event::poll(Duration::from_millis(250)).is_ok() {
            if let Event::Key(key) = event::read().unwrap() {
                return Ok(key);
            } else {
                let key = event::KeyEvent {
                    code: KeyCode::Null,
                    modifiers: event::KeyModifiers::NONE,
                    state: event::KeyEventState::empty(),
                    kind: event::KeyEventKind::Release,
                };
                return Ok(key);
            }
        } else {
            return Err(AppError::InternalError("Error while polling".to_string()));
        }
    }

    fn handle_event(&mut self, key_event: event::KeyEvent) {
        // If the event is Ctrl+C, exit
        if key_event.code == KeyCode::Char('c') {
            if key_event.modifiers.contains(event::KeyModifiers::CONTROL) {
                self.running_state = RunningState::Exited;
                return;
            }
        }
        match self.running_state {
            RunningState::Listing => {
                self.handle_listing_event(key_event);
            }
            RunningState::Deleting => {
                self.handle_deleting_event(key_event);
            }
            RunningState::Updating => {
                self.handle_updating_event(key_event);
            }
            RunningState::Done => {}
            RunningState::Exited => {}
        }
    }

    fn handle_listing_event(&mut self, key_event: event::KeyEvent) {
        if key_event.code == KeyCode::Esc {
            self.running_state = RunningState::Exited;
            return;
        }

        if key_event.code == KeyCode::Enter {
            self.running_state = RunningState::Done;
            return;
        }

        if key_event.code == KeyCode::Up {
            self.decrement_selection_index();
            return;
        }

        if key_event.code == KeyCode::Down {
            self.increment_selection_index();
            return;
        }

        if key_event.modifiers.contains(event::KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char('d') => {
                    // Set the deleting state
                    if self.selected_bookmark.is_none() {
                        return;
                    }
                    self.running_state = RunningState::Deleting;

                    // Set the state-specific data
                    self.deleting_state = Option::from(DeletingState {
                        bookmark: self.selected_bookmark.clone(),
                        selection: Option::from(false),
                    });
                }
                KeyCode::Char('j') => {
                    // Increment the selection index
                    self.increment_selection_index();
                }
                KeyCode::Char('k') => {
                    // Decrement the selection index
                    self.decrement_selection_index();
                }
                _ => {}
            }
            return;
        }
        // Update the search bar
        self.search_bar.as_mut().unwrap().input(key_event);
    }

    fn handle_deleting_event(&mut self, key_event: event::KeyEvent) {
        if key_event.code == KeyCode::Esc {
            // Reset the deleting state
            self.deleting_state = None;

            self.running_state = RunningState::Listing;
            return;
        }

        match key_event.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.deleting_state.as_mut().unwrap().selection = Option::from(true);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.deleting_state.as_mut().unwrap().selection = Option::from(false);
            }

            KeyCode::Char('y') => {
                // Delete the bookmark
                self.delete_selected_bookmark();
                self.deleting_state = None;
                self.running_state = RunningState::Listing;
            }
            KeyCode::Char('n') => {
                self.deleting_state = None;
                self.running_state = RunningState::Listing;
            }

            KeyCode::Enter => {
                if self.deleting_state.as_ref().unwrap().selection.unwrap() {
                    // Delete the bookmark
                    self.delete_selected_bookmark();
                }
                self.deleting_state = None;
                self.running_state = RunningState::Listing;
            }
            _ => {}
        }
    }

    fn handle_updating_event(&mut self, key_event: event::KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.running_state = RunningState::Listing;
            }
            _ => {}
        }
    }

    fn update_filtered_bookmarks(&mut self) {
        let search_term = self.search_bar.as_ref().unwrap().lines()[0].clone();
        let search_term = search_term.trim().to_string();
        let filtered_bookmarks = fuzzy_filter::get_filtered_bookmarks(
            self.bookmarks.as_ref().unwrap().clone(),
            search_term,
        );
        self.filtered_bookmarks = Option::from(filtered_bookmarks);
    }

    fn update_selection_index(&mut self) {
        // If the selection index is out of bounds, set it to the last index
        let bookmark_count = self.filtered_bookmarks.as_ref().unwrap().len() as i32;
        if self.selection_index.unwrap() < 0 || self.selection_index.unwrap() > bookmark_count - 1 {
            self.selection_index = Option::from(bookmark_count - 1);
        }
    }

    fn update_selected_bookmark(&mut self) {
        self.selected_bookmark = self
            .filtered_bookmarks
            .as_ref()
            .unwrap()
            .get(self.selection_index.unwrap() as usize)
            .cloned();
    }

    fn refresh_bookmarks(&mut self) {
        self.bookmarks = Option::from(self.sqlite_repository.get_all_bookmarks().unwrap());
        self.filtered_bookmarks = self.bookmarks.clone();
    }

    fn delete_selected_bookmark(&mut self) {
        self.sqlite_repository
            .delete_bookmark(
                self.deleting_state
                    .as_ref()
                    .unwrap()
                    .bookmark
                    .as_ref()
                    .unwrap()
                    .id
                    .unwrap(),
            )
            .unwrap();
    }

    fn increment_selection_index(&mut self) {
        let bookmark_count = self.filtered_bookmarks.as_ref().unwrap().len() as i32;
        let selection_index = self.selection_index.unwrap();
        self.selection_index = Option::from(min(selection_index + 1, bookmark_count - 1));
    }

    fn decrement_selection_index(&mut self) {
        let selection_index = self.selection_index.unwrap();
        self.selection_index = Option::from(max(selection_index - 1, 0));
    }

    //
    // Exit handler
    //
    fn handle_exit(&mut self, print_command: bool) {
        if self.selected_bookmark.is_none() {
            return;
        }

        let bookmark_path = self
            .selected_bookmark
            .as_ref()
            .unwrap()
            .path
            .clone()
            .unwrap();

        if self.running_state == RunningState::Done {
            if print_command {
                let is_directory = std::path::Path::new(&bookmark_path).is_dir();

                let is_file = std::path::Path::new(&bookmark_path).is_file();

                let editor = std::env::var("EDITOR").unwrap_or("vi".to_string());

                let command = if is_directory {
                    format!("cd {}", bookmark_path)
                } else if is_file {
                    format!("{} {}", editor, bookmark_path)
                } else {
                    "".to_string()
                };

                println!("{}", command);
            } else {
                println!("{}", bookmark_path);
            }
        }
    }

    //
    // Terminal helpers
    //
    fn init_terminal(&mut self) -> ratatui::Terminal<CrosstermBackend<io::Stderr>> {
        let mut terminal =
            ratatui::Terminal::new(CrosstermBackend::new(std::io::stderr())).unwrap();

        // TODO: handle errors
        enable_raw_mode().unwrap();

        execute!(terminal.backend_mut(), EnterAlternateScreen).unwrap();

        terminal.clear().unwrap();

        execute!(terminal.backend_mut(), cursor::Hide).unwrap();
        terminal
    }

    fn restore(&mut self, mut terminal: ratatui::Terminal<CrosstermBackend<io::Stderr>>) {
        // TODO: handle errors
        disable_raw_mode().unwrap();

        execute!(terminal.backend_mut(), cursor::Show).unwrap();
        execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    }
}

//
// Utils
//

fn get_popup_area(
    area: ratatui::layout::Rect,
    percent_height: u16,
    percent_width: u16,
) -> ratatui::layout::Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_height)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_width)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

//
// AppError
//

#[derive(Debug)]
pub enum AppError {
    InternalError(String),
}

impl AppError {
    pub fn message(&self) -> String {
        match self {
            AppError::InternalError(message) => format!("Internal error: {}", message),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rustmarks - AppError: {}", self.message())
    }
}
