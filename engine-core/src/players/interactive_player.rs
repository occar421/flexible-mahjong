use self::super::Player;
use crate::game::{TurnChoice, MeldChoice};
use crate::tile::Tile;
use std::marker::PhantomData;

pub struct InteractivePlayer<TTile: Tile> {
    _type: PhantomData<TTile>
}

impl<TTile: Tile> InteractivePlayer<TTile> {
    pub fn new() -> InteractivePlayer<TTile> {
        InteractivePlayer {
            _type: PhantomData
        }
    }
}

impl<TTile: Tile> Player for InteractivePlayer<TTile> {
    type Tile = TTile;

    fn set_dealt_tiles(&self, tiles: &Vec<Self::Tile>) {
        unimplemented!()
    }

    fn draw(&self, tile: &Self::Tile, options: &Vec<TurnChoice<Self::Tile>>) -> TurnChoice<Self::Tile> {
        unimplemented!()
    }

    fn consider_melding(&self, discarded_tile: &Self::Tile, options: &Vec<MeldChoice<Self::Tile>>) -> MeldChoice<Self::Tile> {
        unimplemented!()
    }
}