use super::Bookmark;
use crate::ui::App;

use ratatui::layout::{Flex, Layout, Offset};
use ratatui::prelude::Stylize;
use ratatui::prelude::{Constraint, Margin};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, ListItem, Paragraph};

const DEFAULT_BLOCK: ratatui::widgets::Block =
    ratatui::widgets::Block::bordered().border_type(ratatui::widgets::BorderType::Rounded);
const TITLE: &str = "Rustmarks";
const BOOKMARK_TITLE_WIDTH: usize = 20;

pub fn render(app: &App, frame: &mut ratatui::Frame) {
    let bookmarks_vec = app.filtered_bookmarks.clone().unwrap_or_else(Vec::new);

    // Create the elements
    let search_bar = app.search_bar.clone().unwrap();

    let list_items = bookmarks_vec
        .iter()
        .map(|bookmark| {
            bookmark.to_list_item(
                app,
                bookmarks_vec.iter().position(|b| b == bookmark).unwrap(),
            )
        })
        .collect::<Vec<ListItem>>();

    let list = ratatui::widgets::List::new(list_items);

    // Create the layout areas

    let layout_areas = get_layout_areas(frame);

    // Render the background
    frame.render_widget(DEFAULT_BLOCK, layout_areas.main_area);

    // Render the title
    frame.render_widget(Paragraph::new(TITLE).centered(), layout_areas.title_area);

    // Render the search input block
    frame.render_widget(DEFAULT_BLOCK, layout_areas.search_area);

    // Render the search bar
    frame.render_widget(
        &search_bar,
        layout_areas.search_area.inner(Margin::new(3, 1)),
    );

    // Render the list
    frame.render_widget(list.block(DEFAULT_BLOCK), layout_areas.list_area);

    // Render the preview
    render_preview(app, layout_areas.preview_area, frame);

    // Render the help
    render_help(app, layout_areas.help_area, frame);
}

fn get_layout_areas(frame: &ratatui::Frame) -> LayoutAreas {
    let main_area = frame.area().inner(Margin::new(2, 0));
    let vertical_flex = Layout::vertical([
        Constraint::Max(3),
        Constraint::Max(3),
        Constraint::Fill(1),
        Constraint::Max(1),
    ])
    .flex(Flex::Center);

    let [title_area, search_area, list_area, help_area] =
        vertical_flex.areas(main_area.inner(Margin::new(3, 1)));

    let list_horizontal_flex =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .flex(Flex::Center);
    let [list_area, preview_area] = list_horizontal_flex.areas(list_area);

    LayoutAreas {
        main_area,
        title_area,
        search_area,
        list_area,
        preview_area,
        help_area,
    }
}

//
// Preview
//

fn render_preview(
    app: &App,
    total_preview_area: ratatui::layout::Rect,
    frame: &mut ratatui::Frame,
) {
    let (title_area, preview_area) = get_preview_areas(total_preview_area);

    frame.render_widget(DEFAULT_BLOCK, total_preview_area);

    // Render the title
    let bookmark_name = app
        .selected_bookmark
        .clone()
        .unwrap_or(Bookmark::default())
        .name
        .clone()
        .unwrap_or("".to_string());
    frame.render_widget(Paragraph::new(bookmark_name).centered(), title_area);

    // Render the preview
    let preview_widget = get_bookmark_preview(app);
    frame.render_widget(preview_widget, preview_area);
}

fn get_bookmark_preview(app: &App) -> ratatui::widgets::Paragraph {
    if app.selected_bookmark.clone().is_none() {
        return ratatui::widgets::Paragraph::new("No bookmark selected")
            .style(Style::default().fg(Color::LightBlue))
            .centered();
    }

    let bookmark_path_string = app
        .selected_bookmark
        .clone()
        .unwrap_or(Bookmark::default())
        .path
        .clone()
        .unwrap_or("".to_string());

    let bookmark_path = std::path::Path::new(&bookmark_path_string);
    if !bookmark_path.exists() {
        return ratatui::widgets::Paragraph::new("Path does not exist")
            .style(Style::default().fg(Color::Red))
            .centered();
    }
    if bookmark_path.is_file() {
        return ratatui::widgets::Paragraph::new("File preview not yet supported")
            .style(Style::default().fg(Color::Blue))
            .centered();
    }

    // Directory preview

    // Get the directory contents

    let preview = get_bookmark_directory_preview(app);

    // Add the block style
    preview.block(
        Block::default()
            .style(Style::default().fg(Color::White))
            .borders(Borders::TOP),
    )
}

