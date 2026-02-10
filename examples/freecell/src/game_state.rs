use arrayvec::ArrayVec;
use glam::Vec2;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use state_machine_games::prelude::{GameCommand, GameState, MutationResult, TinyRng};
use strum::IntoEnumIterator;

use crate::card::{Card, Rank, Suit};

#[derive(Debug, Clone, PartialEq)]
pub struct FreecellGameState {
    pub top_cells: ArrayVec<Card, 4>,

    /// Counts of cards up by suit
    pub cards_up: [u8; 4],
    pub stacks: [Stack; 8],
}

impl FreecellGameState {
    pub fn cards_up_by_suit(&mut self, suit: Suit) -> Option<&mut u8> {
        self.cards_up.get_mut(suit.index() as usize)
    }

    pub fn get_stack(&mut self, stack_index: u8) -> Option<&mut Stack> {
        self.stacks.get_mut(stack_index as usize)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    pub cards: ArrayVec<Card, 19>,
    //todo split into ordered and unordered
}
impl Stack {
    pub const EMPTY: Stack = Stack {
        cards: ArrayVec::new_const(),
    };

    pub fn can_add(&self, card: Card) -> bool {
        if let Some(last) = self.cards.last() {
            card.rank() as u8 + 1 == last.rank() as u8 && !card.suit().matches_color(last.suit())
        } else {
            //stack is empty
            return true;
        }
    }
}

impl FreecellGameState {
    pub fn new_rand(rng: &mut impl Rng) -> Self {
        let mut iter = Card::iter();
        let mut cards: [Card; 52] = std::array::from_fn(|_| iter.next().unwrap());
        cards.shuffle(rng);

        let mut stacks = [Stack::EMPTY; 8];
        let mut stack = 0usize;
        for card in cards {
            let max_size = if stack < 4 { 8 } else { 7 };
            stacks[stack].cards.push(card);
            if stacks[stack].cards.len() >= max_size {
                stack += 1;
            }
        }

        Self {
            top_cells: ArrayVec::new(),
            cards_up: [0; 4],
            stacks,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FreecellCommand {
    NewRandomGame,
    MoveCard {
        from: MoveFromPosition,
        to: MoveToPosition,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CardPosition {
    CardsUp { suit: Suit },
    Stack { stack_index: u8, row_index: u8 },
    TopCells(u8),
}

impl CardPosition {
    pub fn position(&self) -> Vec2 {
        crate::layout::get_position(self.clone())
    }

    pub fn as_from_position(&self) -> MoveFromPosition {
        match self.clone() {
            CardPosition::CardsUp { suit } => MoveFromPosition::CardsUp { suit },
            CardPosition::Stack { stack_index, .. } => MoveFromPosition::Stack(stack_index),
            CardPosition::TopCells(index) => MoveFromPosition::TopCells(index),
        }
    }

    pub fn as_to_position(&self) -> MoveToPosition {
        match self.clone() {
            CardPosition::CardsUp { .. } => MoveToPosition::CardsUp,
            CardPosition::Stack { stack_index, .. } => MoveToPosition::Stack(stack_index),
            CardPosition::TopCells(_) => MoveToPosition::TopCells,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MoveFromPosition {
    CardsUp { suit: Suit },
    Stack(u8),
    TopCells(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MoveToPosition {
    CardsUp,
    Stack(u8),
    TopCells,
}

impl GameState for FreecellGameState {
    type Command = FreecellCommand;

    fn maybe_transition(&mut self) -> state_machine_games::prelude::MutationResult {
        MutationResult::NO_CHANGE
    }
}

impl GameCommand<FreecellGameState> for FreecellCommand {
    fn apply_command(&self, game_state: &mut FreecellGameState) -> MutationResult {
        match self {
            FreecellCommand::NewRandomGame => {
                let mut rng = TinyRng::from_os_rng();
                *game_state = FreecellGameState::new_rand(&mut rng);
                MutationResult::CHANGED_NO_TRANSITION
            }
            FreecellCommand::MoveCard { from, to } => {
                let mut new_state = game_state.clone();
                let card = match from {
                    MoveFromPosition::CardsUp { suit } => {
                        let Some(rank_number) = new_state.cards_up_by_suit(*suit) else {
                            return MutationResult::NO_CHANGE;
                        };

                        let Some(rank) = Rank::from_repr(*rank_number) else {
                            return MutationResult::NO_CHANGE;
                        };
                        *rank_number = *rank_number - 1;
                        Card::from_suit_and_rank(*suit, rank)
                    }
                    MoveFromPosition::Stack(stack_index) => {
                        let Some(stack) = new_state.stacks.get_mut(*stack_index as usize) else {
                            return MutationResult::NO_CHANGE;
                        };
                        let Some(card) = stack.cards.pop() else {
                            return MutationResult::NO_CHANGE;
                        };
                        card
                    }
                    MoveFromPosition::TopCells(index) => {
                        let index = *index as usize;
                        if index >= new_state.top_cells.len() {
                            return MutationResult::NO_CHANGE;
                        }
                        new_state.top_cells.remove(index)
                    }
                };

                match to {
                    MoveToPosition::CardsUp => {
                        let Some(rank_number) = new_state.cards_up_by_suit(card.suit()) else {
                            return MutationResult::NO_CHANGE;
                        };
                        if *rank_number == card.rank() as u8 + 1 {
                            *rank_number = *rank_number + 1;
                        } else {
                            return MutationResult::NO_CHANGE;
                        }
                    }
                    MoveToPosition::Stack(stack_index) => {
                        let Some(stack) = new_state.get_stack(*stack_index) else {
                            return MutationResult::NO_CHANGE;
                        };
                        if stack.can_add(card) {
                            stack.cards.push(card);
                        } else {
                            return MutationResult::NO_CHANGE;
                        }
                    }
                    MoveToPosition::TopCells => {
                        if !new_state.top_cells.try_push(card).is_ok() {
                            return MutationResult::NO_CHANGE;
                        }
                    }
                }

                *game_state = new_state;
                MutationResult::CHANGED_NO_TRANSITION
            }
        }
    }
}
