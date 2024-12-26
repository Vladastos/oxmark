use crate::ui::App;

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Paragraph,Clear,BorderType};

pub fn render(_app: &App,popup_area: Rect, frame: &mut ratatui::Frame) {
    let block =
        Block::bordered().border_type(BorderType::Rounded);
    let paragraph = Paragraph::new("TODO").block(block);
    frame.render_widget(Clear,popup_area);
    frame.render_widget(paragraph, popup_area);
}
