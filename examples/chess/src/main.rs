use std::sync::mpsc;

use glam::Vec2;
use leptos::prelude::*;
use state_machine_games::prelude::*;
use strum::{EnumIter, IntoEnumIterator};
use timecat::prelude::*;

type Stores = (
    ArcRwSignal<SingleTypeEntityStore<ChessSquareEntity>>,
    ArcRwSignal<SingleTypeEntityStore<ChessPiece>>,
    ArcRwSignal<SingleTypeEntityStore<ButtonEntity>>,
);

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(|| game_component());
    // // Initialize a chess board with the default starting position.
    // let mut board = Board::default();

    // //bo

    // //ValidOrNullMove::new(source, dest, promotion)

    // // Apply moves in standard algebraic notation.
    // board.push_san("e4").expect("Failed to make move: e4");
    // board.push_san("e5").expect("Failed to make move: e5");

    // // Evaluate the current board position using the inbuilt_nnue feature.
    // let evaluation = board.evaluate();
    // println!("Current Evaluation: {}\n", evaluation);

    // // Initialize the engine with the current board state.
    // let mut engine = Engine::from_board(board);

    // // Configure the engine to search for the best move up to a depth of 10 plies.
    // let response = engine.search_depth_verbose(10);
    // let best_move = response.get_best_move().expect("No best move found");

    // //board.get_piece_at(square)

    // // Output the best move found by the engine.
    // println!(
    //     "\nBest Move: {}",
    //     best_move
    //         .san(engine.get_board())
    //         .expect("Failed to generate SAN")
    // );
}

fn game_component() -> impl IntoView {
    let state = ChessState {
        board: Default::default(),
        selected_square: None,
    };

    let (click_sender, click_receiver) = mpsc::channel::<ClickSquareCommand>();
    let (button_sender, button_receiver) = mpsc::channel::<ChessButtonCommand>();
    let stores = Stores::default();
    let machine = GameMachine::new(state, stores.clone(), (click_receiver, button_receiver));

    machine.run_game();

    view! {
        <svg viewBox="0 0 320.0 320.0"  style="max-width: 800px;  margin-inline: auto; ">
        {move || SingleTypeEntityStore::render(stores.0.clone(), click_sender.clone())}
        {move || SingleTypeEntityStore::render(stores.1.clone(), ())}
        </svg>
        <div>
            {move || SingleTypeEntityStore::render(stores.2.clone(), button_sender.clone())}
        </div>

    }
}

#[derive(Debug, PartialEq, Hash, Eq, PartialOrd, Ord, Clone, Copy, EnumIter)]
pub enum ButtonEntity {
    PlayBestMove,
    Restart,
}

impl GameEntityKey for ButtonEntity {}

impl GameEntity for ButtonEntity {
    type Artifact = ButtonArtifact;
    type Key = Self;

    type GameState = ChessState;

    fn key(&self) -> Self::Key {
        *self
    }

    fn get_entities(_game_state: &Self::GameState) -> impl Iterator<Item = Self> {
        ButtonEntity::iter()
    }

    fn on_death(&self, _artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact> {
        vec![]
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let artifact = match self {
            ButtonEntity::PlayBestMove => ButtonArtifact::PlayBestMove,
            ButtonEntity::Restart => ButtonArtifact::Restart,
        };
        (artifact, vec![])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        vec![]
    }
}

#[derive(Debug, Clone)]
pub enum ButtonArtifact {
    PlayBestMove,
    Restart,
}

impl GameArtifact for ButtonArtifact {
    type Command = ChessButtonCommand;

