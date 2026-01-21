use std::sync::mpsc::Sender;

use const_sized_bit_set::prelude::{BitSet, BitSet64};
use glam::Vec2;
use leptos::{logging::log, prelude::*};
use state_machine_games::prelude::*;
use timecat::prelude::*;

type Stores = (
    ArcRwSignal<SingleTypeEntityStore<ChessSquareEntity>>,
    ArcRwSignal<SingleTypeEntityStore<ChessPiece>>,
    ArcRwSignal<SingleTypeEntityStore<BoardAnnotation>>,
    ResourceStore<EvaluationResource>,
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
    let squares = stores.0.clone();
    let pieces = stores.1.clone();
    let annotations = stores.2.clone();
    let evaluation: RwSignal<i16> = stores.3.artifact.value.clone();
    let machine = GameMachine::new(state, stores);
    let command_sender1 = machine.command_sender();
    let command_sender2 = machine.command_sender();

    machine.run_game();

    view! {
        <svg viewBox="0 0 320.0 320.0"  style="max-width: 800px;  margin-inline: auto; ">
        {move || SingleTypeEntityStore::render(squares.clone(), (), command_sender1.clone())}
        {move || SingleTypeEntityStore::render(annotations.clone(), (), ())}
        {move || SingleTypeEntityStore::render(pieces.clone(), (), ())}


        </svg>
        <div>
            {buttons_view(command_sender2)}
            {evaluation_view(evaluation)}
        </div>

    }
}

fn evaluation_view(evaluation: RwSignal<i16>) -> impl IntoView {
    view! {
        <code>
        {evaluation}
        </code>
    }
}

