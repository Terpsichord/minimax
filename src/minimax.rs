use num_traits::Float;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Player {
    #[default]
    Max,
    Min,
}

impl Player {
    pub fn opposite(self) -> Player {
        match self {
            Player::Min => Player::Max,
            Player::Max => Player::Min,
        }
    }
}

pub trait State<V: Float, A: Clone + FromStr>: Default {
    fn is_terminal(&self) -> bool;
    fn heuristic_value(&self) -> V;
    fn current_player(&self) -> Player;
    fn actions(&self) -> Vec<A>;
    fn result(&self, action: &A) -> Self;
}

pub fn best_move<S, V, A>(state: &S) -> A
where
    S: State<V, A>,
    V: Float,
    A: Clone + FromStr,
{
    state
        .actions()
        .into_iter()
        .max_by(|x, y| {
            let key = |action| minimax(&state.result(action));
            key(x).partial_cmp(&key(y)).unwrap_or(Ordering::Equal)
        })
        .expect("No moves available")
}

pub fn minimax<S, V, A>(state: &S /* action: Option<A> */) -> V
where
    S: State<V, A>,
    V: Float,
    A: Clone + FromStr,
{
    if state.is_terminal() {
        return state.heuristic_value();
        // Result {
        //     value: state.heuristic_value(),
        //     action: action.expect("expected non-terminal state"),
        // }
    }

    let reduce_result = if let Player::Max = state.current_player() {
        V::max
    } else {
        V::min
    };

    state
        .actions()
        .into_iter()
        .map(|a| minimax(&state.result(&a)))
        .reduce(reduce_result)
        .expect("expected non-terminal state but no more moves were available")
}
