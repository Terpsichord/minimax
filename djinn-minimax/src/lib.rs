use num_traits::{One, Bounded};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::{Add, Neg};

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

pub trait State<V: Evaluation, A: Clone> {
    fn is_terminal(&self) -> bool;
    fn evaluation(&self) -> V;
    fn current_player(&self) -> Player;
    fn actions(&self) -> Vec<A>;
    fn result(&self, action: &A) -> Self;
}

pub trait Evaluation: Bounded + Ord + Neg<Output=Self> + Copy + Add<Self, Output=Self> + One {}

impl<T: Bounded + Ord + Neg<Output=Self> + Copy + Add<Self, Output=Self> + One> Evaluation for T {}

pub fn best_move<S, V, A>(state: &S, depth: u32) -> A
    where
        S: State<V, A>,
        V: Evaluation,
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
        V: Evaluation,
        A: Clone,
{
    alpha_beta(
        state,
        // add 1 to prevent overflow when negated
        V::min_value() + V::one(),
        V::max_value(),
        depth
    )
}

fn alpha_beta<S, V, A>(state: &S, mut alpha: V, beta: V, depth: u32) -> V
    where
        S: State<V, A>,
        V: Evaluation,
        A: Clone,
{
    if state.is_terminal() || depth == 0 {
        let eval = state.evaluation();
        return if state.current_player() == Player::Max {
            eval
        } else {
            -eval
        }
    }



    let mut best_value = V::min_value();

    for action in state.actions() {
        let value  = -alpha_beta(&state.result(&action), -beta, -alpha, {
            depth - 1
        });

        best_value = V::max(best_value, value);
        alpha = V::max(alpha, value);

        if alpha >= beta {
            break;
        }
    }

    return best_value;
}
