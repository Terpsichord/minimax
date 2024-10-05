use crate::action::Action;
use crate::components::Component;
use crate::games::{Game, WinState};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use itertools::Itertools;
use ratatui::layout::{Constraint, Flex, Margin, Rect};
use ratatui::prelude::Layout;
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use tui_textarea::TextArea;
use tui_widgets::popup::{Popup, PopupState};

#[derive(Clone, Copy)]
enum GameOver {
    Win,
    Lose,
    Draw,
}

pub struct GameScreen<'a> {
    game: Box<dyn Game>,
    input: TextArea<'a>,
    valid_input: bool,
    game_over: Option<GameOver>,
    popup_state: PopupState,
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
            game_over: None,
            popup_state: PopupState::default(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.game.name()
    }

    fn enter_input(&mut self) {
        if self.game.is_valid_move(&self.input.lines()[0]) {
            self.valid_input = true;

            self.game.play_move(&self.input.lines()[0]);
            if let Some(win_state) = self.game.win_state() {
                self.game_over = Some(match win_state {
                    WinState::Decisive => GameOver::Win,
                    WinState::Draw => GameOver::Draw,
                })
            } else {
                self.game.play_move(&self.game.computer_move());
                if let Some(win_state) = self.game.win_state() {
                    self.game_over = Some(match win_state {
                        WinState::Decisive => GameOver::Lose,
                        WinState::Draw => GameOver::Draw,
                    })
                }
            }

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

    fn restart(&mut self) {
        self.game_over = None;
        self.game.reset();
    }
}

impl Component for GameScreen<'_> {
    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        if self.game_over.is_some() {
            #[allow(clippy::single_match)]
            match key.code {
                KeyCode::Char('r') => self.restart(),
                _ => {}
            }
        } else if let KeyCode::Enter = key.code {
                self.enter_input();
        } else if self.input.input_without_shortcuts(key) {
            self.valid_input = true;
        }

        Ok(None)
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> color_eyre::Result<Option<Action>> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.popup_state.mouse_down(mouse.column, mouse.row);
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.popup_state.mouse_up(mouse.column, mouse.row);
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.popup_state.mouse_drag(mouse.column, mouse.row);
            }
            _ => {}
        }
        Ok(None)
    }

    // TODO: separate into multiple functions
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let [game_area, input] = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)])
            .flex(Flex::SpaceAround)
            .areas(area);

        let [_, game_view_area, move_history_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(self.game.display_size().0 + 2),
            Constraint::Fill(1),
        ])
        .flex(Flex::SpaceAround)
        .areas(game_area);

        let [game_view_area] = Layout::vertical([self.game.display_size().1 + 2])
            .flex(Flex::SpaceAround)
            .areas(game_view_area);
        let move_history_area = move_history_area.inner(Margin {
            horizontal: 4,
            vertical: 2,
        });

        let game_view = Paragraph::new(self.game.display()).block(Block::bordered());
        frame.render_widget(game_view, game_view_area);

        let move_history_text = Text::from(self
            .game
            .move_history()
            .into_iter()
            .enumerate()
            .map(|(n, (move_1, move_2))| {
                format!("{}. {} {}", n + 1, move_1, move_2.unwrap_or_default())
            })
            .join("\n"));

        let line_count = u16::try_from(move_history_text.lines.len()).expect("too many lines in move history");
        let move_history_height = move_history_area.as_size().height - 2;
        let scroll = if line_count <= move_history_height {
            0
        } else {
            line_count - move_history_height
        };

        let move_history = Paragraph::new(move_history_text).scroll((scroll,  0)).block(Block::bordered());
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
        frame.render_widget(&self.input, input);

        if let Some(game_over) = self.game_over {
            let title = match game_over {
                GameOver::Win => "You win!",
                GameOver::Lose => "You lost!",
                GameOver::Draw => "Draw",
            };
            let mut popup =
                Popup::new(Text::raw("<r> - retry\n<Ctrl-b> - back\n<Ctrl-q> - quit")).title(title);
            popup.border_set = border::THICK;
            frame.render_stateful_widget_ref(popup, frame.area(), &mut self.popup_state);
        }

        Ok(())
    }
}
