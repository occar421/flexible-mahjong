use crate::hands::{Hand, HandTestResult};
use super::super::tile::Tile;
use itertools::Itertools;
use super::super::game::{PlayerHandJp4s17t, WinningPoint};
use std::collections::HashMap;
use std::marker::PhantomData;
use crate::game::Meld;
use super::FanHand;

/// 対々和
pub(crate) struct AllInTriplets;

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

impl Hand<PlayerHandJp4s17t> for FanHand<AllInTriplets> {
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
    use super::AllInTriplets;
    use crate::hands::{Hand, HandTestResult};
    use super::super::super::game::{PlayerHandJp4s17t, WinningPoint};
    use super::super::super::tile::Tile::{Number, Wind, Symbol};
    use super::super::super::tile::Suite::{Green, Red, White, Black};
    use super::super::FanHand;
    use crate::game::{Meld, Side};
    use std::iter::repeat;

    #[test]
    /// 裸単騎待ち
    fn when_drawn_wins_only_one_tile() {
        let matcher = FanHand::<AllInTriplets>::new(2, 2);
        let hand = PlayerHandJp4s17t::create(
            vec![Number(Green, 6)],
            (1..=5).map(|i| {
                let t = Number(Green, i);
                Meld::Pong([t, t, t], Side::Left)
            }).collect(),
            vec![],
        );
        let result = matcher.test_completion_on_drawing(&hand, &Number(Green, 6));
        assert_eq!(result, HandTestResult::Winning(WinningPoint::Fan(2)));
    }

    #[test]
    fn when_drawn_wins_with_double_wait() {
        let matcher = FanHand::<AllInTriplets>::new(2, 2);
        let hand = PlayerHandJp4s17t::create(
            vec![Number(Green, 5), Number(Green, 5), Number(Green, 6), Number(Green, 6)],
            (1..=4).map(|i| {
                let t = Number(Green, i);
                Meld::Pong([t, t, t], Side::Left)
            }).collect(),
            vec![],
        );
        let result = matcher.test_completion_on_drawing(&hand, &Number(Green, 6));
        assert_eq!(result, HandTestResult::Winning(WinningPoint::Fan(2)));
    }

    #[test]
    fn when_drawn_nothing_happens() {
        let matcher = FanHand::<AllInTriplets>::new(2, 2);
        let hand = PlayerHandJp4s17t::create(
            (1..=8).flat_map(|i| repeat(Number(Red, i)).take(2)),
            vec![], vec![]);
        let result = matcher.test_completion_on_drawing(&hand, &Number(Red, 9));
        assert_eq!(result, HandTestResult::Nothing);
    }
}