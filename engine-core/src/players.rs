mod interactive_player;

pub use interactive_player::InteractivePlayer;
use crate::game::{TurnChoice, MeldChoice};
use crate::tile::Tile;

pub(crate) trait Player {
    type Tile: Tile;
    fn set_dealt_tiles(&self, tiles: &Vec<Self::Tile>);
    fn draw(&self, drawn_tile: &Self::Tile, options: &Vec<TurnChoice<Self::Tile>>) -> TurnChoice<Self::Tile>;
    fn consider_melding(&self, discarded_tile: &Self::Tile, options: &Vec<MeldChoice<Self::Tile>>) -> MeldChoice<Self::Tile>;
}

// players/cpu_discards_drawn_tile.rs ツモ切り CPU
// players/cpu_weak.rs