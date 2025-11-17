use ws_core::{ArrayVec, Grid4x4, Tile4x4};

use crate::{chosen_state::ChosenState, found_words_state::FoundWordsState};

#[derive(Debug, Default)]
pub struct GridInputState {
    last_tile: Option<Tile4x4>,
    multi_click: Option<MultiClick>,
    last_truncate: Option<Tile4x4>,
    started_empty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MultiClick {
    DeleteOnEndThenStop,
    DeleteOnEndThenMaybeSwitch(Tile4x4),
    SwitchOnStart(Tile4x4),
}

impl GridInputState {
    pub fn handle_input_start_no_location(&mut self) {
        self.started_empty = true;
    }

    pub fn handle_input_start(
        &mut self,
        chosen_state: &mut ChosenState,
        tile: Tile4x4,
        grid: &Grid4x4,
        found_words: &FoundWordsState,
    ) {
        self.started_empty = false;
        if self.last_tile == Some(tile) {
            self.multi_click = Some(MultiClick::DeleteOnEndThenStop);
            return;
        }

        let next_multi_click: Option<MultiClick>;

        self.last_tile = Some(tile);

        if chosen_state.is_just_finished {
            *chosen_state = ChosenState::default();
            self.last_truncate = None;
        }

        if let Some(last) = chosen_state.solution.last() {
            if let Some(index) = chosen_state.solution.iter().position(|x| *x == tile) {
                // element is already present
                if index + 1 == chosen_state.solution.len() {
                    if Some(tile) == self.last_truncate {
                        chosen_state.solution.clear();
                        chosen_state.solution.push(tile);
                        self.last_truncate = None;
                        //info!("His1a index: {index}");
                        next_multi_click = None;
                    } else {
                        next_multi_click = Some(MultiClick::DeleteOnEndThenMaybeSwitch(tile));
                        //info!("His1b index: {index}");
                    }

                    self.last_truncate = None;
                } else if index == 0 {
                    //info!("His2 index: {index}");
                    chosen_state.solution.clear();
                    self.last_truncate = None;
                    next_multi_click = None;
                } else {
                    //info!("His3 index: {index}");
                    chosen_state.solution.truncate(index + 1);

                    //info!("His3 index: {index}  cs len {}");
                    self.last_truncate = Some(tile);
                    next_multi_click = None;
                }
            } else if last.is_adjacent_to(&tile) {
                //element is not already present
                if allow_tile(tile, grid, found_words) {
                    //info!("His4");
                    if self.multi_click == Some(MultiClick::SwitchOnStart(tile)) {
                        chosen_state.solution = ArrayVec::from_iter([tile]);
                    } else {
                        chosen_state.solution.push(tile);
                    }
                }
                next_multi_click = None;
            } else {
                //info!("His5");
                *chosen_state = ChosenState::default();
                next_multi_click = None;
            }
        } else {
            next_multi_click = None;
            //array is empty
            if allow_tile(tile, grid, found_words) {
                //info!("His5");
                chosen_state.solution.push(tile);
            }
        }

        self.multi_click = next_multi_click;
    }

    pub fn handle_input_move(
        &mut self,
        chosen_state: &mut ChosenState,
        tile: Tile4x4,
        grid: &Grid4x4,
        found_words: &FoundWordsState,
    ) {
        self.started_empty = false;
        if self.last_tile == Some(tile) {
            return;
        }
        self.multi_click = None;
        self.last_tile = Some(tile);

        if chosen_state.is_just_finished {
            *chosen_state = ChosenState::default();
            self.last_truncate = None;
        }

        if let Some(last) = chosen_state.solution.last() {
            if let Some(index) = chosen_state.solution.iter().position(|x| *x == tile) {
                //info!("Him1");
                // element is already present
                chosen_state.solution.truncate(index + 1);
                self.last_truncate = None;
            } else if last.is_adjacent_to(&tile) {
                //element is not already present
                if allow_tile(tile, grid, found_words) {
                    //info!("Him2");
                    chosen_state.solution.push(tile);
                    self.last_truncate = None;
                }
            }
        }
    }

