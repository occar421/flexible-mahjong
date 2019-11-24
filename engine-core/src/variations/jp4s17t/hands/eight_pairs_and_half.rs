use crate::hands::{Hand, HandTestResult};
use super::super::tile::Tile;
use itertools::Itertools;
use super::super::game::{PlayerHandJp4s17t, WinningPoint};
use std::collections::HashMap;
use std::marker::PhantomData;
use super::FanHand;

/// 八対子半
pub(crate) struct EightPairsAndHalf;

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

impl Hand for FanHand<EightPairsAndHalf> {
    type Point = WinningPoint;
    type PlayerHand = PlayerHandJp4s17t;
    type Tile = Tile;

    fn test_completion_on_drawing(&self, player_hand: &Self::PlayerHand, drawn_tile: &Self::Tile) -> HandTestResult<Self::Point> {
        self.test(player_hand, drawn_tile)
    }

    fn test_completion_when_discarded(&self, player_hand: &Self::PlayerHand, discarded_tile: &Self::Tile) -> HandTestResult<Self::Point> {
        self.test(player_hand, discarded_tile)
    }
}

#[cfg(test)]
mod tests {
    use super::EightPairsAndHalf;
    use crate::hands::{Hand, HandTestResult};
    use super::super::super::game::{PlayerHandJp4s17t, WinningPoint};
    use super::super::super::tile::Tile::{Number, Wind, Symbol};
    use super::super::super::tile::Suite::{Green, Red, White, Black};
    use super::FanHand;
    use std::iter::repeat;

    #[test]
    fn when_drawn_wins() {
        let matcher = FanHand::<EightPairsAndHalf>::new(2, 1);
        let hand = PlayerHandJp4s17t::create(
            (1..=8).flat_map(|i| repeat(Number(Green, i)).take(2)),
            vec![], vec![]);
        let result = matcher.test_completion_on_drawing(&hand, &Number(Green, 1));
        assert_eq!(result, HandTestResult::Winning(WinningPoint::Fan(2)));
    }

    #[test]
    fn when_drawn_nothing_happens() {
        let matcher = FanHand::<EightPairsAndHalf>::new(2, 1);
        let hand = PlayerHandJp4s17t::create(
            (1..=8).flat_map(|i| repeat(Number(Red, i)).take(2)),
            vec![], vec![]);
        let result = matcher.test_completion_on_drawing(&hand, &Number(Red, 9));
        assert_eq!(result, HandTestResult::Nothing);
    }
}