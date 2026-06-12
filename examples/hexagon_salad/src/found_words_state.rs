use std::num::NonZeroUsize;

use const_sized_bit_set::prelude::{BitSet, FiniteBitSet};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use shaped_word_grid_generator::{grid_layout::GridLayout, prelude::*};
use strum::EnumIs;

use crate::{DesignedLevelType, LayoutType, NUM_TILES, SolutionType};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FoundWordsState {
    pub unneeded_tiles: GridSet,
    pub word_completions: Vec<Completion>,
    pub hints_used: usize,
}

impl FoundWordsState {
    fn update_unneeded_tiles(&mut self, level: &DesignedLevel<{ NUM_TILES }, LayoutType>) {
        self.unneeded_tiles = level.calculate_unneeded_tiles(self.unneeded_tiles, |index| {
            self.word_completions
                .get(index)
                .map(|x| x.is_complete())
                .unwrap_or(true)
        });
    }

    /// replays the found words in order to get the same set of unneeded tiles
    pub fn recalculate_unneeded_tiles(&mut self, level: &impl LevelTrait<19>) {
        let mut new_tiles = Default::default();
        for finish_index in self
            .word_completions
            .iter()
            .filter_map(|completion| match completion {
                Completion::Complete { index } => Some(*index),
                _ => None,
            })
            .sorted_by_key(|x| *x)
        {
            new_tiles = level.calculate_unneeded_tiles(new_tiles, |index| {
                self.word_completions
                    .get(index)
                    .map(|completion| {
                        if let Completion::Complete { index } = completion {
                            index <= &finish_index
                        } else {
                            false
                        }
                    })
                    .unwrap_or(true)
            });
        }

        self.unneeded_tiles = new_tiles;
    }

