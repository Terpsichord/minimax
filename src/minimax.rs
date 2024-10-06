use num_traits::Float;
use std::cmp::Ordering;
use std::fmt::{Debug, Display};

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

pub trait State<V: Float, A: Clone>: Default {
    fn is_terminal(&self) -> bool;
    fn heuristic_value(&self) -> V;
    fn current_player(&self) -> Player;
    fn actions(&self) -> Vec<A>;
    fn result(&self, action: &A) -> Self;
}

pub fn best_move<S, V, A>(state: &S, depth: u32) -> A
where
    S: State<V, A>,
    V: Float,
    A: Clone,
{
    let cmp: fn(&V, &V) -> Option<Ordering> = match state.current_player() {
        Player::Max => V::partial_cmp,
        Player::Min => |a, b| V::partial_cmp(a, b).map(|o| o.reverse()),
    };

     state
        .actions()
        .into_iter()
        .max_by(|x, y| {
            let key = |action| minimax(&state.result(action), depth);
            cmp(&key(x), &key(y)).unwrap_or(Ordering::Equal)
        })
        .expect("No moves available")
}

pub fn minimax<S, V, A>(state: &S, depth: u32) -> V
where
    S: State<V, A>,
    V: Float,
    A: Clone,
{
    if state.is_terminal() || depth == 0 {
        return state.heuristic_value();
    }

    let reduce_result = if let Player::Max = state.current_player() {
        V::max
    } else {
        V::min
    };

    state
        .actions()
        .into_iter()
        .map(|a| minimax(&state.result(&a), depth - 1))
        .reduce(reduce_result)
        .expect("expected non-terminal state but no more moves were available")
}
