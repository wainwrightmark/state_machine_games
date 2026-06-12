use bevy_color::prelude::Srgba;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use shaped_word_grid_generator::*;

use crate::SolutionType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SuperSaladPuzzlesState {
    pub puzzles: Vec<Puzzle>,
}

// impl FromStr for SuperSaladPuzzlesState {
//     type Err = anyhow::Error;

//     fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
//         let mut puzzles = vec![];

//         for line in s.lines() {
//             let puzzle = Puzzle::from_tsv_line(line)?;
//             puzzles.push(puzzle);
//         }
//         Ok(Self { puzzles })
//     }
// }

// impl Display for SuperSaladPuzzlesState {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for p in self.puzzles.iter() {
//             f.write_str(&p.to_tsv_line().as_str())?;
//             f.write_str("\n")?;
//         }

//         Ok(())
//     }
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Puzzle {
    pub name: Ustr,
    pub extra_info: Option<Ustr>,
    pub special_characters: SpecialCharacters,
    pub special_colors: Option<Vec<Srgba>>,
    pub grid: Grid<19>,
    pub words: Vec<PuzzleWord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PuzzleWord {
    pub characters: CharsArray<19>,
    pub text: Ustr,
    pub clue: Option<Ustr>,
}

impl PuzzleWord {
    pub fn clue_with_number(&self) -> String {
        let hidden_text = self.hidden_text();

        match self.clue {
            Some(clue) => format!("{clue} ({hidden_text})"),
            None => format!("({hidden_text})"),
        }
    }

    pub fn from_string(
        mut s: &str,
        special_characters: &SpecialCharacters,
    ) -> Result<Self, &'static str> {
        let mut characters: ArrayVec<Character, 19> = Default::default();

        let clue: Option<Ustr> = {
            if s.ends_with(']') {
                if let Some(open_bracket_index) = s.find('[') {
                    let (start, end) = s.split_at(open_bracket_index);
                    s = start;
                    Some(end.trim_start_matches('[').trim_end_matches(']').into())
                } else {
                    return Err("Word contains ']' but not '['");
                }
            } else {
                None
            }
        };

        let iter = NormalizedCharacterIterator::new(s, special_characters);

        for r in iter {
            match r {
                NormalizedCharacterResult::Error(err) => {
                    return Err(err);
                }
                NormalizedCharacterResult::RegularCharacter {
                    character: inner, ..
                } => {
                    characters.try_push(inner).map_err(|_| "Word is too long")?;
                }
                NormalizedCharacterResult::Blank { .. } => {}
            }
        }

        // if characters.len() <= 3 {
        //     return Err("Word has 3 or fewer characters");
        // }

        Ok(Self {
            characters,
            text: Ustr::from(s),
            clue,
        })
    }
}

impl WordTrait<19> for PuzzleWord {
    fn characters(&self) -> &ArrayVec<Character, 19> {
        &self.characters
    }

    fn quiz_question(&self) -> Option<Ustr> {
        self.clue
    }
}

impl PartialOrd for PuzzleWord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PuzzleWord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.characters
            .iter()
            .map(|x| x.as_char())
            .cmp(other.characters.iter().map(|x| x.as_char()))
    }
}

impl BasicWordTrait for PuzzleWord {
    fn text(&self) -> Ustr {
        self.text
    }

    fn characters_slice(&self) -> &[Character] {
        &self.characters
    }
}

// impl std::str::FromStr for PuzzleWord {
//     type Err = &'static str;

//     fn from_str(mut s: &str) -> Result<Self, Self::Err> {
//         s = s.trim();
//         let clue: Option<Ustr> = if s.ends_with(']') {
//             s = s.trim_end_matches(']');
//             match s.split_once('[') {
//                 Some((prefix, suffix)) => {
//                     s = prefix;
//                     Some(suffix.into())
//                 }
//                 None => None,
//             }
//         } else {
//             None
//         };

//         let (characters, text) = word_from_str(&s.to_uppercase(), 1)?;

//         Ok(Self {
//             characters,
//             text,
//             clue,
//         })
//     }
// }

impl Puzzle {
    /// Check if this is a valid solution
    /// If so, return the index of the word
    pub fn check_solution(&self, solution: &SolutionType) -> Option<usize> {
        let tiles: CharsArray<19> =
            ArrayVec::from_iter(solution.iter().map(|&tile| self.grid[tile]));

        let word = self.words.iter().position(|x| x.characters == tiles);

        word
    }