fn get_bookmark_directory_preview(app: &App) -> ratatui::widgets::Paragraph {
    let selected_bookmark = app.selected_bookmark.clone().unwrap_or(Bookmark::default());

    let bookmark_path_string = selected_bookmark.path.clone().unwrap_or("".to_string());

    let bookmark_path = std::path::Path::new(&bookmark_path_string);

    let directory_contents = std::fs::read_dir(bookmark_path)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| DirectoryPreviewEntry {
            _path: entry.path(),
            name: entry.file_name().to_string_lossy().to_string(),
            is_file: entry.path().is_file(),
            is_dir: entry.path().is_dir(),
            is_last: false,
        })
        .collect::<Vec<DirectoryPreviewEntry>>();

    // Filter out the hidden files and directories
    let mut filtered_directory_contents: Vec<DirectoryPreviewEntry> = directory_contents
        .iter()
        .filter(|entry| !entry.name.starts_with("."))
        .cloned()
        .collect();

    // Sort the directory contents by name
    filtered_directory_contents.sort_by(|a, b| a.name.cmp(&b.name));

    // Sort the directory contents by file type
    filtered_directory_contents.sort_by(|a, b| a.is_file.cmp(&b.is_file));

    // Set the last entry to be marked as last
    if let Some(last_entry) = filtered_directory_contents.last_mut() {
        last_entry.is_last = true;
    }

    let directory_contents_lines = filtered_directory_contents
        .iter()
        .map(|entry| get_preview_line(entry.to_owned()))
        .collect::<Vec<Line>>();

    let mut lines = vec![get_path_line(bookmark_path_string.clone())];

    lines.extend(directory_contents_lines);

    return ratatui::widgets::Paragraph::new(lines);
}

// Help
fn render_help(app: &App, total_help_area: ratatui::layout::Rect, frame: &mut ratatui::Frame) {
    let paragraph_text = match app.running_state {
        crate::ui::RunningState::Listing => "[Ctrl+k] : move up | [Ctrl+j] : move down | [Ctrl+d] : delete | [Esc] : exit | [Enter] : select",
        crate::ui::RunningState::Deleting => "[Y] : delete | [N / Esc] : cancel | [h/l] : Move selection | [Enter] : select",
        crate::ui::RunningState::Updating => "[Esc] : cancel",
        _ => "Press enter to exit the application", 
    };
    let paragraph = Paragraph::new(paragraph_text).style(Style::default().fg(Color::Blue));
    frame.render_widget(paragraph, total_help_area);
}

#[derive(Debug, Clone)]
struct DirectoryPreviewEntry {
    _path: std::path::PathBuf,
    name: String,
    is_file: bool,
    is_dir: bool,
    is_last: bool,
}

fn get_preview_line(entry: DirectoryPreviewEntry) -> Line<'static> {
    let icon = if entry.is_file {
        ""
    } else if entry.is_dir {
        ""
    } else {
        ""
    };

    let tree_symbol = if entry.is_last { "└─" } else { "├─" };

    let style = if entry.is_dir {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::White)
    };

    return Line::from(vec![
        Span::styled(tree_symbol, Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" {} {}", icon, entry.name), style),
    ]);
}

fn get_path_line(path_string: String) -> Line<'static> {
    return Line::from(vec![Span::styled(
        format!("{}", path_string),
        Style::default().fg(Color::DarkGray),
    )]);
}
fn get_preview_areas(
    preview_area: ratatui::layout::Rect,
) -> (ratatui::layout::Rect, ratatui::layout::Rect) {
    let main_area = preview_area;
    let vertical_flex =
        Layout::vertical([Constraint::Max(2), Constraint::Fill(1)]).flex(Flex::Center);

    let [title_area, preview_area] = vertical_flex.areas(main_area);

    return (
        title_area.offset(Offset { x: 0, y: 1 }),
        preview_area.inner(Margin::new(1, 1)),
    );
}

struct LayoutAreas {
    main_area: ratatui::layout::Rect,
    title_area: ratatui::layout::Rect,
    search_area: ratatui::layout::Rect,
    list_area: ratatui::layout::Rect,
    preview_area: ratatui::layout::Rect,
    help_area: ratatui::layout::Rect,
}

impl Bookmark {
    fn to_list_item(&self, app: &App, list_item_index: usize) -> ListItem<'_> {
        let is_selected = app.selection_index.unwrap_or(0) == list_item_index as i32;

        let is_directory =
            std::path::Path::new(&self.path.clone().unwrap_or("".to_string())).is_dir();

        let bookmark_name = self.name.clone().unwrap_or("<No name>".to_string());

        let decoration_span = if is_selected {
            Span::styled("> ", Style::default().fg(Color::Green))
        } else {
            Span::raw("  ")
        };
        let icon_span = if is_directory {
            Span::styled(" ", Style::default().fg(Color::Blue))
        } else {
            Span::styled(" ", Style::default().fg(Color::White))
        };

        let bookmark_name_span = if is_selected {
            Span::styled(
                bookmark_name.clone(),
                Style::default().fg(Color::Green).underlined(),
            )
        } else {
            Span::raw(bookmark_name.clone())
        };

        let bookmark_description_space =
            if self.name.clone().unwrap_or("".to_string()).len() < BOOKMARK_TITLE_WIDTH {
                " ".repeat(BOOKMARK_TITLE_WIDTH - bookmark_name.len())
            } else {
                "".to_string()
            };
        let bookmark_description_string = format!(
            "{}{}",
            bookmark_description_space,
            self.description.clone().unwrap_or("".to_string())
        );
        let bookmark_path_span = Span::styled(
            bookmark_description_string,
            Style::default().fg(Color::DarkGray),
        );

        let line_content = vec![
            decoration_span,
            icon_span,
            bookmark_name_span,
            bookmark_path_span,
        ];

        let line = Line::from(line_content);
        ListItem::new(line)
    }
}
