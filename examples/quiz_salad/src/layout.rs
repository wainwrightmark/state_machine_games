use state_machine_games::prelude::Vec2;
use ws_core::Tile4x4;

use crate::layout_util;

pub const GAME_WIDTH: f32 = 640f32;
pub const GAME_HEIGHT: f32 = 1136f32;

pub const BOARD_SIZE: f32 = 600f32;
pub const TILE_SIZE: f32 = (BOARD_SIZE / 4.0) * 0.9;
pub const TILE_RADIUS: f32 = TILE_SIZE * 0.05;

pub const LEFT_OFFSET: f32 = (GAME_WIDTH - BOARD_SIZE) / 2.0;
pub const TOP_OFFSET: f32 = (GAME_HEIGHT - BOARD_SIZE) / 2.0;

pub const LOZENGE_WIDTH: f32 = 50.0;
pub const LOZENGE_HEIGHT: f32 = LOZENGE_WIDTH * 2.0;
pub const LOZENGE_RADIUS: f32 = 30.0;

pub const CLUE_FONT_SIZE: f32 = 40.0;
pub const FONT_FAMILY: &'static str = "Montserrat";


const SCALE: f32 = 160.0;
pub const FONT_SIZE: f32 = SCALE * 0.5;
pub const PATH_STROKE_WIDTH: f32 = SCALE * 0.6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionOrigin {
    TopLeft,
    Center,
}

/// The position of the center of a tile
pub const fn tile_position(tile: Tile4x4, origin: PositionOrigin) -> Vec2 {
    let x = tile.x() as f32;
    let y = tile.y() as f32;

    let mut x =
        LEFT_OFFSET + layout_util::Spacing::SpaceBetween.apply(BOARD_SIZE, TILE_SIZE, 4.0, x);
    let mut y =
        TOP_OFFSET + layout_util::Spacing::SpaceBetween.apply(BOARD_SIZE, TILE_SIZE, 4.0, y);

    if matches!(origin, PositionOrigin::Center) {
        x = x + (TILE_SIZE * 0.5);
        y = y + (TILE_SIZE * 0.5);
    }

    Vec2 { x, y }
}

pub const fn lozenge_position(index: usize, lozenge_count: usize, origin: PositionOrigin) -> Vec2 {
    let mut x = LEFT_OFFSET
        + layout_util::Spacing::SpaceBetween.apply(
            BOARD_SIZE,
            LOZENGE_WIDTH,
            lozenge_count as f32,
            index as f32,
        );

    let mut y = GAME_HEIGHT - LOZENGE_HEIGHT - 20.0; //todo reposition

    if matches!(origin, PositionOrigin::Center) {
        x = x - (LOZENGE_WIDTH * 0.5);
        y = y - (LOZENGE_HEIGHT * 0.5);
    }

    Vec2 { x, y }
}
