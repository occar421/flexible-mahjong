use crate::hands::{Hand, HandTestResult};
use super::super::tile::{Tile, Suite};
use itertools::Itertools;
use super::super::game::{PlayerHandJp4s17t, WinningPoint};
use std::collections::{HashMap, BTreeSet};
use std::iter::FromIterator;
use super::YakumanHand;

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
                if tiles == correct_tiles && correct_tiles.contains(new_tile) {
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

impl Hand<PlayerHandJp4s17t> for YakumanHand<SixteenOrphans> {
    type Point = WinningPoint;
    type Tile = Tile;

    fn test_completion_on_drawing(&self, player_hand: &PlayerHandJp4s17t, drawn_tile: &Self::Tile) -> HandTestResult<Self::Point> {
        self.test(player_hand, drawn_tile)
    }

    fn test_completion_when_discarded(&self, player_hand: &PlayerHandJp4s17t, discarded_tile: &Self::Tile) -> HandTestResult<Self::Point> {
        self.test(player_hand, discarded_tile)
    }
}

#[cfg(test)]
mod tests {
    use super::SixteenOrphans;
    use crate::hands::{Hand, HandTestResult};
    use super::super::super::game::{PlayerHandJp4s17t, WinningPoint};
    use super::super::super::tile::Tile::{Number, Wind, Symbol};
    use super::super::super::tile::Suite::{Green, Red, White, Black};
    use super::super::YakumanHand;

    #[test]
    fn when_drawn_wins_1_wait() {
        let matcher = YakumanHand::<SixteenOrphans>::new(1, 2);
        let hand = PlayerHandJp4s17t::create(
            vec![Number(Green, 1)].iter().chain(
                YakumanHand::<SixteenOrphans>::TERMINALS_AND_HONERS.iter()
                    .filter(|&t| t != &Number(Green, 9))).copied(),
            vec![], vec![]);
        let result = matcher.test_completion_on_drawing(&hand, &Number(Green, 9));
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
        let result = matcher.test_completion_on_drawing(&hand, &Number(Green, 1));
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
        let result = matcher.test_completion_on_drawing(&hand, &Number(Green, 5));
        assert_eq!(result, HandTestResult::Nothing);
    }
}