use std::time::Duration;


use geometrid::{prelude::TileMap, vector::Vector};
use leptos::{ prelude::*};
use rand::{Rng, SeedableRng, };
use state_machine_games::prelude::*;
use state_machine_games::leptos::prelude::*;
use strum::{EnumCount, EnumIs, FromRepr};

pub type Match3Tile = geometrid::tile::Tile<8, 8>;
pub type Match3Grid = geometrid::tile_map::TileMap<Option<Gem>, 8, 8, 64>;

pub fn main(){
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    mount_to_body(app);
}

pub fn app() -> impl IntoView {
    let (state_signal, write_signal) = signal(Match3Game::new_random(123));
    let (settings, _) = signal(());
    let (assets, _) = signal(());
    let (storage, _) = signal(());

    view! {
        <main>
        {game_view_component(800.0, 800.0, state_signal, write_signal, settings, assets, storage)}
        </main>
    }
}

#[derive(Debug, Clone)]
pub struct Match3Game {
    pub score: u64,
    pub moves_left: u64,
    pub next_index: u32,


    pub next_tiles: [Gem; 8],

    pub grid: Match3Grid,

    pub selected_tile: Option<Match3Tile>,

    pub rng_state: TinyRng
}

impl Match3Game{
    pub fn new_random(seed: u64) -> Self {        
        let mut rng_state = TinyRng::seed_from_u64(seed);
        let mut  next_index = 0;
        
        let arr = [None; 64];

        

        let next_tiles = std::array::from_fn(|_|{
            let gem = Gem{
                gem_type: GemType::new_random(&mut rng_state),
                index: next_index
            };
            next_index += 1;
            gem
        } );

        Self {
            grid: TileMap::from_inner(arr),
            selected_tile: None,
            score: 0,
            moves_left: 3,
            next_index,
            next_tiles,
            rng_state
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gem {
    index: u32,
    gem_type: GemType,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromRepr, EnumCount)]
pub enum GemType {
    Blue,Yellow,Orange,Pink,Green,Purple
}

impl GemType{
    pub fn fill(self)-> &'static str{
        match self{
            GemType::Blue => "#01befe",
            GemType::Yellow => "#ffdd00",
            GemType::Orange => "#ff7d00",
            GemType::Pink => "#ff006d",
            GemType::Green => "#adff02",
            GemType::Purple => "#8f00ff",
        }
    }


    pub fn new_random(rng: &mut impl Rng)-> Self{
        let i = rng.random_range(0..Self::COUNT);
        GemType::from_repr(i).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, EnumIs)]
pub enum Match3Entity {
    ScoreText(u64),
    MovesLeftText(u64),
    HiddenTile{
        index: u32,
        gem_type: GemType,
        column_index: u8,

    },
    Tile{
        index: u32,
        tile: Match3Tile,    
        gem_type: GemType,
        selected: bool,
    }  
}

impl GameEntity for Match3Entity{
    type GameState = Match3Game ;

    fn key(&self) -> Match3EntityKey {

        match self{
            Match3Entity::ScoreText(..) => Match3EntityKey::ScoreText,
            Match3Entity::MovesLeftText(..) => Match3EntityKey::MovesLeftText,
            Match3Entity::Tile { index, .. } | Match3Entity::HiddenTile { index,.. } => Match3EntityKey::Tile{index: *index},
        }

        
    }

