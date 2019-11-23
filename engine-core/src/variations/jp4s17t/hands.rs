use crate::hands::{Hand as HandBase, HandTestResult};
use super::tile::{Tile, Suite};
use itertools::Itertools;
use super::game::{PlayerHandJp4s17t, WinningPoint};
use std::collections::{HashMap, BTreeSet};
use std::marker::PhantomData;
use crate::game::Meld;
use std::iter::FromIterator;

// trait NormalHand = HandBase<PlayerHandJp4s17t, Tile=Tile>;

pub(crate) struct FanHand<T> {
    closed_han: u8,
    open_han: u8,
    phantom: PhantomData<T>,
}

pub(crate) struct YakumanHand<T> {
    config: T
}

/// 八対子半
struct EightPairsAndHalf;

impl FanHand<EightPairsAndHalf> {
    pub(crate) fn new(closed_han: u8, open_han: u8) -> FanHand<EightPairsAndHalf> {
        FanHand {
            closed_han,
            open_han,
            phantom: PhantomData,
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

impl HandBase<PlayerHandJp4s17t> for FanHand<EightPairsAndHalf> {
    type Point = WinningPoint;
    type Tile = Tile;

    fn test_with_drawn_tile(&self, player_hand: &PlayerHandJp4s17t, drawn_tile: &Self::Tile) -> HandTestResult<Self::Point> {
        self.test(player_hand, drawn_tile)
    }

    fn test_with_discarded_tile(&self, player_hand: &PlayerHandJp4s17t, discarded_tile: &Self::Tile) -> HandTestResult<Self::Point> {
        self.test(player_hand, discarded_tile)
    }
}

/// 対々和
struct AllInTriplets;

impl FanHand<AllInTriplets> {
    pub(crate) fn new(closed_han: u8, open_han: u8) -> FanHand<AllInTriplets> {
        FanHand {
            closed_han,
            open_han,
            phantom: PhantomData,
        }
    }

    fn test(&self, player_hand: &PlayerHandJp4s17t, new_tile: &Tile) -> HandTestResult<WinningPoint> {
        if !player_hand.melds.iter().all(|m| match m {
            Meld::Kong(_, _) | Meld::Pong(_, _) => true,
            _ => false
        }) {
            return HandTestResult::Nothing;
        }

        let winning_result = HandTestResult::Winning(WinningPoint::Fan(if player_hand.is_closed() { self.closed_han } else { self.open_han }));

        let mut tiles = player_hand.closed_tiles.clone();
        tiles.insert(*new_tile);
        let n_closed_tiles = tiles.len();
        let groups = tiles.get_by_buckets();
        let groups = groups.group_by(|&(_, &n)| n);
        let map: HashMap<_, Vec<_>> = groups.into_iter().map(|(n, gv)| (n, gv.collect())).collect();

        if map.keys().len() == 1 && map.contains_key(&2) {
            return winning_result;
        }
        match (map.get(&2), map.get(&3)) {
            (Some(n2), Some(n3)) if n2.len() == 1 && n3.len() == (n_closed_tiles - 2) / 3 =>
                winning_result,
            _ => HandTestResult::Nothing
        }
    }
}

impl HandBase<PlayerHandJp4s17t> for FanHand<AllInTriplets> {
    type Point = WinningPoint;
    type Tile = Tile;

    fn test_with_drawn_tile(&self, player_hand: &PlayerHandJp4s17t, drawn_tile: &Self::Tile) -> HandTestResult<Self::Point> {
        self.test(player_hand, drawn_tile)
    }

    fn test_with_discarded_tile(&self, player_hand: &PlayerHandJp4s17t, discarded_tile: &Self::Tile) -> HandTestResult<Self::Point> {
        self.test(player_hand, discarded_tile)
    }
}

/// 国士無双
struct SixteenOrphans { standard_value: u8, sixteen_wait_value: u8 }

impl YakumanHand<SixteenOrphans> {
    pub(crate) fn new(standard_value: u8, sixteen_wait_value: u8) -> YakumanHand<SixteenOrphans> {
        YakumanHand {
            config: SixteenOrphans { standard_value, sixteen_wait_value }
        }
    }

    const TERMINALS_AND_HONERS: [Tile; 16] = [
        Tile::Number(Suite::Green, 1), Tile::Number(Suite::Green, 9),
        Tile::Number(Suite::Red, 1), Tile::Number(Suite::Red, 9),
        Tile::Number(Suite::White, 1), Tile::Number(Suite::White, 9),
        Tile::Number(Suite::Black, 1), Tile::Number(Suite::Black, 9),
        Tile::Wind(Suite::Green), Tile::Wind(Suite::Red), Tile::Wind(Suite::White), Tile::Wind(Suite::Black),
        Tile::Symbol(Suite::Green), Tile::Symbol(Suite::Red), Tile::Symbol(Suite::White), Tile::Symbol(Suite::Black)
    ];

    fn test(&self, player_hand: &PlayerHandJp4s17t, new_tile: &Tile) -> HandTestResult<WinningPoint> {
        if player_hand.melds.len() != 0 {
            return HandTestResult::Nothing;
        }

        let correct_tiles = BTreeSet::from_iter(&YakumanHand::<SixteenOrphans>::TERMINALS_AND_HONERS);

        let groups = player_hand.closed_tiles.get_by_buckets();
        let groups = groups.group_by(|&(_, &n)| n);
        let map: HashMap<_, Vec<_>> = groups.into_iter().map(|(n, gv)| (n, gv.collect())).collect();
        if map.keys().len() == 1 {
            if let Some(tiles) = map.get(&1) {
                let tiles = BTreeSet::from_iter(tiles.iter().map(|(t, _)| *t));
                let is_identical = tiles.is_superset(&correct_tiles) && tiles.is_subset(&correct_tiles);
                if is_identical && correct_tiles.contains(new_tile) {
                    return HandTestResult::Winning(WinningPoint::Yakuman(self.config.sixteen_wait_value));
                }
            }
            return HandTestResult::Nothing;
        }

        let mut tiles = player_hand.closed_tiles.clone();
        tiles.insert(*new_tile);
        let groups = tiles.get_by_buckets();
        let groups = groups.group_by(|&(_, &n)| n);
        let map: HashMap<_, Vec<_>> = groups.into_iter().map(|(n, gv)| (n, gv.collect())).collect();
        match (map.get(&1), map.get(&2)) {
            (Some(n1), Some(n2)) if n1.len() == 15 && n2.len() == 1 => {
                let n1_tiles = BTreeSet::from_iter(n1.iter().map(|(t, _)| *t));
                let diff: Vec<_> = n1_tiles.symmetric_difference(&correct_tiles).collect();
                let tile_for_pair = n2[0].0;
                if diff.len() == 1 && diff[0] == &tile_for_pair && correct_tiles.contains(&tile_for_pair) {
                    HandTestResult::Winning(WinningPoint::Yakuman(self.config.standard_value))
                } else {
                    HandTestResult::Nothing
                }
            }
            _ => HandTestResult::Nothing
        }
    }
}

impl HandBase<PlayerHandJp4s17t> for YakumanHand<SixteenOrphans> {
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
    use super::super::tile::Tile;
    use crate::game::Meld;
    use crate::collections::MultiBTreeSet;
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
        use crate::hands::{Hand as HandBase, HandTestResult};
        use super::super::super::game::{PlayerHandJp4s17t, WinningPoint};
        use super::super::super::tile::Tile::{Number, Wind, Symbol};
        use super::super::super::tile::Suite::{Green, Red, White, Black};
        use super::super::FanHand;

        #[test]
        fn when_drawn_wins() {
            let matcher = FanHand::<EightPairsAndHalf>::new(2, 1);
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
            let matcher = FanHand::<EightPairsAndHalf>::new(2, 1);
            let hand = PlayerHandJp4s17t::create(
                (1..=8).map(|i| Number(Red, i))
                    .map(|t| vec![t, t])
                    .flatten(),
                vec![], vec![]);
            let result = matcher.test_with_drawn_tile(&hand, &Number(Red, 9));
            assert_eq!(result, HandTestResult::Nothing);
        }
    }

    mod all_in_triplets {
        use super::super::AllInTriplets;
        use crate::hands::{Hand as HandBase, HandTestResult};
        use super::super::super::game::{PlayerHandJp4s17t, WinningPoint};
        use super::super::super::tile::Tile::{Number, Wind, Symbol};
        use super::super::super::tile::Suite::{Green, Red, White, Black};
        use super::super::FanHand;
        use crate::game::{Meld, Side};

        #[test]
        /// 裸単騎待ち
        fn when_drawn_wins_only_one_tile() {
            let matcher = FanHand::<AllInTriplets>::new(2, 2);
            let hand = PlayerHandJp4s17t::create(
                vec![Number(Green, 6)],
                (1..=5).map(|i| Number(Green, i))
                    .map(|t| Meld::Pong([t, t, t], Side::Left))
                    .collect(),
                vec![],
            );
            let result = matcher.test_with_drawn_tile(&hand, &Number(Green, 6));
            assert_eq!(result, HandTestResult::Winning(WinningPoint::Fan(2)));
        }

        #[test]
        fn when_drawn_wins_with_double_wait() {
            let matcher = FanHand::<AllInTriplets>::new(2, 2);
            let hand = PlayerHandJp4s17t::create(
                vec![Number(Green, 5), Number(Green, 5), Number(Green, 6), Number(Green, 6)],
                (1..=4).map(|i| Number(Green, i))
                    .map(|t| Meld::Pong([t, t, t], Side::Left))
                    .collect(),
                vec![],
            );
            let result = matcher.test_with_drawn_tile(&hand, &Number(Green, 6));
            assert_eq!(result, HandTestResult::Winning(WinningPoint::Fan(2)));
        }

        #[test]
        fn when_drawn_nothing_happens() {
            let matcher = FanHand::<AllInTriplets>::new(2, 2);
            let hand = PlayerHandJp4s17t::create(
                (1..=8).map(|i| Number(Red, i))
                    .map(|t| vec![t, t])
                    .flatten(),
                vec![], vec![]);
            let result = matcher.test_with_drawn_tile(&hand, &Number(Red, 9));
            assert_eq!(result, HandTestResult::Nothing);
        }
    }

    mod sixteen_orphans {
        use super::super::SixteenOrphans;
        use crate::hands::{Hand as HandBase, HandTestResult};
        use super::super::super::game::{PlayerHandJp4s17t, WinningPoint};
        use super::super::super::tile::Tile::{Number, Wind, Symbol};
        use super::super::super::tile::Suite::{Green, Red, White, Black};
        use super::super::YakumanHand;
        use crate::game::{Meld, Side};

        #[test]
        fn when_drawn_wins_1_wait() {
            let matcher = YakumanHand::<SixteenOrphans>::new(1, 2);
            let hand = PlayerHandJp4s17t::create(
                vec![Number(Green, 1)].iter().chain(
                    YakumanHand::<SixteenOrphans>::TERMINALS_AND_HONERS.iter()
                        .filter(|&t| t != &Number(Green, 9))).copied(),
                vec![], vec![]);
            let result = matcher.test_with_drawn_tile(&hand, &Number(Green, 9));
            assert_eq!(result, HandTestResult::Winning(WinningPoint::Yakuman(1)));
        }

        #[test]
        fn when_drawn_wins_16_waits() {
            let matcher = YakumanHand::<SixteenOrphans>::new(1, 2);
            let hand = PlayerHandJp4s17t::create(
                YakumanHand::<SixteenOrphans>::TERMINALS_AND_HONERS.iter().copied(),
                vec![],
                vec![],
            );
            let result = matcher.test_with_drawn_tile(&hand, &Number(Green, 1));
            assert_eq!(result, HandTestResult::Winning(WinningPoint::Yakuman(2)));
        }

        #[test]
        fn when_drawn_nothing_happens() {
            let matcher = YakumanHand::<SixteenOrphans>::new(1, 2);
            let hand = PlayerHandJp4s17t::create(
                YakumanHand::<SixteenOrphans>::TERMINALS_AND_HONERS.iter().copied(),
                vec![],
                vec![],
            );
            let result = matcher.test_with_drawn_tile(&hand, &Number(Green, 5));
            assert_eq!(result, HandTestResult::Nothing);
        }
    }
}
