use crate::action::Action;
use crate::components::Component;
use crate::games::{Game, WinState};
use color_eyre::eyre::eyre;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use itertools::Itertools;
use ratatui::layout::{Constraint, Flex, Margin, Rect};
use ratatui::prelude::Layout;
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread;
use std::thread::JoinHandle;
use tui_textarea::TextArea;
use tui_widgets::popup::{Popup, PopupState};

#[derive(Clone, Copy)]
enum GameOver {
    Win,
    Lose,
    Draw,
}

#[derive(Clone, Copy, Default)]
enum InputLabel {
    #[default]
    Invalid,
    Thinking,
}

pub struct GameScreen<'a> {
    game: Arc<RwLock<Box<dyn Game>>>,
    input: TextArea<'a>,
    input_label: Option<InputLabel>,
    game_over: Option<GameOver>,
    popup_state: PopupState,
    computer_move_thread: Option<JoinHandle<String>>,
}

impl GameScreen<'_> {
    pub fn new(game: Box<dyn Game>) -> Self {
        let mut input = TextArea::default();
        input.set_cursor_line_style(Style::default());
        input.set_placeholder_text("Enter move: ");
        Self {
            game: Arc::new(RwLock::new(game)),
            input,
            input_label: None,
            game_over: None,
            popup_state: PopupState::default(),
            computer_move_thread: None,
        }
    }

    fn game(&self) -> RwLockReadGuard<Box<dyn Game>> {
        self.game.read().expect("Failed to access the game state")
    }

    fn game_mut(&self) -> RwLockWriteGuard<Box<dyn Game>> {
        self.game.write().expect("Failed to access the game state")
    }

    pub fn name(&self) -> String {
        self.game().name()
    }

    fn enter_input(&mut self) {
        if self.computer_move_thread.is_none() {
            if self.game().is_valid_move(&self.input.lines()[0]) {
                self.input_label = None;

                self.game_mut().play_move(&self.input.lines()[0]);
                self.update_game_over();

                if self.game_over.is_none() {
                    self.computer_move_thread = Some(thread::spawn({
                        let game = Arc::clone(&self.game);
                        move || {
                            game.read()
                                .expect("Failed to access the game state")
                                .computer_move()
                        }
                    }));
                }

                // clear the input
                self.input.select_all();
                self.input.cut();
            } else {
                self.input_label = Some(InputLabel::Invalid);
                self.input.set_block(
                    Block::bordered()
                        .title("Invalid move")
                        .title_style(Color::LightRed),
                );
            }
        } else {
            self.input_label = Some(InputLabel::Thinking);
        }
    }

    fn update_game_over(&mut self) {
        let win_state = { self.game().win_state() };
        if let Some(win_state) = win_state {
            self.game_over = Some(match win_state {
                WinState::Decisive => GameOver::Win,
                WinState::Draw => GameOver::Draw,
            })
        }
    }

    fn restart(&mut self) {
        self.game_over = None;
        self.game_mut().reset();
    }

    /// Splits the rect into 3 areas (game view, move history, and move input, returned in that order)
    fn layout_areas(area: Rect, display_size: (u16, u16)) -> [Rect; 3] {
        let [game_area, input_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)])
                .flex(Flex::SpaceAround)
                .areas(area);

        let [_, game_view_area, move_history_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(display_size.0 + 2),
            Constraint::Fill(1),
        ])
        .flex(Flex::SpaceAround)
        .areas(game_area);

        let [game_view_area] = Layout::vertical([display_size.1 + 2])
            .flex(Flex::SpaceAround)
            .areas(game_view_area);
        let move_history_area = move_history_area.inner(Margin {
            horizontal: 4,
            vertical: 2,
        });

        [game_view_area, move_history_area, input_area]
    }

    fn format_move_history(moves: &[String]) -> String {
        moves
            .chunks(2)
            .map(|turn| (&turn[0], turn.get(1)))
            .enumerate()
            .map(|(n, (move_1, move_2))| {
                format!(
                    "{}. {} {}",
                    n + 1,
                    move_1,
                    move_2.map(String::as_str).unwrap_or_default()
                )
            })
            .join("\n")
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
            self.input_label = None;
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

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let [game_view_area, move_history_area, input_area] =
            Self::layout_areas(area, self.game().display_size());

        let game_view = Paragraph::new(self.game().display()).block(Block::bordered());
        frame.render_widget(game_view, game_view_area);

        let move_history_text = Text::from(Self::format_move_history(&self.game().move_history()));

        let line_count =
            u16::try_from(move_history_text.lines.len()).expect("too many lines in move history");
        let move_history_height = move_history_area.as_size().height - 2;

        let scroll = line_count.saturating_sub(move_history_height);

        let move_history = Paragraph::new(move_history_text)
            .scroll((scroll, 0))
            .block(Block::bordered());
        frame.render_widget(move_history, move_history_area);

        if let Some(input_label) = self.input_label {
            let (input_text, color) = match input_label {
                InputLabel::Invalid => ("Invalid move", Color::LightRed),
                InputLabel::Thinking => ("Computer is thinking", Color::LightBlue),
            };

            self.input
                .set_block(Block::bordered().title(input_text).title_style(color))
        } else {
            self.input.set_block(Block::bordered());
        }
        frame.render_widget(&self.input, input_area);

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

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        if action == Action::Tick
            && self
                .computer_move_thread
                .as_ref()
                .is_some_and(|t| t.is_finished())
        {
            let handle = self.computer_move_thread.take().unwrap(); // guaranteed to be Some
            let computer_move = handle
                .join()
                .map_err(|_| eyre!("Failed to make computer move"))?;

            self.game_mut().play_move(&computer_move);
            let win_state = { self.game().win_state() };
            if let Some(win_state) = win_state {
                self.game_over = Some(match win_state {
                    WinState::Decisive => GameOver::Lose,
                    WinState::Draw => GameOver::Draw,
                })
            }
        }

        Ok(None)
    }
}
