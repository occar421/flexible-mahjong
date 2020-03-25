use std::iter::FromIterator;

use crate::game::{Meld, ClosedHandPart};
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

    pub(crate) fn calculate_possible_combinations(&self, new_tile: &Tile) -> Vec<Vec<ClosedHandPart<Tile>>> {
        fn a(buckets: Vec<(&Tile, &usize)>, current_combination: Vec<ClosedHandPart<Tile>>) -> Vec<Vec<ClosedHandPart<Tile>>> {
            if let Some((index, &(tile, &n))) = buckets.iter().find_position(|&(_, &n)| n > 0) {
                if let Tile::Number(suite, number) = tile {
                    let mut combinations = vec![];
                    {
                        // 順子優先
                        if let Some((next_index, (&next_tile, next_n))) = buckets.iter().find_position(|&(&t, &n)| t == Tile::Number(*suite, *number + 1) && n > 0) {
                            if let Some((after_the_next_index, (&after_the_next_tile, after_the_next_n))) = buckets.iter().find_position(|&(&t, &n)| t == Tile::Number(*suite, *number + 2) && n > 0) {
                                let mut buckets = buckets.clone();
                                let next_index_n_1 = *next_n - 1;
                                buckets[next_index] = (&buckets[next_index].0, &next_index_n_1);
                                let after_the_next_index_n_1 = *after_the_next_n - 1;
                                buckets[after_the_next_index] = (&buckets[after_the_next_index].0, &after_the_next_index_n_1);
                                let mut current_combination = current_combination.clone();
                                current_combination.push(ClosedHandPart::SequenceTrio([*tile, next_tile, after_the_next_tile]));
                                combinations.push(a(buckets, current_combination));
                            }
                        }
                    }
                    {
                        // 刻子優先
                        if n == 3 {
                            // 刻子あり
                            let mut buckets = buckets.clone();
                            buckets[index] = (tile, &0);
                            let mut current_combination = current_combination.clone();
                            current_combination.push(ClosedHandPart::Triple(*tile));
                            combinations.push(a(buckets, current_combination));
                        }
                    }
                    return combinations.iter().cloned().flatten().collect();
                } else {
                    if n == 3 {
                        // 刻子あり
                        let mut buckets = buckets.clone();
                        buckets[index] = (tile, &0);
                        let mut current_combination = current_combination.clone();
                        current_combination.push(ClosedHandPart::Triple(*tile));
                        return a(buckets, current_combination);
                    } else {
                        return vec![];
                    }
                }
            }
            return vec![current_combination];
        }

        let mut tiles = self.closed_tiles.clone();
        tiles.insert(*new_tile);
        let mut combinations = vec![];
        for (&pair_tile, _) in tiles.get_by_buckets().filter(|&(_, &n)| n >= 2) {
            let mut tiles = tiles.clone();
            tiles.remove(&pair_tile);
            tiles.remove(&pair_tile);

            let buckets: Vec<_> = tiles.get_by_buckets().sorted().collect();
            let combination = vec![ClosedHandPart::Double(pair_tile)];
            combinations.extend_from_slice(&a(buckets, combination));
        }

        return combinations;
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

#[cfg(test)]
mod tests {
    use super::PlayerHand;
    use super::super::super::tile::Tile;
    use super::super::super::tile::Tile::{Number, Wind, Symbol};
    use super::super::super::tile::Suite::{Green, Red, White, Black};
    use crate::game::ClosedHandPart;

    fn sorted(list: Vec<Vec<ClosedHandPart<Tile>>>) -> Vec<Vec<ClosedHandPart<Tile>>> {
        // TODO this
        unimplemented!();
    }

    #[test]
    fn empty_combination() {
        let player_hand = PlayerHand::create(vec![], vec![], vec![]);
        let result = player_hand.calculate_possible_combinations(&Symbol(Green));
        assert_eq!(result, Vec::<Vec<_>>::new());
    }

    #[test]
    fn one_pair_combination() {
        let player_hand = PlayerHand::create(vec![Symbol(Green)], vec![], vec![]);
        let result = player_hand.calculate_possible_combinations(&Symbol(Green));
        assert_eq!(result, vec![vec![ClosedHandPart::Double(Symbol(Green))]]);
    }

    #[test]
    fn one_pair_one_triple_combination() {
        let player_hand = PlayerHand::create(vec![Number(Green, 2), Number(Green, 2), Number(Green, 3), Number(Green, 3)], vec![], vec![]);
        let result = player_hand.calculate_possible_combinations(&Number(Green, 2));
        assert_eq!(sorted(result), sorted(vec![vec![ClosedHandPart::Double(Number(Green, 2)), ClosedHandPart::Triple(Number(Green, 3))]]));
    }
}