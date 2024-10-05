use color_eyre::eyre::eyre;
use crossterm::event::{KeyCode, KeyEvent};
use futures::SinkExt;
use itertools::Itertools;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::text::Span;
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::components::gamecard::{GameCard, GAMECARD_SIZE};
use crate::components::Component;

pub struct GameMenu {
    game_cards: Vec<GameCard>,
    selected_game: usize,
    action_tx: Option<UnboundedSender<Action>>,
}

impl GameMenu {
    pub fn new(game_cards: Vec<GameCard>) -> Self {
        Self {
            game_cards,
            selected_game: 0,
            action_tx: None,
        }
    }

    fn previous_game(&mut self) {
        if self.selected_game == 0 {
            self.selected_game = self.game_cards.len() - 1;
        } else {
            self.selected_game -= 1;
        }
    }

    fn next_game(&mut self) {
        self.selected_game = (self.selected_game + 1) % self.game_cards.len();
    }

    fn open_game(&mut self) -> color_eyre::Result<()> {
        self.action_tx
            .as_ref()
            .ok_or(eyre!("no action_tx set"))?
            .send(Action::OpenGame(self.selected_game))?;
        Ok(())
    }
}

const GRID_SPACING: u16 = 1;

impl Component for GameMenu {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let area = area.inner(Margin {
            horizontal: 6,
            vertical: 3,
        });

        let areas = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
            .spacing(1)
            .split(area);
        let (text, grid) = areas.into_iter().collect_tuple().unwrap();

        let column_count = (grid.width / (GAMECARD_SIZE + GRID_SPACING)) as usize;
        let row_count = ((self.game_cards.len() - 1) / column_count) + 1;

        let rows = Layout::vertical(vec![GAMECARD_SIZE / 2 + 1; row_count]).split(*grid);

        let column_layout =
            Layout::horizontal(vec![GAMECARD_SIZE; column_count]).spacing(GRID_SPACING);

        let grid_areas = rows
            .into_iter()
            .map(|row| {
                column_layout
                    .split(*row)
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .flatten();

        for (i, (card, area)) in self.game_cards.iter_mut().zip(grid_areas).enumerate() {
            card.set_selected(i == self.selected_game);
            card.draw(frame, area)?;
        }

        frame.render_widget(Span::raw("Select a game to play:"), *text);

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        match key.code {
            KeyCode::Left => self.previous_game(),
            KeyCode::Right => self.next_game(),
            KeyCode::Enter | KeyCode::Char(' ') => self.open_game()?,
            _ => {}
        }

        Ok(None)
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.action_tx = Some(tx);

        Ok(())
    }
}
