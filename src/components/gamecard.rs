use crate::components::Component;
use crate::games::Game;
use ratatui::layout::Rect;
use ratatui::prelude::{Alignment, Modifier, Style};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

pub const THUMBNAIL_SIZE: u16 = 11;

/// `THUMBNAIL_SIZE` plus the size of the borders
pub const GAMECARD_SIZE: u16 = THUMBNAIL_SIZE + 2;

#[derive(Debug)]
pub struct GameCard {
    name: &'static str,
    thumbnail: &'static str,
    selected: bool,
}

impl GameCard {
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

impl From<&dyn Game> for GameCard {
    fn from(value: &dyn Game) -> Self {
        Self {
            name: value.name(),
            thumbnail: value.thumbnail(),
            selected: false,
        }
    }
}

impl Component for GameCard {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        frame.render_widget(
            Paragraph::new(self.thumbnail).block(
                Block::bordered()
                    .title(self.name)
                    .title_alignment(Alignment::Center)
                    .title_style(if self.selected {
                        Modifier::BOLD.into()
                    } else {
                        Style::new()
                    }),
            ),
            area,
        );
        Ok(())
    }
}
