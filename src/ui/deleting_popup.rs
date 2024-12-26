use crate::ui::App;

use ratatui::widgets::{Block, Paragraph,Clear,BorderType};
use ratatui::prelude::Constraint;
use ratatui::layout::Flex;
use ratatui::style::{Color, Style};
use ratatui::prelude::Layout;

pub fn render(app: &App, popup_area: ratatui::layout::Rect, frame: &mut ratatui::Frame) {

    let block =
        Block::bordered().border_type(BorderType::Rounded);

    // Create the "Yes" and "No" buttons
    let yes_button_text = "Yes";
    let yes_button = Paragraph::new(yes_button_text)
        .block(Block::bordered().border_type(BorderType::Rounded))
        .centered();

    // Highlight the "Yes" button if it is selected
    let yes_button = if app
        .deleting_state
        .clone()
        .unwrap()
        .selection
        .unwrap_or(false)
    {
        yes_button.style(Style::default().fg(Color::Red))
    } else {
        yes_button
    };

    let no_button_text = "No";
    let no_button = Paragraph::new(no_button_text)
        .block(Block::bordered().border_type(BorderType::Rounded))
        .centered();

    // Highlight the "No" button if it is selected
    let no_button = if !app
        .deleting_state
        .clone()
        .unwrap()
        .selection
        .unwrap_or(false)
    {
        no_button.style(Style::default().fg(Color::Blue))
    } else {
        no_button
    };

    // Create the title paragraph
    let title_text = "Are you sure you want to delete bookmark? ".to_string() +" (y/N)";
    let title = Paragraph::new(title_text.clone())
        .block(Block::default())
        .centered();

    // Create the flex layouts
    let flex_horizontal_buttons = Layout::horizontal([
        Constraint::Max(yes_button_text.len() as u16 + 6),
        Constraint::Max(no_button_text.len() as u16 + 6),
    ])
    .flex(Flex::SpaceAround);

    let flex_horizontal_title =
        Layout::horizontal([Constraint::Max(title_text.len() as u16 + 6)]).flex(Flex::Center);

    let flex_vertical =
        Layout::vertical([Constraint::Max(1), Constraint::Max(3)]).flex(Flex::SpaceAround);

    let [title_area, buttons_area] = flex_vertical.areas(popup_area);

    let [button1_area, button2_area] = flex_horizontal_buttons.areas(buttons_area);

    let [title_area] = flex_horizontal_title.areas(title_area);

    // Render the popup block
    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    // Render the title
    frame.render_widget(title, title_area);

    // Render the buttons
    frame.render_widget(yes_button, button1_area);
    frame.render_widget(no_button, button2_area);
}
