use std::iter::{FromIterator, once};
use std::rc::Rc;
use rand::Rng;

use crate::game::{GameModerator, GameState, Meld, MeldChoice, N_PLAYER, PlayerBroker, TurnChoice};
use crate::players::Player;
use crate::collections::MultiBTreeSet;

use super::tile::{Suite, Tile};
use crate::hands::{Hand, HandTestResult};
use super::hands::*;

const N_TILES: u8 = 9 * 4 * 4 + 4 * 4 + 4 * 4;

#[derive(Eq, PartialEq, Debug)]
pub(crate) enum WinningPoint {
    // 飜
    Fan(u8),
    // 役満 (could be double Yakuman or triple Yakuman etc.)
    Yakuman(u8),
}

pub(crate) struct PlayerHandJp4s17t {
    pub(crate) closed_tiles: MultiBTreeSet<Tile>,
    pub(crate) melds: Vec<Meld<Tile>>,
    pub(crate) discard_pile: Vec<(Tile, bool)>,
}

impl PlayerHandJp4s17t {
    pub(crate) fn new<I: IntoIterator<Item=Tile>>(tiles: I) -> PlayerHandJp4s17t {
        PlayerHandJp4s17t {
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
}

#[cfg(test)]
impl PlayerHandJp4s17t {
    pub(crate) fn create<I: IntoIterator<Item=Tile>>(tiles: I, melds: Vec<Meld<Tile>>, discard_pile: Vec<(Tile, bool)>) -> PlayerHandJp4s17t {
        PlayerHandJp4s17t {
            closed_tiles: MultiBTreeSet::from_iter(tiles),
            melds,
            discard_pile,
        }
    }
}

struct PlayerBrokerJp4s17t(PlayerHandJp4s17t);

impl PlayerBroker for PlayerBrokerJp4s17t {
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

enum MatchResult {
    RunningOut(Vec<usize>)
}

struct GameModeratorJp4s17t<P: Player + Sized> {
    state: GameState<P>
}

impl<P: Player<Tile=Tile> + Sized> GameModeratorJp4s17t<P> {
    fn new(players: [Rc<P>; 4]) -> GameModeratorJp4s17t<P> {
        GameModeratorJp4s17t {
            state: GameState::new(players)
        }
    }

    // can we generalize this function like match_jp ?
    fn match_core(&mut self, wall: &Vec<Tile>, dead_wall: &Vec<Tile>, players_tiles: &[&Vec<Tile>; N_PLAYER]) -> MatchResult {
        let mut wall = wall.clone();
        let mut dead_wall = dead_wall.clone();

        // ドラ牌
        let reward_indication_tiles = dead_wall.split_off(4).chunks(2).map(|c| (c[0], c[1])).collect::<Vec<_>>();

        // 嶺上牌
        let supplemental_tiles = dead_wall;

        let mut brokers: [_; N_PLAYER] = [
            PlayerBrokerJp4s17t(PlayerHandJp4s17t::new(players_tiles[0].clone())),
            PlayerBrokerJp4s17t(PlayerHandJp4s17t::new(players_tiles[1].clone())),
            PlayerBrokerJp4s17t(PlayerHandJp4s17t::new(players_tiles[2].clone())),
            PlayerBrokerJp4s17t(PlayerHandJp4s17t::new(players_tiles[3].clone())),
        ];

        for i in 0..N_PLAYER {
            self.state.players[i].set_dealt_tiles(&players_tiles[i]);
        }

        let eight_pairs_and_half = FanHand::<EightPairsAndHalf>::new(2, 1);
        let all_in_triplets = FanHand::<AllInTriplets>::new(2, 2);
        let sixteen_orphans = YakumanHand::<SixteenOrphans>::new(1, 2);

        let static_hands: [&dyn Hand<PlayerHand=PlayerHandJp4s17t, Point=WinningPoint, Tile=Tile>; 3] = [
            &eight_pairs_and_half,
            &all_in_triplets,
            &sixteen_orphans
        ];

        // ドラ表示牌 数
        let mut n_rewards = 1;

        let mut turn_index = self.state.dealer_index;
        // start game
        while let Some(drawn_tile) = wall.pop() {
            let player = &self.state.players[turn_index];
            let broker = &mut brokers[turn_index];
            let possible_hands = static_hands.to_vec();
            let options = broker.get_options_on_drawing(&possible_hands, &drawn_tile);

            // 自摸
            let choice = player.draw(&drawn_tile, &options);

            match choice {
                TurnChoice::Discard(discarded_tile, index) => {
                    broker.discard(&drawn_tile, &discarded_tile, index);
                    broker.add_tile_to_discard_pile(&discarded_tile, false);
                }
                _ => unimplemented!()
            }

            turn_index = (turn_index + 1) % N_PLAYER;
        }

        // running out, 流局
        MatchResult::RunningOut((0..N_PLAYER)
            .map(|i| (i, brokers[i].is_ready()))
            .filter(|(_, r)| *r)
            .map(|(i, _)| i)
            .collect())
    }
}

impl<P: Player<Tile=Tile> + Sized> GameModerator for GameModeratorJp4s17t<P> {
    fn do_a_match_with_rng<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        // dealing tiles, 配牌作業

        // 壁牌
        let mut wall: Vec<_> = {
            let mut tiles_seeds = (0..N_TILES)
                .map(|i| (i, rng.gen::<u64>())).collect::<Vec<_>>();
            // Shuffling tiles
            tiles_seeds.sort_by_key(|a| a.1);
            tiles_seeds.iter().map(|(i, _)| {
                let i = i % (N_TILES / 4);
                let suite = match i % 4 {
                    0 => Suite::Red,
                    1 => Suite::Green,
                    2 => Suite::White,
                    3 => Suite::Black,
                    _ => unreachable!()
                };
                match i / 4 {
                    n @ 0..=8 => Tile::Number(suite, n + 1),
                    9 => Tile::Wind(suite),
                    10 => Tile::Symbol(suite),
                    _ => unreachable!()
                }
            }).collect()
        };

        // 王牌
        let dead_wall: Vec<_> = wall.drain(0..14).collect();

        // 配牌
        let players_tiles: [&Vec<_>; N_PLAYER] = [
            &wall.drain(0..16).collect(),
            &wall.drain(0..16).collect(),
            &wall.drain(0..16).collect(),
            &wall.drain(0..16).collect(),
        ];

        let wall = wall;

        let result = self.match_core(&wall, &dead_wall, &players_tiles);

        match result {
            MatchResult::RunningOut(ready_players) =>
                if !ready_players.iter().find(|&&i| i == self.state.dealer_index).is_none() {
                    // move dealer if it was not ready
                    self.state.dealer_index = (self.state.dealer_index + 1) & N_PLAYER;
                }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use itertools::Itertools;

    use crate::game::{GameModerator, MeldChoice, TurnChoice, PlayerBroker};
    use crate::players::Player;

    use super::{GameModeratorJp4s17t, PlayerHandJp4s17t, FanHand, PlayerBrokerJp4s17t};
    use super::super::tile::Tile;
    use super::super::tile::Tile::{Number, Wind, Symbol};
    use super::super::tile::Suite::{Green, Red, White, Black};
    use super::super::hands::AllInTriplets;
    use std::collections::HashSet;
    use std::collections::hash_map::RandomState;
    use std::iter::{FromIterator, once, repeat};

    pub struct OnlyDiscardFakePlayer;

    impl OnlyDiscardFakePlayer {
        pub fn new() -> OnlyDiscardFakePlayer {
            OnlyDiscardFakePlayer {}
        }
    }

    impl Player for OnlyDiscardFakePlayer {
        type Tile = Tile;

        fn set_dealt_tiles(&self, tiles: &Vec<Self::Tile>) {
            let mut tiles = tiles.clone();
            tiles.sort_unstable();
            println!("{}", tiles.into_iter().map(|t| format!("{:#?}", t)).join(" "));
        }

        fn draw(&self, drawn_tile: &Self::Tile, _: &Vec<TurnChoice<Self::Tile>>) -> TurnChoice<Self::Tile> {
            TurnChoice::Discard(*drawn_tile, 0)
        }

        fn consider_melding(&self, _: &Self::Tile, _: &Vec<MeldChoice<Self::Tile>>) -> MeldChoice<Self::Tile> {
            MeldChoice::DoNothing
        }
    }

    #[test]
    fn finishes_match() {
        let players = [
            Rc::new(OnlyDiscardFakePlayer::new()),
            Rc::new(OnlyDiscardFakePlayer::new()),
            Rc::new(OnlyDiscardFakePlayer::new()),
            Rc::new(OnlyDiscardFakePlayer::new()),
        ];
        let mut m = GameModeratorJp4s17t::new(players);
        m.start_a_match();
    }

    #[test]
    fn test_match_core() {
        let players = [
            Rc::new(OnlyDiscardFakePlayer::new()),
            Rc::new(OnlyDiscardFakePlayer::new()),
            Rc::new(OnlyDiscardFakePlayer::new()),
            Rc::new(OnlyDiscardFakePlayer::new()),
        ];
        let mut m = GameModeratorJp4s17t::new(players);
        let all_green: Vec<_> = [2, 3, 4, 6, 8].iter()
            .flat_map(|&n| repeat(Number(Green, n)).take(3))
            .chain(once(Symbol(Green)))
            .collect();
        let sixteen_orphans = vec![
            Number(Green, 1), Number(Green, 9), Number(Red, 1), Number(Red, 9), Number(White, 1), Number(White, 9), Number(Black, 1), Number(Black, 9),
            Wind(Green), Wind(Red), Wind(White), Wind(Black), Number(Green, 1) /*Symbol(Green)*/, Symbol(Red), Symbol(White), Symbol(Black)
        ];
        let four_winds: Vec<_> = [Green, Red, White, Black].iter()
            .flat_map(|&s| repeat(Wind(s)).take(3))
            .chain(vec![Number(Red, 2), Number(Red, 2), Number(Red, 2), Number(Red, 3)])
            .collect();
        let four_dragons: Vec<_> = [Green, Red, White, Black].iter()
            .flat_map(|&s| repeat(Symbol(s)).take(3))
            .chain(vec![Number(White, 2), Number(White, 2), Number(White, 2), Number(White, 3)])
            .collect();
        m.match_core(
            &vec![],
            &(2..=8).flat_map(|i| repeat(Number(Black, i)).take(2)).collect(),
            &[&all_green, &sixteen_orphans, &four_winds, &four_dragons],
        );
    }

    #[test]
    fn test_get_options_on_drawing() {
        let player_hand = PlayerHandJp4s17t::create(
            (1..=4).flat_map(|n| repeat(Number(Green, n)).take(4)),
            vec![], vec![]);
        let broker = PlayerBrokerJp4s17t(player_hand);
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
