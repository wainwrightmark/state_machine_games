use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use ws_core::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SuperSaladPuzzlesState {
    pub puzzles: Vec<Puzzle>,
}

impl FromStr for SuperSaladPuzzlesState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut puzzles = vec![];

        for line in s.lines() {
            let puzzle = Puzzle::from_tsv_line(line)?;
            puzzles.push(puzzle);
        }
        Ok(Self { puzzles })
    }
}

impl Display for SuperSaladPuzzlesState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for p in self.puzzles.iter() {
            f.write_str(&p.to_tsv_line().as_str())?;
            f.write_str("\n")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Puzzle {
    pub category: String,
    pub title: String,
    pub grid: Grid<4, 16>,
    pub words: Vec<PuzzleWord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PuzzleWord {
    pub characters: CharsArray<16>,
    pub text: Ustr,
    pub clue: Option<Ustr>,
}

impl WordTrait<4, 16> for PuzzleWord {
    fn characters(&self) -> &ArrayVec<Character, 16> {
        &self.characters
    }

    fn quiz_question(&self)-> Option<Ustr> {
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

impl std::str::FromStr for PuzzleWord {
    type Err = &'static str;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = s.trim();
        let clue: Option<Ustr> = if s.ends_with(']') {
            s = s.trim_end_matches(']');
            match s.split_once('[') {
                Some((prefix, suffix)) => {
                    s = prefix;
                    Some(suffix.into())
                }
                None => None,
            }
        } else {
            None
        };

        let (characters, text) = word_from_str(s, 1)?;

        Ok(Self {
            characters,
            text,
            clue,
        })
    }
}

impl Puzzle {

    /// Check if this is a valid solution
    /// If so, return the index of the word
    pub fn check_solution(&self, solution: &Solution4x4)-> Option<usize>{
        let tiles: CharsArray<16> = ArrayVec::from_iter(
                solution.iter().map(|&tile| self.grid[tile])
        );

        let word = self.words.iter().position(|x|  x.characters == tiles);

        word
    }

    pub fn to_tsv_line(&self) -> String {
        use itertools::Itertools;
        let grid = self.grid().iter().join("");
        let title = &self.title;
        let category = &self.category;
        let words = self
            .words()
            .iter()
            .map(|word| match word.clue {
                Some(clue) => {
                    format!("{}[{clue}]", word.text)
                }
                None => word.text.to_string(),
            })
            .join("\t");

        let unencoded = format!("{grid}\t{title}[{category}]\t{words}");

        unencoded
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

        let grid = ws_core::try_make_grid(chars)
            .ok_or_else(|| anyhow::anyhow!("Level '{line}' should be able to make grid"))?;

        let words: Vec<PuzzleWord> = iter
            .map(|x| {
                <PuzzleWord as std::str::FromStr>::from_str(x.trim())
                    .map_err(|e| anyhow::anyhow!("Word '{x}' is not valid {e}"))
            })
            .try_collect()?;

        //We are not sorting words
        //words.sort();

        let mut title = title;

        let category = if title.ends_with(']') {
            if let Some(index) = title.find('[') {
                let (prefix, category) = title.split_at(index);
                title = prefix.trim_end();

                category[1..(category.len() - 1)].to_string()
            } else {
                "???".to_string()
            }
        } else {
            "???".to_string()
        };

        Ok(Self {
            title: title.trim_end().to_string(),
            grid,
            words,
            category,
        })
    }
}

impl Puzzle {
    pub const DEFAULT_PUZZLE: Self = Self {
        category: String::new(),
        title: String::new(),
        grid: Grid::from_inner([Character::Blank; 16]),
        words: vec![],
    };

    pub const DEFAULT_PUZZLE_REF: &'static Self = &Self {
        category: String::new(),
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
