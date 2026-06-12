use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct GameChoice {
    pub name: String,
    pub color: String,
    pub url_prefix: String,
    pub level_groups: Vec<LevelGroup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelGroup {
    pub name: String,
    pub levels: Vec<Level>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub name: String,
    pub url: String,
}

pub fn main_menu_component(choices: RwSignal<Vec<GameChoice>>) -> impl IntoView {
    let choices = leptos::control_flow::For(ForProps {
        each: move || choices.get(),
        key: |game_choice| game_choice.url_prefix.clone(),
        children: |game_choice| {
            game_choice_component(game_choice)
        },
    });

    view! {
        {choices}
    }
}

fn game_choice_component(choice: GameChoice)-> impl IntoView{
    let style = format!("background: {}; width: 100vw; height; 50vw;", choice.color);
    view! {
        <div style=style>
            <h1>{choice.name} </h1>
        </div>
    }
}
