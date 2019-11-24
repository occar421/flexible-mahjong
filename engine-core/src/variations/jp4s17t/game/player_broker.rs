use std::iter::once;

use crate::game::{Meld, MeldChoice, TurnChoice};

use super::super::tile::Tile;
use crate::hands::{Hand, HandTestResult};
use super::{PlayerHandJp4s17t, WinningPoint};

pub(crate) struct PlayerBroker(pub(crate) PlayerHandJp4s17t);

impl crate::game::PlayerBroker for PlayerBroker {
    type Point = WinningPoint;
    type PlayerHand = PlayerHandJp4s17t;
    type Tile = Tile;
    // trait Hand = Hand<Point=Self::Point, PlayerHand=Self::PlayerHand, Tile=Self::Tile>;

    fn get_options_on_drawing(&self, possible_hands: &Vec<&dyn Hand<Point=Self::Point, PlayerHand=Self::PlayerHand, Tile=Self::Tile>>, drawn_tile: &Self::Tile) -> Vec<TurnChoice<Self::Tile>> {
        let mut tiles = self.0.closed_tiles.clone();
        tiles.insert(*drawn_tile);
        let mut buckets = tiles.get_by_buckets();
        let mut options = vec![];
        options.extend(buckets.clone().filter(|(_, &n)| n == 4).map(|(&t, _)| TurnChoice::MakeConcealedKong(t)));
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

    fn get_options_for_meld(&self, discarded_tile: &Self::Tile) -> Vec<MeldChoice<Self::Tile>> { // TODO change name to when discarded
        vec![MeldChoice::DoNothing] // FIXME
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
    use crate::game::{TurnChoice, PlayerBroker as _};

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
}
