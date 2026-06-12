use serde::{Deserialize, Serialize};
use shaped_word_grid_generator::prelude::*;

use crate::{SolutionType, found_words_state::FoundWordsState};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChosenState {
    pub solution: SolutionType,
    pub word_just_found: bool
}

impl ChosenState {
    
    pub fn current_solution(&self) -> &SolutionType {
        &self.solution
    }

    /// Is length >=6 and Damerau–Levenshtein distance to a solution <= 1 and same first letter

    pub fn is_close_to_a_solution(
        &self,
        level: &impl LevelTrait<19>,
        found_words: &FoundWordsState,
    ) -> bool {
        let solution = self.current_solution();
        if solution.len() < 5 {
            return false;
        }

        let chars: CharsArray<19> = solution.iter().map(|t| level.grid()[*t]).collect();

        for (word, completion) in level.words().iter().zip(found_words.word_completions.iter()) {
            if completion.is_complete() {
                continue;
            }
            if word.characters().first() == chars.first()
                && Self::lev_distance_one_or_less(&chars, word.characters())
            {
                return true;
            }
        }

        false
    }

    fn lev_distance_one_or_less(l_chars: &CharsArray<19>, r_chars: &CharsArray<19>) -> bool {
        let difference = l_chars.len().abs_diff(r_chars.len());
        let mut changes = 0;
        if difference == 0 {
            let mut iter = l_chars.iter().zip(r_chars.iter()).peekable();

            while let Some((l, r)) = iter.next() {
                if l != r {
                    if changes > 0 {
                        return false;
                    }
                    changes += 1;
                    if let Some((l2, r2)) = iter.peek() {
                        if &l == r2 && &r == l2 {
                            //transposition
                            iter.next();
                        }
                    }
                }
            }
        } else if difference == 1 {
            // one character was added or removed
            let (long, short) = if l_chars.len() > r_chars.len() {
                (l_chars, r_chars)
            } else {
                (r_chars, l_chars)
            };

            let mut long = long.iter();
            let mut short = short.iter();

            while let Some(l) = long.next() {
                if let Some(s) = short.next() {
                    if l != s {
                        if changes > 0 {
                            return false;
                        }
                        changes += 1;
                        if let Some(l) = long.next() {
                            if l != s {
                                return false;
                            }
                        }
                    }
                }
            }
        } else {
            return false;
        }

        true
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use test_case::test_case;

    #[test_case("a", "b", true)]
    #[test_case("ab", "b", true)]
    #[test_case("a", "ba", true)]
    #[test_case("a", "bb", false)]
    #[test_case("ab", "ba", true)]
    #[test_case("abc", "abb", true)]
    #[test_case("acc", "abb", false)]
    #[test_case("abab", "baba", false)]
    fn test_difference(l: &str, r: &str, expected: bool) {
        let l = normalize_characters_array(l, &SpecialCharacters::NONE).unwrap();
        let r = normalize_characters_array(r, &SpecialCharacters::NONE).unwrap();

        let actual = ChosenState::lev_distance_one_or_less(&l, &r);

        if expected {
            assert!(actual, "Should return true")
        } else {
            assert!(!actual, "Should not return true")
        }
    }
}