    /// Inadvisable tiles are tiles that are selectable, but can't lead to a solution
    pub fn calculate_inadvisable_tiles(
        &self,
        current_solution: &SolutionType,
        level: &DesignedLevelType,
    ) -> GridSet {
        let mut selectable = match current_solution.last() {
            Some(tile) => LayoutType::ADJACENCIES[tile.inner_usize()],
            None => GridSet::ALL,
        };

        for tile in current_solution {
            selectable.remove_const(tile.inner_u32());
        }

        let mut inadvisable = selectable.with_except(&self.unneeded_tiles);

        let chosen_characters: ArrayVec<Character, NUM_TILES> =
            current_solution.iter().map(|x| level.grid[*x]).collect();

        let mut slices = self
            .word_completions
            .iter()
            .zip(level.words.iter())
            .map(|(completion, word)| completion.known_characters(word))
            //dedup to remove consecutive `None`
            .dedup()
            .peekable();

        let mut predecessor: Option<Character> = None;
        //let mut count = 0;

        while let Some(slice) = slices.next() {
            //   count += 1;
            //todo check length of this word
            if let Some(slice) = slice {
                if !could_precede(slice, &chosen_characters) {
                    //info!("Went past prefix after {count}");
                    return inadvisable;
                }

                if slice.starts_with(&chosen_characters) {
                    predecessor = slice.iter().skip(chosen_characters.len()).cloned().next();
                }
                continue;
            }

            let mut successor: Option<Character> = None;

            if let Some(Some(slice)) = slices.peek() {
                if !could_precede(&chosen_characters, slice) {
                    continue;
                }
                if slice.starts_with(&chosen_characters) {
                    successor = slice.iter().skip(chosen_characters.len()).cloned().next();
                }
            }
            if predecessor.is_none() && successor.is_none() {
                //info!("No pre or successor");
                return GridSet::EMPTY;
            }

            'tiles: for tile in inadvisable.clone().iter() {
                let character = level.grid[tile as usize];
                if let Some(p) = predecessor {
                    if p.as_char() > character.as_char() {
                        continue 'tiles;
                    }
                }
                if let Some(s) = successor {
                    if s.as_char() < character.as_char() {
                        continue 'tiles;
                    }
                }
                inadvisable.remove_const(tile);
            }
            if inadvisable.is_empty() {
                //info!("All bits unset");
                return GridSet::EMPTY;
            }
        }
        //info!("Checked all words {count}");
        inadvisable
    }

    #[allow(dead_code)]
    fn count_inevitable_characters(&self, level: &DesignedLevelType, word_index: usize) -> usize {
        if let Some(completion) = self.word_completions.get(word_index) {
            let prefix_characters = match completion {
                Completion::Unstarted => 0,
                Completion::ManualHinted(a) => a.get(),
                Completion::Complete { .. } => return 0,
            };

            let preceder: &[Character] = self
                .word_completions
                .iter()
                .zip(level.words.iter())
                .take(word_index)
                .flat_map(|(c, w)| c.known_characters(w))
                .next_back()
                .unwrap_or_default();

            let successor: &[Character] = self
                .word_completions
                .iter()
                .zip(level.words.iter())
                .skip(word_index)
                .flat_map(|(c, w)| c.known_characters(w))
                .next()
                .unwrap_or_default();

            if let Some(letters) = level
                .words
                .get(word_index)
                .and_then(|x| NonZeroUsize::new(x.characters.len()))
            {
                let (initial_tiles, preceder, successor) = if prefix_characters == 0 {
                    (self.unneeded_tiles.with_negated(), preceder, successor)
                } else {
                    let preceder = if prefix_characters > preceder.len() {
                        &[]
                    } else {
                        preceder.split_at(prefix_characters).1
                    };
                    let successor = if prefix_characters > successor.len() {
                        &[]
                    } else {
                        successor.split_at(prefix_characters).1
                    };

                    let Some(w) = level.words.get(word_index) else {
                        return 0;
                    };
                    let Some(solution) = w.find_solution::<LayoutType>(level.grid) else {
                        return 0;
                    };

                    let Some(tile) = solution.get(prefix_characters.saturating_sub(1)) else {
                        return 0;
                    };

                    (
                        LayoutType::ADJACENCIES[tile.inner_usize()]
                            .with_except(&self.unneeded_tiles),
                        preceder,
                        successor,
                    )
                };

                let hints = initial_tiles
                    .iter()
                    .flat_map(|tile| {
                        count_hints(
                            GridTile(tile as u8),
                            &level.grid,
                            self.unneeded_tiles,
                            preceder,
                            successor,
                            letters,
                        )
                    })
                    .exactly_one();

                if let Ok(hints) = hints {
                    return hints;
                }
            }
        }

        0
    }

    pub fn most_recently_completed_word<'a, L: LevelTrait<{ NUM_TILES }>>(
        &self,
        level: &'a L,
    ) -> Option<(usize, &'a L::Word)> {
        let (position, _) = self
            .word_completions
            .iter()
            .enumerate()
            .filter_map(|(position, completion)| match completion {
                Completion::Unstarted => None,
                Completion::ManualHinted(_) => None,
                Completion::Complete { index } => Some((position, index)),
            })
            .max_by_key(|(_position, index)| *index)?;

        let word = level.words().get(position)?;

        Some((position, word))
    }

    pub fn ensure_valid(&mut self, level: &DesignedLevelType) {
        //make sure this is a valid state, otherwise act defensively

        self.word_completions.truncate(level.words.len());
        let needed = level
            .words
            .len()
            .saturating_sub(self.word_completions.len());
        for _ in 0..needed {
            self.word_completions.push(Completion::Unstarted);
        }

        for (completion, word) in self.word_completions.iter_mut().zip(level.words.iter()) {
            match completion {
                Completion::Unstarted => {}
                Completion::ManualHinted(n) => {
                    if n.get() > word.characters.len() {
                        *n = NonZeroUsize::MIN;
                    }
                }
                Completion::Complete { .. } => {}
            }
        }
        self.unneeded_tiles = Default::default();
        self.update_unneeded_tiles(level);
    }

    pub fn completed_words_ordered(&self, level: &DesignedLevelType) -> String {
        let mut result = String::default();

        for word_index in self
            .word_completions
            .iter()
            .enumerate()
            .filter_map(|(word_index, completion)| match completion {
                Completion::Unstarted => None,
                Completion::ManualHinted(_) => None,
                Completion::Complete { index } => Some((word_index, index)),
            })
            .sorted_by_key(|x| x.1)
            .map(|x| x.0)
        {
            if let Some(word) = level.words.get(word_index) {
                if !result.is_empty() {
                    result.push_str(", ");
                }
                result.push_str(word.text.as_str());
            }
        }

        result
    }

    pub fn new_from_level(level: &impl LevelTrait<{ NUM_TILES }>) -> Self {
        Self {
            unneeded_tiles: GridSet::EMPTY,
            word_completions: vec![Completion::Unstarted; level.words().len()],
            hints_used: 0,
        }
    }

    pub fn new_level_complete(level: &impl LevelTrait<{ NUM_TILES }>, hints_used: usize) -> Self {
        Self {
            unneeded_tiles: GridSet::ALL,
            word_completions: vec![Completion::Complete { index: 0 }; level.words().len()],
            hints_used,
        }
    }

    /// Grid with unneeded characters blanked
    fn adjusted_grid(&self, level: &impl LevelTrait<{ NUM_TILES }>) -> Grid<NUM_TILES> {
        let mut grid = level.grid();

        for tile in self.unneeded_tiles.iter() {
            grid[tile as usize] = Character::Blank;
        }

        grid
    }

    pub fn manual_hint_set(&self, level: &DesignedLevelType, solution: &SolutionType) -> GridSet {
        self.hint_set::<true>(level, solution)
    }

    fn hint_set<const MANUAL: bool>(
        &self,
        level: &DesignedLevelType,
        solution: &SolutionType,
    ) -> GridSet {
        let mut set = GridSet::default();
        let adjusted_grid = self.adjusted_grid(level);

        if solution.is_empty() {
            //hint all known first letters
            for (word, completion) in level.words.iter().zip(self.word_completions.iter()) {
                if !(
                    MANUAL && completion.is_manual_hinted()
                    // || (!MANUAL && completion.is_auto_hinted())
                ) {
                    continue;
                }

                if let Some(solution) = word.find_solution::<LayoutType>(adjusted_grid) {
                    if let Some(first) = solution.first() {
                        set.insert_const(first.inner_u32());
                    }
                }
            }
        } else {
            // hint all solutions starting with this
            for (word, completion) in level.words.iter().zip(self.word_completions.iter()) {
                let hints = match (completion, MANUAL) {
                    (Completion::ManualHinted(hints), true) => hints,
                    _ => {
                        continue;
                    }
                };

                if let Some(word_solution) = word.find_solution::<LayoutType>(adjusted_grid) {
                    let len = hints.get().min(solution.len());

                    if solution.iter().take(len).eq(word_solution.iter().take(len)) {
                        for tile in word_solution.iter().take(hints.get()) {
                            set.insert(tile.inner_u32());
                        }
                    }
                }
            }
        }
        set
    }

    pub fn is_level_complete(&self) -> bool {
        self.word_completions.iter().all(|x| x.is_complete())
    }

    pub fn is_level_started(&self) -> bool {
        self.word_completions.iter().any(|x| !x.is_unstarted())
    }

    /// Get completion for work with the given index.
    /// Returns `Complete` if index is out of range
    pub fn get_completion(&self, word_index: usize) -> Completion {
        *self
            .word_completions
            .get(word_index)
            .unwrap_or(&Completion::Complete { index: 0 })
    }

    // fn try_hint_word(
    //     &mut self,
    //     hint_state: &mut HintState,
    //     level: &DesignedLevelType,
    //     word_index: usize,
    //     chosen_state: &mut ChosenState,
    //     ew: &mut impl AnyEventWriter<WordFoundEvent>,
    //     popup_state: &mut PopupState,
    //     should_spend_hints: bool,
    //     haptics_settings: &GeneralSettings,
    //     use_custom_colors: bool,
    // ) -> bool {
    //     let new_hints = if should_spend_hints {
    //         let Some(new_hints) = hint_state.hints_remaining.checked_sub(1) else {
    //             popup_state.0 = Some(PopupType::BuyMoreHints(HintEvent { word_index }));
    //             return false;
    //         };
    //         Some(new_hints)
    //     } else {
    //         None
    //     };

    //     let Some(word) = level.words.get(word_index) else {
    //         return false;
    //     };

    //     let min_hint_count = NonZeroUsize::MIN.saturating_add(self.count_selected_characters(
    //         level,
    //         word_index,
    //         chosen_state,
    //     ));

    //     let completion_index = self
    //         .word_completions
    //         .iter()
    //         .filter(|x| x.is_complete())
    //         .count() as u8;
    //     let Some(completion) = self.word_completions.get_mut(word_index) else {
    //         return false;
    //     };

    //     let new_count = match completion {
    //         Completion::Unstarted => {
    //             *completion = Completion::ManualHinted(min_hint_count);
    //             self.hints_used += 1;
    //             if let Some(new_hints) = new_hints {
    //                 hint_state.hints_remaining = new_hints;
    //             }

    //             min_hint_count.get()
    //         }
    //         Completion::ManualHinted(hints) => {
    //             if hints.get() >= word.characters.len() {
    //                 return false;
    //             }
    //             if let Some(new_hints) = new_hints {
    //                 hint_state.hints_remaining = new_hints;
    //                 hint_state.has_ever_double_hinted = true;
    //             }

    //             *hints = hints.saturating_add(1);

    //             if min_hint_count > *hints {
    //                 *hints = min_hint_count;
    //             }

    //             self.hints_used += 1;

    //             hints.get()
    //         }
    //         Completion::Complete { .. } => return false,
    //     };

    //     if let Some(solution) = word
    //         .find_solutions_with_tiles(level.grid, self.unneeded_tiles)
    //         .next()
    //     {
    //         let new_selection: ArrayVec<GridTile, NUM_TILES> =
    //             ArrayVec::from_iter(solution.iter().take(new_count).cloned());
    //         chosen_state.solution = new_selection;

    //         if solution.len() > new_count {
    //             chosen_state.is_just_finished = false;
    //         } else {
    //             //do not select the full word - let the user do that

    //             *completion = Completion::Complete {
    //                 index: completion_index,
    //             };
    //             self.update_unneeded_tiles(level);

    //             ew.send(WordFoundEvent {
    //                 solution,
    //                 is_first_time: true,
    //                 was_hinted: true,
    //                 word: word.clone(),
    //                 level: level.clone(),
    //                 use_custom_colors,
    //             });

    //             chosen_state.is_just_finished = true; //todo change this slightly
    //         }
    //     } else {
    //         warn!("Could not find solution during hint");
    //     }

    //     crate::haptics::HapticEvent::UseHint.try_activate(&haptics_settings);

    //     true
    // }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize, EnumIs, Default)]