    fn death_duration(&self)-> Option<std::time::Duration> {
        Some(Duration::from_secs(1))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Match3Command{
    TileClicked(Match3Tile),
}

impl GameCommand for Match3Command{

}

#[derive(Debug, Clone, Copy, PartialEq,  PartialOrd, Eq, Ord, Hash)]
pub enum Match3EntityKey{
    ScoreText,
    MovesLeftText,
    Tile{index: u32}
}

impl GameEntityKey for Match3EntityKey{

}


impl GameState for Match3Game {
    type Settings = ();
    type Assets = ();
    type Storage = ();
    type Command = Match3Command;
    type Entity = Match3Entity;
    type EntityKey = Match3EntityKey;

    fn get_entities(
        &self,
        _settings: &Self::Settings,
        _assets: &Self::Assets,
        _storage: &Self::Storage,
    ) -> impl Iterator<Item = Self::Entity> {

        let tiles = self.grid.enumerate().flat_map(|(tile, gem)|
        if let Some(Gem { index, gem_type: tile_type }) = *gem{
            Some(Match3Entity::Tile { tile, index: index, gem_type: tile_type, selected: Some(tile) == self.selected_tile })
        }else {
            None
        }
        
        
         );
        let texts = [Match3Entity::MovesLeftText(self.moves_left), Match3Entity::ScoreText(self.score)];
        let hidden_tiles = self.next_tiles.iter().enumerate().map(|(column_index, gem)| Match3Entity::HiddenTile { index: gem.index, gem_type: gem.gem_type, column_index: column_index as u8 });
        texts.into_iter().chain(hidden_tiles).chain(tiles)
    }

    fn maybe_transition(
            &mut self,
            _settings: &Self::Settings,
            _assets: &Self::Assets,
            _storage: &Self::Storage,
        ) -> MutationResult {            
            let mut changed = false;
            
            
            let mut new_grid = self.grid.clone();

            //find all lines            
            let mut any_removed = false;

            for i_is_x in [false, true]
            {
                for i in 0..8u8{
                    let mut previous_gem_type: Option<GemType> = None;
                    let mut consecutive = 0;
                    for j in 0..8u8{
                        let tile = if i_is_x {Match3Tile::try_new(i, j).unwrap()} else {Match3Tile::try_new(j, i).unwrap()};
                        let gem_type = self.grid[tile].map(|x|x.gem_type);

                        if let Some(gem_type) = gem_type{
                            if Some(gem_type) == previous_gem_type{
                                consecutive += 1;
                                if consecutive >= 3{
                                    changed = true;
                                    self.score += 1;
                                    new_grid[tile] = None;
                                    if consecutive == 3{
                                        //also remove the two previous tiles
                                        any_removed = true;
                                        for sub in [2,1]{
                                            let tile_to_remove = if i_is_x {Match3Tile::try_new(i, j -sub).unwrap()} else {Match3Tile::try_new(j - sub, i).unwrap()};
                                            new_grid[tile_to_remove] = None;
                                        }
                                    }
                                }
                            }else{
                                previous_gem_type = Some(gem_type);
                                consecutive = 1;
                            }
                            }else{
                                previous_gem_type = None;
                                consecutive = 0;
                            }
                    }
                }
            }
            if any_removed{
                self.moves_left += 1;
            }

            //move down all tiles with a space below them
            for tile in Match3Tile::iter_by_row().rev(){
                let gem = new_grid[tile];
                if gem.is_none(){
                    if let Some(above_tile)  = tile.const_add(&Vector::NORTH){
                        //log::info!("Moving gem down to {tile}");
                        //move the tile from above down
                        new_grid.swap(tile, above_tile);
                        if new_grid[tile].is_some(){
                            changed = true;
                        }
                    }else{
                        //log::info!("Creating new gem at {tile}");
                        //this tile is on the top row so create a new tile
                        let new_gem_type = GemType::new_random(&mut self.rng_state);
                        let new_index = self.next_index;
                        self.next_index += 1; //todo be careful of this
                        changed = true;
                        new_grid[tile] = Some(self.next_tiles[tile.x() as usize]);
                        self.next_tiles[tile.x() as usize] = Gem { index: new_index, gem_type: new_gem_type };

                    }
                }
            }


            self.grid = new_grid;


            //log::info!("Match 3 Transition Changed {changed}");
            MutationResult{
                changed, transition_callback_in: if changed {Some(Duration::from_secs(1))} else{None}
            }        
    }

    fn apply_command(
        &mut self,
        action: Self::Command,
        settings: &Self::Settings,
        assets: &Self::Assets,
        storage: &Self::Storage,
    ) -> MutationResult {

        self.fast_forward_transitions(settings, assets, storage);
        
        let Match3Command::TileClicked(tile) = action;

        //log::info!("Tile Clicked {tile}");
        
        match self.selected_tile{
            Some(former_selected_tile) => {
                if former_selected_tile == tile{
                    //Unselect the tile
                    self.selected_tile = None;
                }else if former_selected_tile.is_contiguous_with(&tile){
                    //swap the tiles

                    if let Some(new_moves_left)  = self.moves_left.checked_sub(1){
                        self.grid.swap(tile, former_selected_tile);
                        self.moves_left = new_moves_left;
                        self.selected_tile = None;
                    }                   

                    //TODO match 3 logic

                }else{
                    self.selected_tile = Some(tile);
                }
            },
            None => {
                self.selected_tile = Some(tile);
            },
        }
        let transition_callback_in = if true {Some(Duration::from_secs(1))} else {None};

        MutationResult { changed: true, transition_callback_in }
    }
}


impl LeptosGameState for Match3Game{

}

impl LeptosGameEntity for Match3Entity{
    fn render(&self, meta: &StoredEntityMeta<Self>, sender: CommandSender<Self::GameState>) -> leptos::prelude::AnyView {

        const FONT_SIZE: f32 = 72.0;

        struct TileGemData{
            gem_type: GemType,
            row: f32,
            column: f32,
            selected: bool,
            visible: bool,
            click_command: Option<Match3Command> 

        }

        let tile_gem_data = 

        match *self{
            Match3Entity::ScoreText(score) => {
                let text = format!("Score: {score}");
                return view! {
                    <text x=0 y=20 FONT_SIZE=FONT_SIZE>
                        {text}
                    </text>
                }.into_any();
            },
            Match3Entity::MovesLeftText(moves_left) => {
                let text = format!("Moves Left: {moves_left}");
                return view! {
                    <text x=0 y=80 FONT_SIZE=FONT_SIZE>
                        {text}
                    </text>
                }.into_any();
            },
            Match3Entity::HiddenTile { index: _ , gem_type, column_index }
            =>{
                    TileGemData{
                        gem_type,
                        row: -1.0,
                        column: column_index as f32,
                        selected: false,
                        visible: false,
                        click_command: None
                    }
            }

            Match3Entity::Tile { index: _, tile, gem_type, selected }           
             => {
                TileGemData{
                    gem_type,
                    row: tile.y() as f32,
                    column: tile.x() as f32,
                    selected,
                    visible: true,
                    click_command: Some(Match3Command::TileClicked(tile))
                }              
            },
        };

        let TileGemData {  gem_type, row, column, selected, visible, click_command } = tile_gem_data;

                const SCALE: f32 = 80.0;
                const TOP_OFFSET: f32 = 120.0;               
                const SQUARE_SIZE: f32 = SCALE * 0.9;               
                const RECT_OFFSET: f32 = SQUARE_SIZE * 0.5;
                let x = (SCALE * (column + 0.5)) - RECT_OFFSET;
                let y = (SCALE * (row + 0.5)) + TOP_OFFSET - RECT_OFFSET;

                
                let fill = gem_type.fill();

                let stroke = if selected { "#222222FF" } else { "#00000000" }; 

                let scale = if !visible || meta.is_dead() {0.0} else {1.0};


                view! {                   

                    <rect x=0 y=0 
                    width=SQUARE_SIZE 
                    height =SQUARE_SIZE 
                    rx="2%" ry="2%"  
                    transform-origin="center"
                    style= format!("transform: translate({x}px, {y}px) scale({scale}) ; fill: {fill}; stroke: {stroke}; transform-box: fill-box; stroke-width: 10px;  transition: fill 1s, stroke 1s, transform 1s;")
                    on:click={move|_|{if let Some(command) = click_command {sender.send_command(command)} } } >
                    </rect>
                    
                    
                }.into_any()

        
    }
}


impl Match3Entity{
    pub fn svg_text(&self, meta: &StoredEntityMeta<Self>,)-> String{
        const FONT_SIZE: f32 = 72.0;

        struct TileGemData{
            gem_type: GemType,
            row: f32,
            column: f32,
            selected: bool,
            visible: bool,

        }

        let tile_gem_data = 

        match *self{
            Match3Entity::ScoreText(score) => {
                let text = format!("Score: {score}");
                return format! {
                    r##"
                    <text x=0 y=20 font_size={FONT_SIZE}>
                        {text}
                    </text>"##
                };
            },
            Match3Entity::MovesLeftText(moves_left) => {
                let text = format!("Moves Left: {moves_left}");
                return format! {
                    r##"
                    <text x=0 y=80 font_size={FONT_SIZE}>
                        {text}
                    </text>"##
                };
            },
            Match3Entity::HiddenTile { index: _ , gem_type, column_index }
            =>{
                    TileGemData{
                        gem_type,
                        row: -1.0,
                        column: column_index as f32,
                        selected: false,
                        visible: false,
                    }
            }

            Match3Entity::Tile { index: _, tile, gem_type, selected }           
             => {
                TileGemData{
                    gem_type,
                    row: tile.y() as f32,
                    column: tile.x() as f32,
                    selected,
                    visible: true,
                }              
            },
        };

        let TileGemData {  gem_type, row, column, selected, visible,  } = tile_gem_data;

                const SCALE: f32 = 80.0;
                const TOP_OFFSET: f32 = 120.0;               
                const SQUARE_SIZE: f32 = SCALE * 0.9;               
                const RECT_OFFSET: f32 = SQUARE_SIZE * 0.5;
                let x = (SCALE * (column + 0.5)) - RECT_OFFSET;
                let y = (SCALE * (row + 0.5)) + TOP_OFFSET - RECT_OFFSET;

                
                let fill = gem_type.fill();

                let stroke = if selected { "#222222FF" } else { "#00000000" }; 

                let scale = if !visible || meta.is_dead() {0.0} else {1.0};


                format! {                   
                    r##"
                    <rect x=0 y=0 
                    width={SQUARE_SIZE} 
                    height ={SQUARE_SIZE} 
                    rx="2%" ry="2%"  
                    transform-origin="center"
                    style= "transform: translate({x}px, {y}px) scale({scale}) ; fill: {fill}; stroke: {stroke}; transform-box: fill-box; stroke-width: 10px;  transition: fill 1s, stroke 1s, transform 1s;">
                    </rect>"##
                    
                    
                }
    }
}

#[cfg(test)]
mod tests{
    use std::time::{Duration};

    
    use leptos::prelude::{ ReadUntracked};
    use state_machine_games::{entity_store::EntityStore, leptos::leptos_entity_store::LeptosEntityStore};
    use itertools::Itertools;
    use super::*;


    #[test]
    pub fn test_game(){
        let settings = ();
        let assets = ();
        let storage = ();

        let mut game = Match3Game::new_random(123);

        let commands = 
        //[(3,6)]
        
        
         [(3,6),(2,6),(2,3),(1,3),(3,4),(3,5),(5,4),(4,4),(7,7),(0,2),(1,2),(0,2),(1,2),(1,3),(2,3),(3,2),(2,2),]
        .map(|(x,y)|Match3Tile::try_new(x, y).unwrap())
        .map(|x|Match3Command::TileClicked(x));

        let mut entity_store: LeptosEntityStore<Match3Entity> = EntityStore::new(game.get_entities(&settings, &assets, &storage));

        let mut now = web_time::Instant::now();
        for command in commands{

            let _ = game.apply_command(command, &settings, &assets, &storage);
            now += Duration::from_hours(1);
            let _ = entity_store.update(game.get_entities(&settings, &assets, &storage), now);
            game.fast_forward_transitions(&settings, &assets, &storage);
            now += Duration::from_hours(1);
            let _ = entity_store.update(game.get_entities(&settings, &assets, &storage), now);
        }
        now += Duration::from_hours(1);
        let _ = entity_store.update(game.get_entities(&settings, &assets, &storage), now);

        let svg_inner = entity_store.entities.iter()
        
        //.filter(|x|x.signal.read_untracked().0.is_score_text())
        .map(|x|{
            let read_guard = x.signal.read_untracked();
            let svg = read_guard.0.svg_text(&read_guard.1);

            svg
        }) .join("\n");

        let svg = format!(r#"
        <svg viewbox="0 0 800 800" style="width: 800px; height: 800px; background:white;">
            {svg_inner}
        </svg>
        "#);

        insta::assert_snapshot!(svg);


    }
}