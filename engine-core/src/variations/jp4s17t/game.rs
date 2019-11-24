mod game_moderator;
mod player_broker;

use std::iter::FromIterator;

use crate::game::Meld;
use crate::collections::MultiBTreeSet;

use super::tile::Tile;

const N_TILES: u8 = 9 * 4 * 4 + 4 * 4 + 4 * 4;

#[derive(Eq, PartialEq, Debug)]
pub(crate) enum WinningPoint {
    // 飜
    Fan(u8),
    // 役満 (could be double Yakuman or triple Yakuman etc.)
    Yakuman(u8),
}

pub(crate) struct PlayerHandJp4s17t {
    pub(crate) closed_tiles: MultiBTreeSet<Tile>,
    pub(crate) melds: Vec<Meld<Tile>>,
    pub(crate) discard_pile: Vec<(Tile, bool)>,
}

impl PlayerHandJp4s17t {
    pub(crate) fn new<I: IntoIterator<Item=Tile>>(tiles: I) -> PlayerHandJp4s17t {
        PlayerHandJp4s17t {
            closed_tiles: MultiBTreeSet::from_iter(tiles),
            melds: vec![],
            discard_pile: vec![],
        }
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.melds.iter().all(|m| match m {
            Meld::Kong(_, None) => true, // closed kong, 暗槓
            _ => false
        })
    }
}

#[cfg(test)]
impl PlayerHandJp4s17t {
    pub(crate) fn create<I: IntoIterator<Item=Tile>>(tiles: I, melds: Vec<Meld<Tile>>, discard_pile: Vec<(Tile, bool)>) -> PlayerHandJp4s17t {
        PlayerHandJp4s17t {
            closed_tiles: MultiBTreeSet::from_iter(tiles),
            melds,
            discard_pile,
        }
    }
}
