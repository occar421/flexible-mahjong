use std::iter::FromIterator;

use crate::game::{Meld, HandPart};
use crate::collections::MultiBTreeSet;

use super::super::tile::Tile;
use itertools::Itertools;

pub(crate) struct PlayerHand {
    pub(crate) closed_tiles: MultiBTreeSet<Tile>,
    pub(crate) melds: Vec<Meld<Tile>>,
    pub(crate) discard_pile: Vec<(Tile, bool)>,
}

impl PlayerHand {
    pub(crate) fn new<I: IntoIterator<Item=Tile>>(tiles: I) -> PlayerHand {
        PlayerHand {
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
impl PlayerHand {
    pub(crate) fn create<I: IntoIterator<Item=Tile>>(tiles: I, melds: Vec<Meld<Tile>>, discard_pile: Vec<(Tile, bool)>) -> PlayerHand {
        PlayerHand {
            closed_tiles: MultiBTreeSet::from_iter(tiles),
            melds,
            discard_pile,
        }
    }
}
