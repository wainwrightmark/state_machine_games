use glam::Vec2;

use crate::game_state::CardPosition;

//let total_height = 

const CARD_WIDTH: f32 = 26.0;
const CARD_HEIGHT: f32 = 37.0;
const HORIZONTAL_GAP: f32 = 3.0;
const VERTICAL_GAP: f32 = 3.0;

const TOTAL_WIDTH: f32 = CARD_WIDTH * 8.0 + HORIZONTAL_GAP * 9.0;

const TOP_OFFSET: f32 = 10.0;

const fn tile_position(column: u8, row: u8, )-> Vec2{
    let x = HORIZONTAL_GAP + ((HORIZONTAL_GAP + CARD_WIDTH) * column as f32);

    let y = TOP_OFFSET + ((CARD_HEIGHT + VERTICAL_GAP) * row as f32);

    Vec2{x,y}
}


pub fn get_position(p: CardPosition)-> Vec2{
    panic!()
    // match p{
    //     CardPosition::CardsUp { suit } => {
            
    //     },
    //     CardPosition::Stack { stack_index, row_index } => todo!(),
    //     CardPosition::TopCells(index) => {

    //     },
    // }
}