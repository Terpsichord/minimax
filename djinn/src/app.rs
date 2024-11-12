use color_eyre::eyre::eyre;
use color_eyre::Result;
use crossterm::event::KeyEvent;
use pyo3::Python;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::components::game_menu::GameMenu;
use crate::components::game_screen::GameScreen;
use crate::games::chess::Chess;
use crate::games::tictactoe::TicTacToe;
use crate::games::Game;
use crate::plugins::python::PythonPluginManager;
use crate::tui::TuiConfigBuilder;
use crate::{
    action::Action,
    components::{fps::FpsCounter, gamecard::GameCard, Component},
    config::Config,
    tui::{Event, Tui},
};

pub struct App<'a> {
    config: Config,
    home_components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    screen: Screen,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    game_screens: BTreeMap<GameId, GameScreen<'a>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    Home,
    Game,
    All,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct GameId(usize);

static NEXT_GAME_ID: AtomicUsize = AtomicUsize::new(0);

impl GameId {
    fn new() -> Self {
        Self(NEXT_GAME_ID.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Screen {
    #[default]
    Home,
    Game(GameId),
}

impl App<'_> {
    pub fn new() -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let mut games: Vec<(GameId, Box<dyn Game>)> = vec![
            (GameId::new(), Box::new(TicTacToe::default())),
            (GameId::new(), Box::new(Chess::default())),
        ];

        let plugin_games = Python::with_gil(Self::load_python_plugins);
        games.extend(plugin_games.into_iter().map(|g| (GameId::new(), g)));

        let game_cards = games
            .iter()
            .map(|(id, game)| GameCard::from_game_with_id(&**game, *id))
            .collect();

        let game_screens = BTreeMap::from_iter(
            games
                .into_iter()
                .map(|(id, game)| (id, GameScreen::new(game))),
        );

        Ok(Self {
            home_components: vec![
                Box::new(FpsCounter::default()),
                Box::new(GameMenu::new(game_cards)),
            ],
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            screen: Screen::default(),
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            game_screens,
        })
    }

    fn load_python_plugins(py: Python<'_>) -> Vec<Box<dyn Game>> {
        let paths = vec!["../python-plugin/hex.py"];

        let plugin_manager = PythonPluginManager::new(py);
        let mut plugins = Vec::with_capacity(paths.len());
        for path in paths {
            let plugin = plugin_manager
                .load_plugin(path)
                .expect("TODO: failed to load plugin");
            let game: Box<dyn Game> = Box::new(plugin);
            plugins.push(game);
        }
        plugins
    }

    pub async fn run(&mut self) -> Result<()> {
        let tui_config = TuiConfigBuilder::default()
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.config.tick_rate)
            .frame_rate(self.config.frame_rate)
            .build()?;

        let mut tui = Tui::from_config(tui_config)?;

        tui.enter()?;

        for component in self.home_components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.home_components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.home_components.iter_mut() {
            component.init(tui.size()?)?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        self.try_perform_on_components(|component| {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn try_perform_on_components(
        &mut self,
        mut func: impl FnMut(&mut dyn Component) -> Result<()>,
    ) -> Result<()> {
        match self.screen {
            Screen::Home => {
                for component in self.home_components.iter_mut() {
                    func(&mut **component)?;
                }
            }
            Screen::Game(id) => func(
                self.game_screens
                    .get_mut(&id)
                    .expect("couldn't find game with id"),
            )?,
        }
        Ok(())
    }

    fn perform_on_components(&mut self, mut func: impl FnMut(&mut dyn Component)) {
        self.try_perform_on_components(|c| {
            func(c);
            Ok(())
        })
        .unwrap()
    }

    fn get_screen_mode(screen: Screen) -> Mode {
        match screen {
            Screen::Home => Mode::Home,
            Screen::Game(_) => Mode::Game,
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();

        let fallback_map = self.config.keybindings.get(&Mode::All);
        let keymap = self
            .config
            .keybindings
            .get(&Self::get_screen_mode(self.screen));

        if keymap.is_none() && fallback_map.is_none() {
            return Ok(());
        }

        let get_from_maps = |map_key| {
            keymap
                .and_then(|map| map.get(map_key))
                .or_else(|| fallback_map.and_then(|map| map.get(map_key)))
        };

        match get_from_maps(&vec![key]) {
            Some(action) => {
                info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
            }
            None => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = get_from_maps(&self.last_tick_key_events) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                }
            }
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                Action::OpenGame(game_id) => self.open_game(game_id),
                Action::Back if matches!(self.screen, Screen::Game(_)) => self.back(),
                _ => {}
            }

            let action_tx = self.action_tx.clone();
            self.try_perform_on_components(|component| {
                if let Some(action) = component.update(action.clone())? {
                    action_tx.send(action)?
                };
                Ok(())
            })?;
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            let action_tx = self.action_tx.clone();
            self.perform_on_components(|component| {
                if let Err(err) = component.draw(frame, frame.area()) {
                    let _ = action_tx.send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            })
        })?;
        Ok(())
    }

    fn open_game(&mut self, game_id: GameId) {
        self.screen = Screen::Game(game_id);
    }

    fn back(&mut self) {
        self.screen = Screen::Home;
    }

    pub fn open_game_from_name(&self, name: &str) -> Result<()> {
        let game_id = self
            .game_screens
            .iter()
            .find_map(|(id, game_screen)| {
                if game_screen.name().to_lowercase() == name.to_lowercase() {
                    Some(id)
                } else {
                    None
                }
            })
            .ok_or_else(|| eyre!("no game with name \"{name}\" found"))?;
        self.action_tx.send(Action::OpenGame(*game_id))?;
        Ok(())
    }
}
