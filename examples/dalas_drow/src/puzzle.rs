use serde::{Deserialize, Serialize};

use ws_core::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Puzzle {
    pub title: String,
    pub grid: Grid<4, 16>,
    pub words: Vec<PuzzleWord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PuzzleWord {
    pub characters: CharsArray<16>,
    pub text: Ustr,
}

impl WordTrait<4, 16> for PuzzleWord {
    fn characters(&self) -> &ArrayVec<Character, 16> {
        &self.characters
    }

    fn quiz_question(&self) -> Option<Ustr> {
        None
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

impl std::str::FromStr for PuzzleWord {
    type Err = &'static str;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = s.trim();

        let (characters, text) = word_from_str(s, 1)?;

        Ok(Self { characters, text })
    }
}

impl Puzzle {
    /// Check if this is a valid solution
    /// If so, return the index of the word
    pub fn check_solution(&self, solution: &Solution4x4) -> Option<usize> {
        let tiles: CharsArray<16> =
            ArrayVec::from_iter(solution.iter().map(|&tile| self.grid[tile]));

        let word = self.words.iter().position(|x| x.characters == tiles);

        word
    }

    pub fn to_tsv_line(&self) -> String {
        use itertools::Itertools;
        let grid = self.grid().iter().join("");
        let title = &self.title;

        let words = self
            .words()
            .iter()
            .map(|word| word.text.to_string())
            .join("\t");

        let unencoded = format!("{grid}\t{title}\t{words}");

        unencoded
    }

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

    pub fn from_tsv_line(line: &str) -> Result<Self, anyhow::Error> {
        use itertools::Itertools;
        let mut iter = line.trim_matches(char::is_whitespace).split('\t');

        let chars: &str = iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("Level '{line}' should have a grid"))?;
        let title: &str = iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("Level '{line}' should have a name"))?;

        let  grid: TileMap<Character, 4, 4, 16> = ws_core::try_make_grid(chars)
            .ok_or_else(|| anyhow::anyhow!("Level '{line}' should be able to make grid"))?;

        let words: Vec<PuzzleWord> = iter
            .map(|x| {
                <PuzzleWord as std::str::FromStr>::from_str(x.trim())
                    .map_err(|e| anyhow::anyhow!("Word '{x}' is not valid {e}"))
            })
            .try_collect()?;

        //We are not sorting words
        //words.sort();
        let mut grid_chars = grid.into_inner();
        grid_chars.sort_by_key(|x|x.as_char());
        let grid = TileMap::from_inner(grid_chars);

        Ok(Self {
            title: title.trim_end().to_string(),
            grid,
            words,
        })
    }
}

impl Puzzle {
    pub const DEFAULT_PUZZLE: Self = Self {
        title: String::new(),
        grid: Grid::from_inner([Character::Blank; 16]),
        words: vec![],
    };

    pub const DEFAULT_PUZZLE_REF: &'static Self = &Self {
        title: String::new(),
        grid: Grid::from_inner([Character::A; 16]),
        words: vec![],
    };
}

impl Default for Puzzle {
    fn default() -> Self {
        Self::DEFAULT_PUZZLE
    }
}

impl std::fmt::Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f) //todo improve
    }
}

impl LevelTrait<4, 16> for Puzzle {
    type Word = PuzzleWord;

    fn grid(&self) -> ws_core::Grid<4, 16> {
        self.grid
    }

    fn words(&self) -> &[Self::Word] {
        self.words.as_slice()
    }
}
