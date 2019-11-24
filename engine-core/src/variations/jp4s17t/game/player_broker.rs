use std::iter::once;

use crate::game::{Meld, MeldChoice, TurnChoice};

use super::super::tile::Tile;
use crate::hands::{Hand, HandTestResult};
use super::{PlayerHandJp4s17t, WinningPoint};
use std::collections::HashMap;

pub(crate) struct PlayerBroker(pub(crate) PlayerHandJp4s17t);

impl crate::game::PlayerBroker for PlayerBroker {
    type Point = WinningPoint;
    type PlayerHand = PlayerHandJp4s17t;
    type Tile = Tile;
    // trait Hand = Hand<Point=Self::Point, PlayerHand=Self::PlayerHand, Tile=Self::Tile>;

    fn get_options_on_drawing(&self, possible_hands: &Vec<&dyn Hand<Point=Self::Point, PlayerHand=Self::PlayerHand, Tile=Self::Tile>>, drawn_tile: &Self::Tile) -> Vec<TurnChoice<Self::Tile>> {
        let mut tiles = self.0.closed_tiles.clone();
        tiles.insert(*drawn_tile);
        let buckets = tiles.get_by_buckets();
        let mut options = vec![];
        options.extend(buckets.clone().filter_map(|(&t, &n)| if n == 4 { Some(TurnChoice::MakeConcealedKong(t)) } else { None }));
        options.extend(self.0.melds.iter().filter_map(|m| match m {
            Meld::Pong([t, _, _], _) if tiles.contains(t) => Some(TurnChoice::MakeKongFromPong(*t)),
            _ => None
        }));
        if possible_hands.iter().any(|h| match h.test_completion_on_drawing(&self.0, drawn_tile) {
            HandTestResult::Winning(_) => true,
            _ => false
        }) {
            // FIXME: check sacred discard フリテン
            // FIXME: check only closed hand 面前役
            options.extend(once(TurnChoice::Complete));
        };
        options.extend(buckets.flat_map(|(&t, &n)| (0..n).map(move |n| TurnChoice::Discard(t, n))));
        options // FIXME: declare-ready option
    }

    fn get_options_when_discarded(&self, can_kong: bool, can_meld: bool, possible_hands: &Vec<&dyn Hand<Point=Self::Point, PlayerHand=Self::PlayerHand, Tile=Self::Tile>>, discarded_tile: &Self::Tile) -> Vec<MeldChoice<Self::Tile>> {
        let closed_tiles = self.0.closed_tiles.clone();
        let mut buckets = closed_tiles.get_by_buckets();
        let map: HashMap<_, _> = buckets.map(|(&t, &n)| (t, n)).collect();
        let mut options = vec![MeldChoice::DoNothing];

        if can_kong && can_meld {
            if let Some(&n) = map.get(discarded_tile) {
                if n == 3 {
                    options.extend(once(MeldChoice::MakeExposedKong));
                }
            }
        }

        if can_meld {
            if let Some(&n) = map.get(discarded_tile) {
                if n == 2 || n == 3 {
                    options.extend(once(MeldChoice::MakePong));
                }
            }

            // FIXME: Chow
        }

        let mut tiles = self.0.closed_tiles.clone();
        tiles.insert(*discarded_tile);

        if possible_hands.iter().any(|h| match h.test_completion_when_discarded(&self.0, discarded_tile) {
            HandTestResult::Winning(_) => true,
            _ => false
        }) {
            // FIXME: check sacred discard フリテン
            options.extend(once(MeldChoice::Complete));
        };

        options
    }

    fn discard(&mut self, drawn_tile: &Self::Tile, tile: &Self::Tile, index: usize) {
        self.0.closed_tiles.insert(*drawn_tile);
        if self.0.closed_tiles.remove(tile) {
            return;
        }
        panic!("Can't discard because they don't have it");
    }

    fn add_tile_to_discard_pile(&mut self, tile: &Self::Tile, is_used_in_meld: bool) {
        self.0.discard_pile.push((*tile, is_used_in_meld));
    }

    // is 聴牌
    fn is_ready(&self) -> bool {
        false // FIXME
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{TurnChoice, PlayerBroker as _, MeldChoice};

    use super::PlayerBroker;
    use super::super::PlayerHandJp4s17t;
    use super::super::super::tile::Tile::{Number, Wind, Symbol};
    use super::super::super::tile::Suite::{Green, Red, White, Black};
    use super::super::super::hands::{FanHand, AllInTriplets};
    use std::collections::HashSet;
    use std::collections::hash_map::RandomState;
    use std::iter::{FromIterator, once, repeat};

    #[test]
    fn test_get_options_on_drawing() {
        let player_hand = PlayerHandJp4s17t::create(
            (1..=4).flat_map(|n| repeat(Number(Green, n)).take(4)),
            vec![], vec![]);
        let broker = PlayerBroker(player_hand);
        let options = broker.get_options_on_drawing(&vec![&FanHand::<AllInTriplets>::new(1, 2)], &Number(Green, 6));
        let options: HashSet<_, RandomState> = HashSet::from_iter(options.iter().copied());

        let expected_options = HashSet::from_iter(
            once(TurnChoice::Discard(Number(Green, 6), 0))
                .chain((1..=4).flat_map(|n| vec![
                    TurnChoice::MakeConcealedKong(Number(Green, n)),
                    TurnChoice::Discard(Number(Green, n), 0),
                    TurnChoice::Discard(Number(Green, n), 1),
                    TurnChoice::Discard(Number(Green, n), 2),
                    TurnChoice::Discard(Number(Green, n), 3),
                ])));
        assert_eq!(options, expected_options);
    }

    #[test]
    fn test_get_options_when_discarded() {
        let player_hand = PlayerHandJp4s17t::create(
            (1..=5).flat_map(|n| repeat(Number(Green, n)).take(3)).chain(once(Number(Green, 6))),
            vec![], vec![]);
        let broker = PlayerBroker(player_hand);
        let options = broker.get_options_when_discarded(true, true, &vec![&FanHand::<AllInTriplets>::new(1, 2)], &Number(Green, 1));
        let options: HashSet<_, RandomState> = HashSet::from_iter(options.iter().copied());

        let expected_options = HashSet::from_iter(
            vec![MeldChoice::DoNothing, MeldChoice::MakePong, MeldChoice::MakeExposedKong].iter().cloned()
        );
        assert_eq!(options, expected_options);
    }
}
