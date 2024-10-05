use crate::action::Action;
use crate::app::GameOwned;
use crate::components::Component;
use crate::games::Game;
use crossterm::event::{KeyCode, KeyEvent};
use itertools::Itertools;
use ratatui::layout::{Constraint, Flex, Margin, Rect};
use ratatui::prelude::Layout;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use tui_textarea::TextArea;

pub struct GameScreen<'a> {
    game: Box<dyn Game>,
    input: TextArea<'a>,
    valid_input: bool,
}

impl GameScreen<'_> {
    pub fn new(game: Box<dyn Game>) -> Self {
        let mut input = TextArea::default();
        input.set_cursor_line_style(Style::default());
        input.set_placeholder_text("Enter move: ");
        Self {
            game,
            input,
            valid_input: true,
        }
    }

    fn enter_input(&mut self) {
        if self.game.is_valid_move(&self.input.lines()[0]) {
            self.valid_input = true;
            self.game.play_move(&self.input.lines()[0]);

            // clear the input
            self.input.select_all();
            self.input.cut();
        } else {
            self.valid_input = false;
            self.input.set_block(
                Block::bordered()
                    .title("Invalid move")
                    .title_style(Color::LightRed),
            );
        }
    }
}

impl Component for GameScreen<'_> {
    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        if let KeyCode::Enter = key.code {
            self.enter_input();
        } else {
            if self.input.input_without_shortcuts(key) {
                self.valid_input = true;
            }
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let areas = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)])
            .flex(Flex::SpaceAround)
            .split(area);
        let (game_area, input) = areas.into_iter().collect_tuple().unwrap();

        let game_areas = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(self.game.display_size().0 + 2),
            Constraint::Fill(1),
        ])
        .flex(Flex::SpaceAround)
        .split(*game_area);

        let (_, game_view_area, move_history_area) =
            game_areas.into_iter().collect_tuple().unwrap();
        let game_view_area = Layout::vertical([self.game.display_size().1 + 2])
            .flex(Flex::SpaceAround)
            .split(*game_view_area)[0];
        let move_history_area = move_history_area.inner(Margin {
            horizontal: 4,
            vertical: 2,
        });

        let game_view = Paragraph::new(self.game.display()).block(Block::bordered());
        frame.render_widget(game_view, game_view_area);

        let move_history = Paragraph::new(self.game.move_history()).block(Block::bordered());
        frame.render_widget(move_history, move_history_area);

        if self.valid_input {
            self.input.set_block(Block::bordered());
        } else {
            self.input.set_block(
                Block::bordered()
                    .title("Invalid move")
                    .title_style(Color::LightRed),
            )
        }
        frame.render_widget(&self.input, *input);

        Ok(())
    }
}
