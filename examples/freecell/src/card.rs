use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, FromRepr};

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord, Hash, FromRepr, Serialize, Deserialize)]
pub enum Suit {
    Club = 0,
    Diamond = 16,
    Spade = 32,
    Heart = 48,
}

impl Suit {
    pub fn index(&self) -> u8 {
        *self as u8 >> 4
    }

    pub fn from_index(i:u8)-> Option<Self> {
        Self::from_repr(i << 4)
    }

    fn is_red(&self) -> bool {
        match self {
            Suit::Club => false,
            Suit::Diamond => true,
            Suit::Spade => false,
            Suit::Heart => true,
        }
    }

    pub(crate) fn matches_color(&self, suit: Suit) -> bool {
        self.is_red() == suit.is_red()
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord, Hash, FromRepr)]
pub enum Rank {
    Rank1 = 1,
    Rank2 = 2,
    Rank3 = 3,
    Rank4 = 4,
    Rank5 = 5,
    Rank6 = 6,
    Rank7 = 7,
    Rank8 = 8,
    Rank9 = 9,
    Rank10 = 10,
    Rank11 = 11,
    Rank12 = 12,
    Rank13 = 13,
}

#[repr(u8)]
#[derive(
    Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord, Hash, FromRepr, EnumCount, EnumIter,
)]
pub enum Card {
    Club1 = 1,
    Club2 = 2,
    Club3 = 3,
    Club4 = 4,
    Club5 = 5,
    Club6 = 6,
    Club7 = 7,
    Club8 = 8,
    Club9 = 9,
    Club10 = 10,
    Club11 = 11,
    Club12 = 12,
    Club13 = 13,

    Diamond1 = 16 + 1,
    Diamond2 = 16 + 2,
    Diamond3 = 16 + 3,
    Diamond4 = 16 + 4,
    Diamond5 = 16 + 5,
    Diamond6 = 16 + 6,
    Diamond7 = 16 + 7,
    Diamond8 = 16 + 8,
    Diamond9 = 16 + 9,
    Diamond10 = 16 + 10,
    Diamond11 = 16 + 11,
    Diamond12 = 16 + 12,
    Diamond13 = 16 + 13,

    Spade1 = 32 + 1,
    Spade2 = 32 + 2,
    Spade3 = 32 + 3,
    Spade4 = 32 + 4,
    Spade5 = 32 + 5,
    Spade6 = 32 + 6,
    Spade7 = 32 + 7,
    Spade8 = 32 + 8,
    Spade9 = 32 + 9,
    Spade10 = 32 + 10,
    Spade11 = 32 + 11,
    Spade12 = 32 + 12,
    Spade13 = 32 + 13,

    Heart1 = 48 + 1,
    Heart2 = 48 + 2,
    Heart3 = 48 + 3,
    Heart4 = 48 + 4,
    Heart5 = 48 + 5,
    Heart6 = 48 + 6,
    Heart7 = 48 + 7,
    Heart8 = 48 + 8,
    Heart9 = 48 + 9,
    Heart10 = 48 + 10,
    Heart11 = 48 + 11,
    Heart12 = 48 + 12,
    Heart13 = 48 + 13,
}

impl Card {
    pub fn from_suit_and_rank(suit: Suit, rank: Rank) -> Self {
        let repr = suit as u8 + rank as u8;
        Self::from_repr(repr).unwrap()
    }

    pub fn suit(self) -> Suit {
        let repr = self as u8 & 0b11111100;
        Suit::from_repr(repr).unwrap()
    }

    pub fn rank(self) -> Rank {
        let repr = self as u8 & 0b11;
        Rank::from_repr(repr).unwrap()
    }

    pub const fn svg_path(&self)-> &'static str{
        match self{
            Card::Club1 =>  "clubs_ace.svg",
            Card::Club2 =>  "clubs_2.svg",
            Card::Club3 =>  "clubs_3.svg",
            Card::Club4 =>  "clubs_4.svg",
            Card::Club5 =>  "clubs_5.svg",
            Card::Club6 =>  "clubs_6.svg",
            Card::Club7 =>  "clubs_7.svg",
            Card::Club8 =>  "clubs_8.svg",
            Card::Club9 =>  "clubs_9.svg",
            Card::Club10 => "clubs_10.svg",
            Card::Club11 => "clubs_jack.svg",
            Card::Club12 => "clubs_queen.svg",
            Card::Club13 => "clubs_king.svg",
            Card::Diamond1 => "diamonds_ace.svg",
            Card::Diamond2 => "diamonds_2.svg",
            Card::Diamond3 => "diamonds_3.svg",
            Card::Diamond4 => "diamonds_4.svg",
            Card::Diamond5 => "diamonds_5.svg",
            Card::Diamond6 => "diamonds_6.svg",
            Card::Diamond7 => "diamonds_7.svg",
            Card::Diamond8 => "diamonds_8.svg",
            Card::Diamond9 => "diamonds_9.svg",
            Card::Diamond10 => "diamonds_10.svg",
            Card::Diamond11 => "diamonds_jack.svg",
            Card::Diamond12 => "diamonds_queen.svg",
            Card::Diamond13 => "diamonds_king.svg",
            Card::Spade1 => "spades_ace.svg",
            Card::Spade2 => "spades_2.svg",
            Card::Spade3 => "spades_3.svg",
            Card::Spade4 => "spades_4.svg",
            Card::Spade5 => "spades_5.svg",
            Card::Spade6 => "spades_6.svg",
            Card::Spade7 => "spades_7.svg",
            Card::Spade8 => "spades_8.svg",
            Card::Spade9 => "spades_9.svg",
            Card::Spade10 => "spades_10.svg",
            Card::Spade11 => "spades_jack.svg",
            Card::Spade12 => "spades_queen.svg",
            Card::Spade13 => "spades_king.svg",
            Card::Heart1 => "hearts_ace.svg",
            Card::Heart2 => "hearts_2.svg",
            Card::Heart3 => "hearts_3.svg",
            Card::Heart4 => "hearts_4.svg",
            Card::Heart5 => "hearts_5.svg",
            Card::Heart6 => "hearts_6.svg",
            Card::Heart7 => "hearts_7.svg",
            Card::Heart8 => "hearts_8.svg",
            Card::Heart9 => "hearts_9.svg",
            Card::Heart10 => "hearts_10.svg",
            Card::Heart11 => "hearts_jack.svg",
            Card::Heart12 => "hearts_queen.svg",
            Card::Heart13 => "hearts_king.svg",
        }
    }
}
