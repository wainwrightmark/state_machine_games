use arrayvec::ArrayVec;
use leptos::ev::{DragEvent, PointerEvent};

use crate::{
    card::{Card, Rank, Suit},
    game_state::{
        CardPosition, FreecellCommand, FreecellGameState, MoveFromPosition, MoveToPosition,
    },
    *,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CardEntity {
    pub card: Card,
    //pub show_all: bool,
    pub position: CardPosition,
}

impl GameEntity for CardEntity {
    type Artifact = CardArtifact;
    type Key = Card;
    type StateSegment = FreecellGameState;

    fn key(&self) -> Self::Key {
        self.card
    }

    fn get_entities(segment: &Self::StateSegment) -> impl Iterator<Item = Self> {
        //todo use an array, use the card as an index
        let mut cards: ArrayVec<CardEntity, 52> = ArrayVec::new();
        for (suit_index, max_rank) in segment.cards_up.iter().enumerate() {
            if let Some(suit) = Suit::from_index(suit_index as u8) {
                for rank in 1..=*max_rank {
                    if let Some(rank) = Rank::from_repr(rank) {
                        let card = Card::from_suit_and_rank(suit, rank);
                        cards.push(CardEntity {
                            card,
                            //show_all: true,
                            position: CardPosition::CardsUp { suit },
                        });
                    }
                }
            }
        }

        for (index, card) in segment.top_cells.iter().enumerate() {
            cards.push(CardEntity {
                card: *card,
                //show_all: true,
                position: CardPosition::TopCells(index as u8),
            });
        }

        for (stack_index, stack) in segment.stacks.iter().enumerate() {
            for (row_index, card) in stack.cards.iter().enumerate() {
                cards.push(CardEntity {
                    card: *card,
                    //show_all: true,
                    position: CardPosition::Stack {
                        stack_index: stack_index as u8,
                        row_index: row_index as u8,
                    },
                });
            }
        }

        cards.sort_by_key(|x| x.card);
        cards.into_iter()
    }

    fn on_new(&self) -> (Self::Artifact, AnimationList<Self::Artifact>) {
        let artifact = CardArtifact {
            position: RwSignal::new(self.position.position()),
            card: self.card,
            //show_all: RwSignal::new(self.show_all),
            from: RwSignal::new(self.position.as_from_position()),
            to: RwSignal::new(self.position.as_to_position()),
        };

        (artifact, Default::default())
    }

    fn on_update(
        &self,
        artifact: &mut Self::Artifact,
        former_entity_state: EntityState,
        previous_animations: AnimationList<Self::Artifact>,
    ) -> AnimationList<Self::Artifact> {
        artifact.position.set(self.position.position());
        //artifact.show_all.set(self.show_all);
        artifact.from.set(self.position.as_from_position());
        artifact.to.set(self.position.as_to_position());

        Default::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CardArtifact {
    pub position: RwSignal<Vec2>,
    pub card: Card,
    //pub show_all: RwSignal<bool>,
    pub from: RwSignal<MoveFromPosition>,
    pub to: RwSignal<MoveToPosition>,
}

impl GameArtifact for CardArtifact {}

impl LeptosGameArtifact for CardArtifact {
    type Command = FreecellCommand;

    fn render(
        self,
        _argument: (),
        sender: impl state_machine_games::prelude::CommandSender<Self::Command>,
    ) -> impl IntoView {
        let on_drag_start = move |ev: DragEvent| {
            if let Some(dt) = ev.data_transfer() {
                if let Ok(data) = serde_json::to_string(&self.from.get_untracked()) {
                    let _ = dt.set_data("text/plain", &data);
                }
            }
        };

        let on_drop = move |ev: DragEvent| {
            ev.prevent_default();
            if let Some(dt) = ev.data_transfer() {
                if let Ok(data) = dt.get_data("text/plain") {
                    if let Ok(from) = serde_json::from_str::<MoveFromPosition>(&data) {
                        sender.send_command(FreecellCommand::MoveCard {
                            from,
                            to: self.to.get_untracked(),
                        });
                    }
                }
            }
        };

        view! {
              <image
              x=move ||{self.position.get().x}
              y=move ||{self.position.get().y}
              width="26"
              height="37"
              href="recursion.svg"
              on:drag_start=on_drag_start
              on:drop = on_drop
              />

        }
    }
}
