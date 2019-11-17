use crate::hands::{Hand as HandBase, HandTestResult};
use super::tile::Tile;
use itertools::Itertools;
use super::game::{PlayerHandJp4s17t, WinningPoint};
use std::collections::HashMap;

// trait Hand = HandBase<PlayerHandJp4s17t, Tile=Tile>;

pub(crate) struct EightPairsAndHalf {
    closed_han: u8,
    open_han: u8,
}

impl EightPairsAndHalf {
    pub(crate) fn new(closed_han: u8, open_han: u8) -> EightPairsAndHalf {
        EightPairsAndHalf {
            closed_han,
            open_han,
        }
    }

    fn test(&self, player_hand: &PlayerHandJp4s17t, new_tile: &Tile) -> HandTestResult<WinningPoint> {
        let mut tiles = player_hand.closed_tiles.clone();
        tiles.insert(*new_tile);
        let groups = tiles.get_by_buckets();
        let groups = groups.group_by(|&(_, &n)| n);
        let map: HashMap<_, Vec<_>> = groups.into_iter().map(|(n, gv)| (n, gv.collect())).collect();
        match (map.get(&2), map.get(&3)) {
            (Some(n2), Some(n3)) if n2.len() == 7 && n3.len() == 1 =>
                HandTestResult::Winning(WinningPoint::Fan(if player_hand.is_closed() { self.closed_han } else { self.open_han })),
            _ => HandTestResult::Nothing
        }
    }
}

impl HandBase<PlayerHandJp4s17t> for EightPairsAndHalf {
    type Point = WinningPoint;
    type Tile = Tile;

    fn test_with_drawn_tile(&self, player_hand: &PlayerHandJp4s17t, drawn_tile: &Self::Tile) -> HandTestResult<Self::Point> {
        self.test(player_hand, drawn_tile)
    }

    fn test_with_discarded_tile(&self, player_hand: &PlayerHandJp4s17t, discarded_tile: &Self::Tile) -> HandTestResult<Self::Point> {
        self.test(player_hand, discarded_tile)
    }
}

#[cfg(test)]
mod tests {
    use super::super::game::PlayerHandJp4s17t;
    use super::super::structure::MultiBTreeSet;
    use super::super::tile::Tile;
    use crate::game::Meld;
    use std::iter::FromIterator;

    impl PlayerHandJp4s17t {
        fn create<I: IntoIterator<Item=Tile>>(tiles: I, melds: Vec<Meld<Tile>>, discard_pile: Vec<(Tile, bool)>) -> PlayerHandJp4s17t {
            PlayerHandJp4s17t {
                closed_tiles: MultiBTreeSet::from_iter(tiles),
                melds,
                discard_pile,
            }
        }
    }

    mod eight_pairs_and_half {
        use super::super::EightPairsAndHalf;
        use crate::hands::{Hand, HandTestResult};
        use super::super::super::game::{PlayerHandJp4s17t, WinningPoint};
        use super::super::super::tile::Tile::{Number, Wind, Symbol};
        use super::super::super::tile::Suite::{Green, Red, White, Black};

        #[test]
        fn when_drawn_wins() {
            let matcher = EightPairsAndHalf { closed_han: 2, open_han: 1 };
            let hand = PlayerHandJp4s17t::create(
                (1..=8).map(|i| Number(Green, i))
                    .map(|t| vec![t, t])
                    .flatten(),
                vec![], vec![]);
            let result = matcher.test_with_drawn_tile(&hand, &Number(Green, 1));
            assert_eq!(result, HandTestResult::Winning(WinningPoint::Fan(2)));
        }

        #[test]
        fn when_drawn_nothing_happens() {
            let matcher = EightPairsAndHalf { closed_han: 2, open_han: 1 };
            let hand = PlayerHandJp4s17t::create(
                (1..=8).map(|i| Number(Red, i))
                    .map(|t| vec![t, t])
                    .flatten(),
                vec![], vec![]);
            let result = matcher.test_with_drawn_tile(&hand, &Number(Red, 9));
            assert_eq!(result, HandTestResult::Nothing);
        }
    }
}
