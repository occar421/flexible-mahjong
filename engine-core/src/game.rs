use rand::prelude::*;
use std::rc::Rc;
use std::collections::BinaryHeap;
use crate::tile::{Suite, Tile};
use crate::players::Player;
use std::iter::FromIterator;

trait Game {
    fn start_a_match(&mut self) {
        let mut rng = thread_rng();
        self.do_a_match_with_rng(&mut rng);
    }
    fn do_a_match_with_rng<R: Rng + ?Sized>(&mut self, rng: &mut R);
}

struct GameState<P: Player + Sized> {
    round: u8,
    r#match: u8,
    players: Rc<[P; 4]>,
}

impl<P: Player + Sized> GameState<P> {
    fn new(players: Rc<[P; 4]>) -> GameState<P> {
        return GameState {
            round: 0,
            r#match: 0,
            players,
        };
    }
}

struct Game4s17t<P: Player + Sized> {
    state: GameState<P>
}

const N_OF_TILES: u8 = 9 * 4 * 4 + 4 * 4 + 4 * 4;

impl<P: Player + Sized> Game4s17t<P> {
    fn new(players: Rc<[P; 4]>) -> Game4s17t<P> {
        return Game4s17t {
            state: GameState::new(players)
        };
    }
}

impl<P: Player + Sized> Game for Game4s17t<P> {
    fn do_a_match_with_rng<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        // dealing tiles, 配牌作業
        let mut tiles: Vec<_> = {
            let mut tiles_seeds = (0..N_OF_TILES)
                .map(|i| (i, rng.gen::<u64>())).collect::<Vec<_>>();
            // Shuffling tiles
            tiles_seeds.sort_by_key(|a| a.1);
            tiles_seeds.iter().map(|(i, _)| {
                let i = i % (N_OF_TILES / 4);
                let suite = match i % 4 {
                    0 => Suite::Red,
                    1 => Suite::Green,
                    2 => Suite::White,
                    3 => Suite::Black,
                    _ => panic!() // won't happen
                };
                match i / 4 {
                    n @ 0..=8 => Tile::Number(suite, n + 1),
                    9 => Tile::Wind(suite),
                    10 => Tile::Symbol(suite),
                    _ => panic!() // won't happen
                }
            }).collect()
        };
        // 配牌
        let mut hands: [BinaryHeap<&Tile>; 4] = [
            BinaryHeap::from_iter(tiles.iter().take(16)),
            BinaryHeap::from_iter(tiles.iter().skip(16).take(16)),
            BinaryHeap::from_iter(tiles.iter().skip(32).take(16)),
            BinaryHeap::from_iter(tiles.iter().skip(48).take(16))
        ];
        // 壁牌
        let mut wall = {
            tiles.drain(0..64);
            tiles
        };
        // ドラ表示牌
        let mut reward_indication_tiles = vec![wall.pop().unwrap()]; // must exist

        unimplemented!();
    }
}
