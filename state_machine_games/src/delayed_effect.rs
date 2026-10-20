use std::collections::BinaryHeap;

use crate::prelude::GameState;

pub struct DelayedMutation<GS: GameState> {
    pub mutation: Mutation<GS>,
    pub delay_ms: f64,
}

pub enum Mutation<GS: GameState> {
    Static(&'static dyn for<'a> Fn(&'a mut GS) -> MutationResult<GS>),
    StaticWithArg(
        u64,
        &'static dyn for<'a> Fn(&'a mut GS, u64) -> MutationResult<GS>,
    ),
    Boxed(Box<dyn for<'a> Fn(&'a mut GS) -> MutationResult<GS>>),
}

impl<GS: GameState> Mutation<GS> {
    pub fn apply(&self, gs: &mut GS) -> MutationResult<GS> {
        match self {
            Mutation::Static(f) => f(gs),
            Mutation::StaticWithArg(arg, f) => f(gs, *arg),
            Mutation::Boxed(b) => b.as_ref()(gs),
        }
    }
}

pub enum MutationResult<GS: GameState> {
    NoChange,
    Changed,
    ChangedWithMutation(DelayedMutation<GS>),
    ChangedWithMutationVec(Vec<DelayedMutation<GS>>),
}

struct MutationAtTime<GS: GameState> {
    time_ms: f64,
    mutation: Mutation<GS>,
}

impl<GS: GameState> Ord for MutationAtTime<GS> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        //This is to be used in a max heap so we reverse the comparison
        //This means shorter times will be put on top of the heap
        self.time_ms.total_cmp(&other.time_ms).reverse()
    }
}

impl<GS: GameState> Eq for MutationAtTime<GS> {}

impl<GS: GameState> PartialEq for MutationAtTime<GS> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<GS: GameState> PartialOrd for MutationAtTime<GS> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        //This is to be used in a max heap so we reverse the comparison
        //This means shorter times will be put on top of the heap
        self.time_ms.partial_cmp(&other.time_ms).map(|x|x.reverse())
    }
}

pub(crate) struct DelayedEffectStore<GS: GameState> {
    now_ms: f64,

    heap: BinaryHeap<MutationAtTime<GS>>,
}

impl<GS: GameState> DelayedEffectStore<GS> {
    pub fn new() -> Self {
        Self {
            now_ms: 0.0,
            heap: Default::default(),
        }
    }

    /// Returns whether the game state was changed
    pub fn tick(&mut self, game_state: &mut GS, delta_ms: f64) -> bool {
        self.now_ms += delta_ms;
        let mut changed = false;

        loop {
            let Some(top) = self.heap.peek() else {
                return changed;
            };
            if top.time_ms > self.now_ms {
                return changed;
            }

            let Some(top) = self.heap.pop() else {
                return changed;
            };
            match top.mutation.apply(game_state) {
                MutationResult::NoChange => {}
                MutationResult::Changed => {
                    changed = true;
                }
                MutationResult::ChangedWithMutation(dm) => {
                    self.push_delayed_mutation(dm);
                    changed = true;
                }
                MutationResult::ChangedWithMutationVec(delayed_mutations) => {
                    for dm in delayed_mutations.into_iter() {
                        self.push_delayed_mutation(dm);
                        changed = true;
                    }
                }
            }
        }
    }

    pub fn push_delayed_mutation(&mut self, delayed_mutation: DelayedMutation<GS>) {
        self.heap.push(MutationAtTime {
            time_ms: self.now_ms + delayed_mutation.delay_ms,
            mutation: delayed_mutation.mutation,
        });
    }
}
