use shaped_word_grid_generator::{GridTile, grid_layout::GridLayout};
use state_machine_games::prelude::Vec2;
use strum::EnumIs;

use crate::{LayoutType, layout_util};

pub const GAME_WIDTH: f32 = 640f32;
pub const GAME_HEIGHT: f32 = 1136f32;

pub const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;

//pub const BOARD_SIZE: f32 = 600f32;

pub const BOARD_WIDTH: f32 = 5.0 * TILE_WIDTH;

pub const BOARD_HEIGHT: f32 = 4.0 * TILE_HEIGHT;// BOARD_WIDTH * 0.5 * SQRT_3;


pub const TILE_RADIUS: f32 = 60.0;
pub const TILE_HEIGHT: f32 = TILE_RADIUS * 2.0;
pub const TILE_WIDTH: f32 = TILE_RADIUS * SQRT_3;

pub const TILE_PROPORTION: f32 = 0.8;

pub const LEFT_OFFSET: f32 = (GAME_WIDTH - BOARD_WIDTH) / 2.0;
pub const TOP_OFFSET: f32 = (GAME_HEIGHT - BOARD_HEIGHT) / 2.0;

pub const LOZENGE_WIDTH: f32 = 40.0;
pub const LOZENGE_HEIGHT: f32 = LOZENGE_WIDTH * 2.0;
pub const LOZENGE_RADIUS: f32 = 25.0;

pub const CLUE_FONT_SIZE: f32 = 40.0;
pub const FONT_FAMILY: &'static str = "Montserrat";

//pub const TILE_LETTER_FONT_SIZE: f32 = TILE_RADIUS * 0.8;
pub const PATH_STROKE_WIDTH: f32 = 1.3 * TILE_RADIUS;

pub const ANIMATED_WORD_FONT_SIZE: f32 = 40.0;

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum PositionOrigin {
    TopLeft,
    Center,
}

/// The position of the  tile
pub fn offset_tile_position(tile: GridTile, origin: PositionOrigin) -> Vec2 {
    LayoutType::tile_position(tile, TILE_RADIUS * 2.0, origin.is_center())
        + Vec2 {
            x: LEFT_OFFSET,
            y: TOP_OFFSET ,
        }
}

pub const fn postgame_tile_position(
    index: usize,
    word_length: usize,
    origin: PositionOrigin,
) -> Vec2 {
    // let scale = if word_length > 4 {
    //     word_length as f32
    // } else {
    //     4.0
    // } / 4.0;

    let mut x = LEFT_OFFSET
        + layout_util::Spacing::SpaceBetween.apply(
            BOARD_WIDTH,
            TILE_WIDTH * TILE_PROPORTION,
            word_length as f32,
            index as f32,
        );

    let mut y = TOP_OFFSET + (BOARD_HEIGHT * 0.5);

    if matches!(origin, PositionOrigin::Center) {
        x = x + (TILE_WIDTH * 0.5);
        y = y + (TILE_HEIGHT * 0.5);
    }

    Vec2 { x, y }
}

pub const fn lozenge_position(index: usize, lozenge_count: usize, origin: PositionOrigin) -> Vec2 {
    let mut x = LEFT_OFFSET
        + layout_util::Spacing::SpaceBetween.apply(
            BOARD_WIDTH - ((1.0 - TILE_PROPORTION) * TILE_WIDTH),
            LOZENGE_WIDTH,
            lozenge_count as f32,
            index as f32,
        );

    let mut y = GAME_HEIGHT - LOZENGE_HEIGHT - 20.0; //todo reposition

    if matches!(origin, PositionOrigin::Center) {
        x = x + (LOZENGE_WIDTH * 0.5);
        y = y + (LOZENGE_HEIGHT * 0.5);
    }

    Vec2 { x, y }
}
