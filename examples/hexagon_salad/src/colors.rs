use bevy_color::Srgba;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorScheme {
    pub background_incomplete: Srgba,
    pub background_complete: Srgba,

    pub tile: Srgba,
    pub tile_letter_unselected: Srgba,
    pub tile_letter_selected: Srgba,

    pub clue_text: Srgba,

    pub lozenge_normal: Srgba,
    pub lozenge_selection_stroke: Srgba,
    pub lozenge_completed: Srgba,

    pub wordline_0: Srgba,
    pub wordline_1: Srgba,
    pub wordline_2: Srgba,
    pub wordline_3: Srgba,
    pub wordline_4: Srgba,
    pub wordline_5: Srgba,
    pub wordline_6: Srgba,
    pub wordline_7: Srgba,
    pub wordline_8: Srgba,
    pub wordline_9: Srgba,
    pub wordline_10: Srgba,
    pub wordline_11: Srgba,

    pub animated_word: Srgba
}

impl ColorScheme {
    pub fn wordline_color(self, index: usize) -> Srgba {
        match index % 12 {
            0 =>  self.wordline_0,
            1 =>  self.wordline_1,
            2 =>  self.wordline_2,
            3 =>  self.wordline_3,
            4 =>  self.wordline_4,
            5 =>  self.wordline_5,
            6 =>  self.wordline_6,
            7 =>  self.wordline_7,
            8 =>  self.wordline_8,
            9 =>  self.wordline_9,
            10 => self.wordline_10,
            11 | _ => self.wordline_11,
        }
    }
}

pub const CLASSIC_COLOR_SCHEME: ColorScheme = ColorScheme {
    background_incomplete: rgb_hex(0xf5f5f5),        // #f5f5f5ff
    background_complete: rgb_hex(0x279f4d),        // #279f4dff
    tile: rgb_hex(0xebebeb),              // #ebebebff
    tile_letter_unselected: rgb_hex(0x043e40),       // #043e40ff
    tile_letter_selected: rgb_hex(0xf5f5f5),       // #f5f5f5ff
    clue_text: rgb_hex(0x676c71),         // #676c71ff
    lozenge_normal: rgb_hex(0xebebeb),    // #ebebebff
    lozenge_selection_stroke: rgb_hex(0x676c71),         // #676c71ff
    lozenge_completed: rgb_hex(0x27bf4d), // #27bf4dff
    wordline_0: rgb_hex(0x006AFF),          // #006AFF
    wordline_1: rgb_hex(0x0015FF),          // #0015FF
    wordline_2: rgb_hex(0x4000FF),          // #4000FF
    wordline_3: rgb_hex(0x9500FF),          // #9500FF
    wordline_4: rgb_hex(0xEA00FF),          // #EA00FF
    wordline_5: rgb_hex(0xFF00BF),          // #FF00BF
    wordline_6: rgb_hex(0xFF006A),          // #FF006A
    wordline_7: rgb_hex(0xFF0015),          // #FF0015
    wordline_8: rgb_hex(0xFF4000),          // #FF4000
    wordline_9: rgb_hex(0xFF9D00),          // #FF9D00
    wordline_10: rgb_hex(0xE8AE00),         // #E8AE00
    wordline_11: rgb_hex(0xC9B900),         // #C9B900

    animated_word: rgb_hex(0x27bf4d),       // #27bf4dff
};

const fn rgb_hex(mut number: u32) -> Srgba {
    let b = number % 256;
    number /= 256;
    let g = number % 256;
    number /= 256;
    let r = number % 256;

    let blue = b as f32 / 256.0;
    let green = g as f32 / 256.0;
    let red = r as f32 / 256.0;

    Srgba {
        alpha: 1.0,
        blue,
        green,
        red,
    }
}