pub enum Completion {
    #[default]
    Unstarted,
    // AutoHinted(NonZeroUsize),
    ManualHinted(NonZeroUsize),
    Complete {
        /// the number of previously completed words
        index: u8,
    },
}

impl Completion {
    // pub fn color(&self) -> &'static Color {
    //     const UNSTARTED: &Color = &Color::Srgba(palette::WORD_BACKGROUND_UNSTARTED);
    //     const MANUAL: &Color = &Color::Srgba(palette::WORD_BACKGROUND_MANUAL_HINT);
    //     const COMPLETE: &Color = &Color::Srgba(palette::WORD_BACKGROUND_COMPLETE);

    //     match self {
    //         Completion::Unstarted => UNSTARTED,
    //         Completion::ManualHinted(_) => MANUAL,
    //         Completion::Complete { .. } => COMPLETE,
    //     }
    // }

    pub fn known_characters<'w>(
        &self,
        word: &'w impl WordTrait<NUM_TILES>,
    ) -> Option<&'w [Character]> {
        let characters = word.characters();
        match self {
            Completion::Unstarted => None,
            Completion::Complete { .. } => Some(&characters),
            Completion::ManualHinted(hints) => {
                Some(characters.split_at(hints.get().min(characters.len())).0)
            }
        }
    }
}

