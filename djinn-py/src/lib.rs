use pyo3::prelude::*;
use djinn_minimax as minimax;

struct State(PyObject);

impl<'a> Default for State {
    fn default() -> Self {
        panic!("Default not implemented for PyStateWrapper");
    }
}

impl minimax::State<f64, String> for State {
    fn is_terminal(&self) -> bool {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "is_terminal")
                .expect("Failed to call Python method 'is_terminal'")
                .extract(py)
                .expect("Failed to extract bool from Python")
        })
    }

    fn evaluation(&self) -> f64 {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "heuristic_value")
                .expect("Failed to call Python method 'heuristic_value'")
                .extract(py)
                .expect("Failed to extract f64 from Python")
        })
    }

    fn current_player(&self) -> minimax::Player {
        Python::with_gil(|py| {
            let is_maximising_player = self.0
                .call_method0(py, "is_maximising_player")
                .expect("Failed to call Python method 'is_maximising_player'")
                .extract(py)
                .expect("Failed to extract bool from Python");
            if is_maximising_player {
                minimax::Player::Max
            } else {
                minimax::Player::Min
            }
        })
    }

    fn actions(&self) -> Vec<String> {
        Python::with_gil(|py| {
            self.0
                .call_method0(py, "actions")
                .expect("Failed to call Python method 'actions'")
                .extract(py)
                .expect("Failed to extract Vec<String> from Python")
        })
    }

    fn result(&self, action: &String) -> Self {
        Python::with_gil(|py| {
            State(self.0
                .call_method1(py, "result", (action,))
                .expect("Failed to call Python method 'result'")
                .extract(py)
                .expect("Failed to extract new PyStateWrapper from Python")
            )
        })
    }
}


#[pyfunction]
fn best_move(state: PyObject, depth: u32) -> String {
    minimax::best_move(&State(state), depth)
}

/// A Python module implemented in Rust.
#[pymodule]
fn djinn_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(best_move, m)?)?;
    Ok(())
}
