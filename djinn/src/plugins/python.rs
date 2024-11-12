use crate::games::{Game, WinState};
use convert_case::{Case, Casing};
use pyo3::prelude::{PyAnyMethods, PyModule};
use pyo3::{PyObject, PyResult, Python};
use std::fs;
use std::path::Path;

pub struct Plugin(PyObject);

impl From<bool> for WinState {
    fn from(decisive: bool) -> Self {
        if decisive {
            WinState::Decisive
        } else {
            WinState::Draw
        }
    }
}

impl Game for Plugin {
    fn name(&self) -> String {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "name")
                .expect("Failed to call Python method 'name'")
                .extract::<String>(py)
                .expect("Failed to extract Python string")
        })
    }

    fn thumbnail(&self) -> String {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "thumbnail")
                .expect("Failed to call Python method 'thumbnail'")
                .extract::<String>(py)
                .expect("Failed to extract Python string")
        })
    }

    fn display(&self) -> String {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "display")
                .expect("Failed to call Python method 'display'")
                .extract::<String>(py)
                .expect("Failed to extract Python string")
        })
    }

    fn display_size(&self) -> (u16, u16) {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "display_size")
                .expect("Failed to call Python method 'display_size'")
                .extract::<(u16, u16)>(py)
                .expect("Failed to extract tuple of (u16, u16)")
        })
    }

    fn move_history(&self) -> Vec<String> {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "move_history")
                .expect("Failed to call Python method 'move_history'")
                .extract::<Vec<String>>(py)
                .expect("Failed to extract Vec<>")
        })
    }

    fn win_state(&self) -> Option<WinState> {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "win_state")
                .expect("Failed to call Python method 'win_state'")
                .extract::<Option<bool>>(py)
                .expect("Failed to extract Option<WinState>")
                .map(WinState::from)
        })
    }

    fn is_valid_move(&self, move_: &str) -> bool {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "is_valid_move", (move_,))
                .expect("Failed to call Python method 'is_valid_move'")
                .extract::<bool>(py)
                .expect("Failed to extract boolean")
        })
    }

    fn play_move(&mut self, move_: &str) {
        Python::with_gil(|py| {
            self.0
                .call_method1(py, "play_move", (move_,))
                .expect("Failed to call Python method 'play_move'");
        });
    }

    fn computer_move(&self) -> String {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "computer_move")
                .expect("Failed to call Python method 'computer_move'")
                .extract::<String>(py)
                .expect("Failed to extract Python string")
        })
    }

    fn reset(&mut self) {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "reset")
                .expect("Failed to call Python method 'reset'");
        });
    }
}

pub struct PythonPluginManager<'py>(Python<'py>);

impl<'py> PythonPluginManager<'py> {
    pub fn new(py: Python<'py>) -> Self {
        Self(py)
    }

    pub fn load_plugin<P: AsRef<Path>>(&self, path: P) -> PyResult<Plugin> {
        let code = fs::read_to_string(&path)?;
        let module_name = path.as_ref().file_stem();
        let class_name = module_name
            .expect("TODO: invalid file path")
            .to_str()
            .unwrap()
            .to_case(Case::Pascal);

        let plugin_module = PyModule::from_code_bound(
            self.0,
            &code,
            path.as_ref().file_name().unwrap().to_str().unwrap(),
            module_name.unwrap().to_str().unwrap(),
        )?;
        plugin_module
            .getattr(&*class_name)?
            .call0()
            .map(|p| Plugin(p.unbind()))
    }
}