fn could_precede(p: &[Character], s: &[Character]) -> bool {
    for (p, s) in p.iter().zip(s.iter()) {
        match p.as_char().cmp(&s.as_char()) {
            std::cmp::Ordering::Less => return true,
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => return false,
        }
    }

    true
}

/// If this doesn't come between the preceder and succeeder, return None
/// If there is exactly one child, which returns a value greater than zero, return that value + 1
/// Otherwise return zero
#[allow(dead_code)]
fn count_hints(
    tile: GridTile,
    grid: &Grid<19>,
    unneeded_tiles: GridSet,
    preceder: &[Character],
    successor: &[Character],
    remaining_letters: NonZeroUsize,
) -> Option<usize> {
    let character = grid[tile];

    let next_preceder = match preceder.split_first() {
        Some((c, next)) => match character.as_char().cmp(&c.as_char()) {
            std::cmp::Ordering::Less => return None,
            std::cmp::Ordering::Equal => next,
            std::cmp::Ordering::Greater => &[],
        },
        None => &[],
    };

    let next_succeeder = match successor.split_first() {
        Some((c, next)) => match character.as_char().cmp(&c.as_char()) {
            std::cmp::Ordering::Less => &[],
            std::cmp::Ordering::Equal => next,
            std::cmp::Ordering::Greater => return None,
        },
        None => &[],
    };

    let next_unneeded = unneeded_tiles.with_inserted(tile.inner_u32());
    let next_remaining_letters = match remaining_letters
        .get()
        .checked_sub(1)
        .and_then(NonZeroUsize::new)
    {
        Some(r) => r,
        None => {
            return Some(1);
        }
    };

    let child = LayoutType::ADJACENCIES[tile.inner_usize()]
        .with_except(&unneeded_tiles)
        .iter_const()
        .flat_map(|next_tile| {
            count_hints(
                GridTile(next_tile as u8),
                grid,
                next_unneeded,
                next_preceder,
                next_succeeder,
                next_remaining_letters,
            )
        })
        .exactly_one();

    match child {
        Ok(count) => Some(count.saturating_add(1)),
        Err(error) => {
            if error.count() == 0 {
                //no possible children
                None
            } else {
                //multiple children so this is as far as it goes
                Some(1)
            }
        }
    }
}