    fn render(self, sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let command = match self {
            ButtonArtifact::PlayBestMove => ChessButtonCommand::PlayBestMove,
            ButtonArtifact::Restart => ChessButtonCommand::Restart,
        };
        let name = match self {
            ButtonArtifact::PlayBestMove => "Play Best Move",
            ButtonArtifact::Restart => "Restart",
        };

        view! {
            <button on:click=move|_|{sender.send_command(command);}>
                {name}
            </button>
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ChessButtonCommand {
    PlayBestMove,
    Restart,
}

const DEPTH: i8 = 5;

impl GameCommand<ChessState> for ChessButtonCommand {
    fn apply_command(&self, games_state: &mut ChessState) -> MutationResult {
        games_state.selected_square = None;
        match self {
            ChessButtonCommand::PlayBestMove => {
                let mut engine = Engine::from_board(games_state.board.clone());

                // Configure the engine to search for the best move up to a depth of 5 plies.
                let response = engine.search_depth_verbose(DEPTH);
                if let Some(best_move) = response.get_best_move() {
                    match games_state.board.push(best_move) {
                        Ok(()) => MutationResult::CHANGED_NO_TRANSITION,
                        Err(err) => {
                            leptos::logging::error!("{err}");
                            MutationResult::NO_CHANGE
                        }
                    }
                } else {
                    MutationResult::NO_CHANGE
                }
            }
            ChessButtonCommand::Restart => {
                games_state.board = Board::default();
                MutationResult::CHANGED_NO_TRANSITION
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClickSquareCommand {
    pub square: Square,
}

impl GameCommand<ChessState> for ClickSquareCommand {
    fn apply_command(&self, games_state: &mut ChessState) -> MutationResult {
        match games_state.selected_square {
            Some(selected_square) => {
                if selected_square == self.square {
                    games_state.selected_square = None;
                } else {
                    games_state.selected_square = None;
                    match Move::new(selected_square, self.square, None) {
                        Ok(m) => match games_state.board.push(m) {
                            Ok(()) => {}
                            Err(err) => {
                                leptos::logging::log!("Error pushing move: {err}");
                            }
                        },
                        Err(err) => {
                            leptos::logging::log!("Error creating move: {err}");
                        }
                    }
                }
            }
            None => {
                if games_state
                    .board
                    .get_piece_at(self.square)
                    .is_some_and(|p| p.get_color() == games_state.board.turn())
                {
                    games_state.selected_square = Some(self.square);
                } else {
                    return MutationResult::NO_CHANGE;
                }
            }
        }
        MutationResult::CHANGED_NO_TRANSITION
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChessSquareArtifact {
    pub square: Square,
    pub selected: RwSignal<bool>,
}

// impl Ord for ChessSquare {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.square
//             .to_int()
//             .cmp(&other.square.to_int())
//             .then(self.selected.cmp(&other.selected))
//     }
// }

// impl PartialOrd for ChessSquare {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(Ord::cmp(self, other))
//     }
// }

impl GameArtifact for ChessSquareArtifact {
    type Command = ClickSquareCommand;

    fn render(self, sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = square_to_position(self.square, false);
        let fill = if self.square.get_rank().to_int() % 2 == self.square.get_file().to_int() % 2 {
            "#739552"
        } else {
            "#ebecd0"
        };
        let stroke_width = move || if self.selected.get() { 5 } else { 0 };
        view! {
            <rect x=x y=y width=SQUARE_SIZE height=SQUARE_SIZE fill={fill} stroke = {"#000000"} stroke-width={stroke_width}
                on:click= move|_|{sender.send_command(ClickSquareCommand { square: self.square });}
             />

        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChessSquareEntity {
    pub square: Square,
    pub selected: bool,
}

impl GameEntity for ChessSquareEntity {
    type Artifact = ChessSquareArtifact;
    type Key = u8;
    type GameState = ChessState;

    fn key(&self) -> Self::Key {
        self.square.to_int()
    }

    fn get_entities(game_state: &Self::GameState) -> impl Iterator<Item = Self> {
        let selected_square = game_state.selected_square;
        let entities = (0..64)
            .map(|i| unsafe { Square::from_int(i) })
            .map(move |square| Self {
                square,
                selected: Some(square) == selected_square,
            });

        entities
    }

    fn on_death(&self, artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact> {
        vec![]
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (
            ChessSquareArtifact {
                square: self.square,
                selected: RwSignal::new(self.selected),
            },
            vec![],
        )
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        artifact.selected.set(self.selected);
        vec![]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChessPiece {
    pub square: Square,
    pub piece: Piece,
}

impl Ord for ChessPiece {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.square
            .to_int()
            .cmp(&other.square.to_int())
            .then(self.piece.cmp(&other.piece))
    }
}

impl PartialOrd for ChessPiece {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl GameEntityKey for ChessPiece {}

impl GameArtifact for ChessPiece {
    type Command = ();

    fn render(self, _sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = square_to_position(self.square, true);

        view! {
            <text x=x y=y font-size={40} style="pointer-events: none;user-select: none;font-family: monospace;dominant-baseline: central;text-anchor: middle;">
                {piece_to_char(&self.piece)}
            </text>
        }
    }
}

impl GameEntity for ChessPiece {
    type Artifact = Self;
    type Key = Self;
    type GameState = ChessState;

    fn key(&self) -> Self::Key {
        *self
    }

    fn get_entities(game_state: &Self::GameState) -> impl Iterator<Item = Self> {
        let entities = game_state.board.occupied().flat_map(|square| {
            game_state
                .board
                .get_piece_at(square)
                .map(|piece| ChessPiece { square, piece })
        });
        entities
    }

    fn on_death(&self, artifact: &mut Self::Artifact) -> AnimationList<Self::Artifact> {
        vec![]
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (*self, vec![])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
    ) -> AnimationList<Self::Artifact> {
        *artifact = *self;
        vec![]
    }
}

const SQUARE_SIZE: f32 = 40.0;
pub fn square_to_position(square: Square, center: bool) -> Vec2 {
    let x = square.get_file().to_int() as f32 * SQUARE_SIZE;
    let y = (7 - square.get_rank().to_int()) as f32 * SQUARE_SIZE;

    let mut r = Vec2 { x, y };
    if center {
        r += Vec2::splat(SQUARE_SIZE * 0.5);
    }
    r
}

pub fn piece_to_char(piece: &Piece) -> char {
    //♙♖♜♘♞♗♝♕♔♚
    match (piece.get_color(), piece.get_piece_type()) {
        (Black, Pawn) => '♟',
        (Black, Knight) => '♞',
        (Black, Bishop) => '♝',
        (Black, Rook) => '♜',
        (Black, Queen) => '♛',
        (Black, King) => '♚',
        (White, Pawn) => '♙',
        (White, Knight) => '♘',
        (White, Bishop) => '♗',
        (White, Rook) => '♖',
        (White, Queen) => '♕',
        (White, King) => '♔',
    }
}

pub struct ChessState {
    pub board: Board,
    pub selected_square: Option<Square>,
}

impl GameState for ChessState {
    fn maybe_transition(&mut self) -> state_machine_games::prelude::MutationResult {
        MutationResult::NO_CHANGE
    }
}