    pub fn handle_input_end(
        &mut self,
        chosen_state: &mut ChosenState,
        location: Tile4x4,
    ) {
        self.started_empty = false;
        if self.last_tile == Some(location) {
            match self.multi_click {
                Some(MultiClick::DeleteOnEndThenStop) => {
                    //info!("hie1");
                    chosen_state.solution.pop();
                    self.multi_click = None;
                }
                Some(MultiClick::DeleteOnEndThenMaybeSwitch(tile)) => {
                    chosen_state.solution.pop();
                    self.multi_click = if tile == location {
                        //info!("hie2a");
                        Some(MultiClick::SwitchOnStart(tile))
                    } else {
                        //info!("hie2b");
                        None
                    };
                }
                _ => {
                    //info!("hie3");
                    self.multi_click = None;
                }
            }
        } else {
            //info!("hie4");
            self.multi_click = None;
        }
        self.last_tile = None;
    }

    pub fn handle_input_end_no_location(
        &mut self,
        chosen_state: &mut ChosenState,
        is_level_complete: bool,
    ) {
        if self.started_empty && !is_level_complete && !chosen_state.solution.is_empty() {
            chosen_state.solution.clear();
        }

        self.started_empty = false;
        self.last_tile = None;
        self.multi_click = None;
    }
}

fn allow_tile(tile: Tile4x4, grid: &Grid4x4, found_words: &FoundWordsState) -> bool {
    if grid[tile].is_blank() {
        false
    } else {
        !found_words.unneeded_tiles.get_bit(&tile)
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use itertools::Itertools;
    use test_case::test_case;
    use ws_core::{Character, Grid, Solution4x4, Tile};
    use super::*;

    #[test_case("", "")]
    #[test_case("s00", "00")]
    #[test_case("s00 e s01", "00 01")]
    #[test_case("s00 e s02", "")]
    #[test_case("s00 m01", "00 01")]
    #[test_case("s00 m01 e", "00 01")]
    #[test_case("s00 m01 m02 e", "00 01 02")]
    #[test_case("s00 m11 m22 e", "00 11 22")]
    #[test_case("s00 m11 m22 m11 e ", "00 11")]
    #[test_case("s00 m01 m02 e s01", "00 01")]
    #[test_case("s00 m01 m02 e s01 e s01", "01")]
    #[test_case("s00 m01 m02 e s03 e03 s03 e03 s03", "03")]
    pub fn test_inputs(input: &str, expected: &str) {
        let input_list = parse_input_list(input);
        let expected = parse_expected_list(expected);

        let mut state = GridInputState::default();

        let found_words = FoundWordsState::default();

        let mut chosen_state = ChosenState::default();

        let grid = Grid::from_fn(|_| Character::A);

        for input in input_list.into_iter() {
            match input {
                Input::EndNoLocation => {
                    state.handle_input_end_no_location(&mut chosen_state, false)
                }
                Input::Start(tile) => {
                    state.handle_input_start(&mut chosen_state, tile, &grid, &found_words)
                }
                Input::Move(tile) => {
                    state.handle_input_move(&mut chosen_state, tile, &grid, &found_words)
                }
                Input::End(tile) => state.handle_input_end(&mut chosen_state, tile),
            }
        }

        assert_eq!(chosen_state.solution, expected);
    }

    fn parse_input_list(input: &str) -> Vec<Input> {
        input
            .split_ascii_whitespace()
            .map(|x| Input::from_str(x).unwrap())
            .collect_vec()
    }

    fn parse_expected_list(expected: &str) -> Solution4x4 {
        Solution4x4::from_iter(
            expected
                .split_ascii_whitespace()
                .map(|x| try_parse_tile(x).unwrap()),
        )
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Input {
        EndNoLocation,
        Start(Tile4x4),
        Move(Tile4x4),
        End(Tile4x4),
    }

    impl FromStr for Input {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s == "E" || s == "e" {
                return Ok(Self::EndNoLocation);
            }
            let (command, tile) = s.split_at(1);

            let tile = try_parse_tile(tile)?;

            match command {
                "s" | "S" => Ok(Self::Start(tile)),
                "m" | "M" => Ok(Self::Move(tile)),
                "e" | "E" => Ok(Self::End(tile)),
                _ => Err("Unrecognized command"),
            }
        }
    }

    fn try_parse_tile(s: &str) -> Result<Tile4x4, &'static str> {
        if s.len() != 2 {
            return Err("Tile string should be two characters");
        }

        let (col, row) = s.split_at(1);

        let col: u8 = col.parse().map_err(|_| "Could not parse column")?;
        let row: u8 = row.parse().map_err(|_| "Could not parse row")?;

        Tile::try_new(col, row).ok_or("Tile out of range")
    }
}