fn buttons_view(command_sender: Sender<ChessCommand>) -> impl IntoView {
    let sender2 = command_sender.clone();

    let depth = RwSignal::new(5i8.to_string());

    view! {
        <input type="number" min="1" max="50" bind:value=depth />
        <button on:click=move|_|{command_sender.send_command(ChessCommand::PlayBestMove{depth: depth.get().parse::<i8>().unwrap_or_default()});}>
            {"Play Best Move"}
        </button>
        <button on:click=move|_|{sender2.send_command(ChessCommand::Restart);}>
            {"Restart"}
        </button>
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ChessCommand {
    PlayBestMove { depth: i8 },
    Restart,
    ClickSquare(Square),
}

impl GameCommand<ChessState> for ChessCommand {
    fn apply_command(&self, game_state: &mut ChessState) -> MutationResult {
        match self.clone() {
            ChessCommand::PlayBestMove { depth } => {
                let mut engine = Engine::from_board(game_state.board.clone());

                // Configure the engine to search for the best move up to a depth of 5 plies.
                let response = engine.search_depth_verbose(depth);
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

//const DEPTH: i8 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChessSquareArtifact {
    pub square: Square,
}

impl GameArtifact for ChessSquareArtifact {}

impl LeptosGameArtifact for ChessSquareArtifact {
    type Command = ChessCommand;
    fn render(self, _: (), sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = square_to_position(self.square, false);
        let fill = if self.square.get_rank().to_int() % 2 == self.square.get_file().to_int() % 2 {
            "#739552"
        } else {
            "#ebecd0"
        };
        view! {
            <rect x=x y=y width=SQUARE_SIZE height=SQUARE_SIZE fill={fill}
                on:click= move|_|{sender.send_command(ChessCommand::ClickSquare(self.square) );}
             />
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChessSquareEntity {
    pub square: Square,
}

impl GameEntity for ChessSquareEntity {
    type Artifact = ChessSquareArtifact;
    type Key = u8;
    type StateSegment = ();

    fn key(&self) -> Self::Key {
        self.square.to_int()
    }

    fn get_entities(_: &()) -> impl Iterator<Item = Self> {
        let entities = (0..64)
            .map(|i| unsafe { Square::from_int(i) })
            .map(move |square| Self { square });

        entities
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        ChessSquareArtifact {
            square: self.square,
        }
        .with_animations([])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        AnimationList::EMPTY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BoardAnnotation {
    pub square: u8,
    pub selected: bool,
    pub non_capture_move: bool,
    pub capture_move: bool,
}

impl GameArtifact for BoardAnnotation {}

impl LeptosGameArtifact for BoardAnnotation {
    type Command = ();
    fn render(self, _: (), sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = square_index_to_position(self.square, false);
        let Vec2 { x: cx, y: cy } = square_index_to_position(self.square, false)
            + Vec2 {
                x: SQUARE_SIZE * 0.5,
                y: SQUARE_SIZE * 0.5,
            };

        let selected = if self.selected {
            Some(view! {
                <rect x=x y=y width=SQUARE_SIZE height=SQUARE_SIZE fill="#AABB00" style="pointer-events: none"/>
            })
        } else {
            None
        };

        let capture_dot = if self.capture_move {
            Some(view! {
                <circle cx=cx cy=cy r={SQUARE_SIZE * 0.5} height=SQUARE_SIZE fill="#666666" style="pointer-events: none"/>
            })
        } else {
            None
        };

        let non_capture_dot = if self.non_capture_move {
            Some(view! {
                <circle cx=cx cy=cy r={SQUARE_SIZE * 0.25} height=SQUARE_SIZE fill="#666666" style="pointer-events: none"/>
            })
        } else {
            None
        };

        view! {
            {selected}
            {capture_dot}
            {non_capture_dot}
        }
    }
}

impl GameEntity for BoardAnnotation {
    type Artifact = Self;
    type Key = Self;
    type StateSegment = BoardAnnotationsSegment;

    fn key(&self) -> Self::Key {
        *self
    }

    fn get_entities(segment: &Self::StateSegment) -> impl Iterator<Item = Self> {
        let tiles = segment
            .selected
            .with_union(&segment.captures.with_union(&segment.non_captures));

        tiles.into_iter().map(|tile| {
            let selected = segment.selected.contains_const(tile);
            let non_capture_move = segment.non_captures.contains_const(tile);
            let capture_move = segment.captures.contains_const(tile);
            Self {
                square: tile as u8,
                selected,
                non_capture_move,
                capture_move,
            }
        })
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        self.with_animations([])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        *artifact = *self;
        previous_animations
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

impl GameArtifact for ChessPiece {}

impl LeptosGameArtifact for ChessPiece {
    type Command = ();
    fn render(self, _: (), _sender: impl CommandSender<Self::Command>) -> impl IntoView {
        let Vec2 { x, y } = square_to_position(self.square, true);

        //log!("Piece {} x: {x} y: {y}", self.piece);

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
        self.with_animations([])
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        _former_entity_state: EntityState,
        _previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        *artifact = *self;
        AnimationList::EMPTY
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

pub fn square_index_to_position(square: u8, center: bool) -> Vec2 {
    let square = unsafe { Square::from_int(square) };
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

#[derive(Debug, Clone, PartialEq)]
pub struct BoardAnnotationsSegment {
    pub selected: BitSet64,
    pub captures: BitSet64,
    pub non_captures: BitSet64,
}

impl HasSegment<BoardAnnotationsSegment> for ChessState {
    fn get_segment(&self) -> BoardAnnotationsSegment {
        let mut selected = BitSet64::EMPTY;
        let mut captures = BitSet64::EMPTY;
        let mut non_captures = BitSet64::EMPTY;

        if let Some(selected_square) = self.selected_square {
            selected.insert_const(selected_square as u32);

            let lm = self.board.generate_legal_moves();

            for m in lm.iter() {
                if m.get_source() == selected_square {
                    let dest = m.get_dest();
                    if self.board.get_piece_at(dest).is_some() {
                        captures.insert_const(dest as u32);
                    } else {
                        non_captures.insert_const(dest as u32);
                    }
                }
            }
        }

        if let Some(m) = self.board.get_last_stack_move() {
            for x in [m.get_source(), m.get_dest()].iter().flatten() {
                selected.insert_const(*x as u32);
            }
        }

        BoardAnnotationsSegment {
            selected,
            captures,
            non_captures,
        }
    }

    fn segment_eq(&self, s: &BoardAnnotationsSegment) -> bool {
        <Self as HasSegment<BoardAnnotationsSegment>>::get_segment(&self) == *s
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

#[derive(Debug, Clone, Default)]
pub struct EvaluationResource {
    pub value: RwSignal<i16>,
}

impl GameArtifact for EvaluationResource {}

impl ResourceValue for EvaluationResource {
    type Segment = ChessPosition;

    type GS = ChessState;

    fn update_value(
        segment: &Self::Segment,
        artifact: &mut Self,
        _animations: &mut AnimationList<Self>,
    ) -> bool {
        artifact.value.set(segment.slow_evaluate());
        true
    }
}