    // pub fn to_tsv_line(&self) -> String {
    //     use itertools::Itertools;
    //     let grid = self.grid().iter().join("");
    //     let title = &self.name;
    //     let category = &self.category;
    //     let words = self
    //         .words()
    //         .iter()
    //         .map(|word| match word.clue {
    //             Some(clue) => {
    //                 format!("{}[{clue}]", word.text)
    //             }
    //             None => word.text.to_string(),
    //         })
    //         .join("\t");

    //     let unencoded = format!("{grid}\t{title}[{category}]\t{words}");

    //     unencoded
    // }

    pub fn try_from_encoded(data: &str) -> Option<Self> {
        use base64::Engine;

        let data = match base64::engine::general_purpose::URL_SAFE.decode(&data) {
            Ok(d) => d,
            Err(err) => {
                if matches!(err, base64::DecodeError::InvalidPadding) {
                    match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data) {
                        Ok(d) => d,
                        Err(err) => {
                            leptos::logging::error!("{err}");
                            return None;
                        }
                    }
                } else {
                    leptos::logging::error!("{err}");
                    return None;
                }
            }
        };

        let data = String::from_utf8(data).ok()?;

        leptos::logging::log!("Data: {data}");

        match Self::from_tsv_line(data.trim()) {
            Ok(data) => Some(data),
            Err(err) => {
                leptos::logging::error!("{err}");
                None
            }
        }
    }

    pub fn from_tsv_line(line: &str) -> Result<Self, String> {
        let mut iter = line.trim_matches(char::is_whitespace).split('\t');

        let grid_chars: &str = iter
            .next()
            .ok_or_else(|| format!("Level '{line}' should have a grid"))?;
        let name: &str = iter
            .next()
            .ok_or_else(|| format!("Level '{line}' should have a name"))?;

        let (special_characters, grid_chars) =
            SpecialCharacters::extract_from_grid_characters(grid_chars);

        let grid = try_make_grid(grid_chars)
            .ok_or_else(|| format!("Level '{line}' should be able to make grid"))?;

        //use itertools::Itertools;
        let words: Vec<PuzzleWord> = iter
            .map(|x| {
                PuzzleWord::from_string(x.trim(), &special_characters)
                    .map_err(|e| format!("Word '{x}' is not valid {e}"))
            })
            .try_collect()?;

        // if sort_words { //don't sort words
        //     words.sort();
        // }

        let mut name = name;

        let special_colors = if name.ends_with('}') {
            if let Some(index) = name.find('{') {
                let (prefix, colors) = name.split_at(index);
                name = prefix.trim_end();
                let colors = &colors[1..(colors.len() - 1)];
                let mut colors_vec = Vec::<bevy_color::prelude::Srgba>::default();

                for c in colors.split(',') {
                    if let Ok(color) = bevy_color::prelude::Srgba::hex(c) {
                        colors_vec.push(color);
                    } else {
                        // wasm_logger::lo ::warn!("Could not parse color '{c}'");
                    }
                }
                if colors_vec.is_empty() {
                    None
                } else {
                    Some(colors_vec)
                }
            } else {
                None
            }
        } else {
            None
        };

        let extra_info = if name.ends_with(']') {
            if let Some(index) = name.find('[') {
                let (prefix, extra_info) = name.split_at(index);
                name = prefix.trim_end();
                let extra_info = Ustr::from(&extra_info[1..(extra_info.len() - 1)]);
                Some(extra_info)
            } else {
                None
            }
        } else {
            None
        };

        let name = Ustr::from(name.trim_end());

        Ok(Self {
            name,

            special_characters,
            extra_info,
            grid,
            words,
            special_colors,
        })
    }
}

impl Default for Puzzle {
    fn default() -> Self {
        Self {
            name: Ustr::from(""),
            grid: Grid([Character::Blank; 19]),
            words: vec![],
            extra_info: None,
            special_characters: SpecialCharacters::NONE,
            special_colors: None,
        }
    }
}

impl std::fmt::Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f) //todo improve
    }
}

impl LevelTrait<19> for Puzzle {
    type Word = PuzzleWord;
    type Layout = crate::LayoutType;

    fn grid(&self) -> shaped_word_grid_generator::Grid<19> {
        self.grid
    }

    fn grid_mut(&mut self) -> &mut Grid<19> {
        &mut self.grid
    }

    fn words(&self) -> &[Self::Word] {
        self.words.as_slice()
    }

    fn extra_info(&self) -> Option<Ustr> {
        self.extra_info
    }

    fn special_colors(&self) -> Option<&[bevy_color::prelude::Srgba]> {
        match &self.special_colors {
            Some(sc) => Some(sc.as_slice()),
            None => None,
        }
    }
}
