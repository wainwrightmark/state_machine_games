use glam::Vec2;
use leptos::{logging::log, prelude::*};
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
}

fn game_component() -> impl IntoView {
    let state = ChessState {
        board: Default::default(),
        selected_square: None,
    };

    let stores = Stores::new(&state);
    let machine = GameMachine::new(state, stores.clone());
    let command_sender1 = machine.command_sender();
    let command_sender2 = machine.command_sender();

    machine.run_game();

    view! {
        <svg viewBox="0 0 320.0 320.0"  style="max-width: 800px;  margin-inline: auto; ">
        {move || SingleTypeEntityStore::render(stores.0.clone(), command_sender1.clone())}
        {move || SingleTypeEntityStore::render(stores.1.clone(), ())}
        </svg>
        <div>
            {move || SingleTypeEntityStore::render(stores.2.clone(), command_sender2.clone())}
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
    type StateSegment = ();

    fn key(&self) -> Self::Key {
        *self
    }

    fn get_entities(_: &()) -> impl Iterator<Item = Self> {
        ButtonEntity::iter()
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
        _artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
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
    type Command = ChessCommand;
}

impl LeptosGameArtifact for ButtonArtifact {
    fn render(self, sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let command = match self {
            ButtonArtifact::PlayBestMove => ChessCommand::PlayBestMove,
            ButtonArtifact::Restart => ChessCommand::Restart,
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
pub enum ChessCommand {
    PlayBestMove,
    Restart,
    ClickSquare(Square),
}

impl GameCommand<ChessState> for ChessCommand {
    fn apply_command(&self, game_state: &mut ChessState) -> MutationResult {
        match self.clone() {
            ChessCommand::PlayBestMove => {
                let mut engine = Engine::from_board(game_state.board.clone());

                // Configure the engine to search for the best move up to a depth of 5 plies.
                let response = engine.search_depth_verbose(DEPTH);
                if let Some(best_move) = response.get_best_move() {
                    match game_state.board.push(best_move) {
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
            ChessCommand::Restart => {
                game_state.board = Board::default();
                MutationResult::CHANGED_NO_TRANSITION
            }
            ChessCommand::ClickSquare(square) => {
                match game_state.selected_square {
                    Some(selected_square) => {
                        if selected_square == square {
                            game_state.selected_square = None;
                        } else {
                            game_state.selected_square = None;
                            match Move::new(selected_square, square, None) {
                                Ok(m) => match game_state.board.push(m) {
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
                        if game_state
                            .board
                            .get_piece_at(square)
                            .is_some_and(|p| p.get_color() == game_state.board.turn())
                        {
                            game_state.selected_square = Some(square);
                        } else {
                            return MutationResult::NO_CHANGE;
                        }
                    }
                }
                MutationResult::CHANGED_NO_TRANSITION
            }
        }
    }
}

const DEPTH: i8 = 5;

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
    type Command = ChessCommand;
}

impl LeptosGameArtifact for ChessSquareArtifact {
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
                on:click= move|_|{sender.send_command(ChessCommand::ClickSquare(self.square) );}
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
    type StateSegment = Option<Square>;

    fn key(&self) -> Self::Key {
        self.square.to_int()
    }

    fn get_entities(selected_square: &Option<Square>) -> impl Iterator<Item = Self> {
        let entities = (0..64)
            .map(|i| unsafe { Square::from_int(i) })
            .map(move |square| Self {
                square,
                selected: Some(square) == selected_square.clone(),
            });

        entities
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
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
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
}

impl LeptosGameArtifact for ChessPiece {
    fn render(self, _sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = square_to_position(self.square, true);

        log!("Piece {} x: {x} y: {y}", self.piece);

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
    type StateSegment = ChessPosition;

    fn key(&self) -> Self::Key {
        *self
    }

    fn get_entities(board: &ChessPosition) -> impl Iterator<Item = Self> {
        let entities = board.occupied().flat_map(|square| {
            board
                .get_piece_at(square)
                .map(|piece| ChessPiece { square, piece })
        });
        entities
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        (*self, vec![])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
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
#[derive(Debug)]
pub struct ChessState {
    pub board: Board,
    pub selected_square: Option<Square>,
}

impl GameState for ChessState {
    type Command = ChessCommand;
    fn maybe_transition(&mut self) -> state_machine_games::prelude::MutationResult {
        MutationResult::NO_CHANGE
    }
}

impl HasSegment<Option<Square>> for ChessState {
    fn get_segment(&self) -> Option<Square> {
        self.selected_square
    }

    fn segment_eq(&self, s: &Option<Square>) -> bool {
        self.selected_square.eq(s)
    }
}

impl HasSegment<ChessPosition> for ChessState {
    fn get_segment(&self) -> ChessPosition {
        let position = self.board.get_position().clone();
        log!("POSITION\n{}", position.to_string());
        position
    }

    fn segment_eq(&self, s: &ChessPosition) -> bool {
        self.board.get_position().eq(s)
    }
}
