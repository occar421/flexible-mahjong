mod eight_pairs_and_half;
mod all_in_triplets;
mod sixteen_orphans;

use std::marker::PhantomData;

#[cfg(test)]
use {
    super::tile::Tile,
    super::game::PlayerHandJp4s17t,
    crate::game::Meld,
    std::iter::FromIterator,
    crate::collections::MultiBTreeSet,
};

pub(crate) struct FanHand<T> {
    closed_han: u8,
    open_han: u8,
    phantom: PhantomData<T>,
}

pub(crate) struct YakumanHand<T> {
    config: T
}

#[cfg(test)]
impl PlayerHandJp4s17t {
    fn create<I: IntoIterator<Item=Tile>>(tiles: I, melds: Vec<Meld<Tile>>, discard_pile: Vec<(Tile, bool)>) -> PlayerHandJp4s17t {
        PlayerHandJp4s17t {
            closed_tiles: MultiBTreeSet::from_iter(tiles),
            melds,
            discard_pile,
        }
    }
}
