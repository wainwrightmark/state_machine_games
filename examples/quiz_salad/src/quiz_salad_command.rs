use crate::*;


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum QuizSaladCommand {
    TileClicked(Tile4x4),
    LozengeClicked(usize),
}

impl GameCommand for QuizSaladCommand {}
