mod tile {
    use std::cmp::Ordering;

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub(crate) enum Suite {
        /// Green, East, S
        /// 發、東、索子
        Green,
        /// Red, South, M
        /// 中、南、萬子
        Red,
        /// White, West, P
        /// 囗、西、筒子
        White,
        /// Black, North, ?
        /// ？、北、？子
        Black,
    }

    impl crate::tile::Suite for Suite {}

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub(crate) enum Tile {
        /// 数牌
        Number(Suite, u8),
        /// 風牌
        Wind(Suite),
        /// 三元牌相当
        Symbol(Suite), // Dragon
    }

    impl Ord for Tile {
        fn cmp(&self, other: &Self) -> Ordering {
            match self {
                Tile::Number(s1, n1) => match other {
                    Tile::Number(s2, n2) => match s1.cmp(&s2) {
                        Ordering::Equal => n1.cmp(n2),
                        o => o
                    },
                    _ => Ordering::Less
                },
                Tile::Wind(s1) => match other {
                    Tile::Number(_, _) => Ordering::Greater,
                    Tile::Wind(s2) => s1.cmp(&s2),
                    Tile::Symbol(_) => Ordering::Less
                },
                Tile::Symbol(s1) => match other {
                    Tile::Symbol(s2) => s1.cmp(&s2),
                    _ => Ordering::Greater
                }
            }
        }
    }

    impl PartialOrd for Tile {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl crate::tile::Tile for Tile { type Suite = Suite; }
}

mod structure {
    use std::borrow::Borrow;
    use std::collections::BTreeMap;
    use std::iter::FromIterator;
    use itertools::Itertools;

    pub(crate) struct MultiBTreeSet<T> {
        map: BTreeMap<T, usize>
    }

    impl<T: Ord> MultiBTreeSet<T> {
        pub fn insert(&mut self, value: T) -> bool {
            if let Some(n) = self.map.get_mut(&value) {
                *n += 1;
            } else {
                self.map.insert(value, 1);
            }
            true
        }

        pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
            where T: Borrow<Q>,
                  Q: Ord
        {
            if let Some(n) = self.map.get_mut(value) {
                *n -= 1;
                if n == &0 {
                    self.map.remove(value);
                }
                true
            } else {
                false
            }
        }
    }

    impl<T: Ord + Copy> FromIterator<T> for MultiBTreeSet<T> {
        fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> MultiBTreeSet<T> {
            MultiBTreeSet {
                map: BTreeMap::from_iter(
                    iter.into_iter()
                        .group_by(|e| e.clone())
                        .into_iter()
                        .map(|(k, gv)| (k, gv.count()))
                )
            }
        }
    }
}

mod game {
    use std::iter::FromIterator;
    use std::rc::Rc;
    use rand::Rng;

    use crate::game::{Game, GameState, Meld, MeldChoice, N_PLAYER, PlayerHand, TurnChoice};
    use crate::players::Player;

    use super::structure::MultiBTreeSet;
    use super::tile::{Suite, Tile};

    const N_TILES: u8 = 9 * 4 * 4 + 4 * 4 + 4 * 4;

    pub(crate) struct PlayerHandJp4s17t {
        closed_tiles: MultiBTreeSet<Tile>,
        melds: Vec<Meld<Tile>>,
        discard_pile: Vec<(Tile, bool)>,
    }

    impl PlayerHandJp4s17t {
        pub(crate) fn new<I: IntoIterator<Item=Tile>>(tiles: I) -> PlayerHandJp4s17t {
            PlayerHandJp4s17t {
                closed_tiles: MultiBTreeSet::from_iter(tiles),
                melds: vec![],
                discard_pile: vec![],
            }
        }
    }

    impl PlayerHand<Tile> for PlayerHandJp4s17t {
        fn get_options_on_drawing(&mut self, drawn_tile: &Tile) -> Vec<TurnChoice<Tile>> {
            self.closed_tiles.insert(*drawn_tile);
            vec![TurnChoice::Discard(*drawn_tile, 0)]
        }

        fn get_options_for_meld(&mut self, discarded_tile: &Tile) -> Vec<MeldChoice<Tile>> {
            vec![MeldChoice::DoNothing]
        }

        fn discard(&mut self, tile: &Tile, _: usize) {
            if self.closed_tiles.remove(tile) {
                return;
            }
            panic!("Can't discard because they don't have it");
        }

        fn add_tile_to_discard_pile(&mut self, tile: &Tile, is_used_in_meld: bool) {
            self.discard_pile.push((*tile, is_used_in_meld));
        }

        // is 聴牌
        fn is_ready(&self) -> bool {
            false
        }
    }

    enum MatchResult {
        RunningOut(Vec<usize>)
    }

    struct GameJp4s17t<P: Player + Sized> {
        state: GameState<P>
    }

    impl<P: Player<Tile=Tile> + Sized> GameJp4s17t<P> {
        fn new(players: [Rc<P>; 4]) -> GameJp4s17t<P> {
            GameJp4s17t {
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

            let mut hands: [_; N_PLAYER] = [
                PlayerHandJp4s17t::new(players_tiles[0].clone()),
                PlayerHandJp4s17t::new(players_tiles[1].clone()),
                PlayerHandJp4s17t::new(players_tiles[2].clone()),
                PlayerHandJp4s17t::new(players_tiles[3].clone()),
            ];

            for i in 0..N_PLAYER {
                self.state.players[i].set_dealt_tiles(&players_tiles[i]);
            }

            // ドラ表示牌 数
            let mut n_rewards = 1;

            let mut turn_index = self.state.dealer_index;
            // start game
            while let Some(drawn_tile) = wall.pop() {
                let player = &self.state.players[turn_index];
                let hand = &mut hands[turn_index];
                let options = hand.get_options_on_drawing(&drawn_tile);

                // 自摸
                let choice = player.draw(&drawn_tile, &options);

                match choice {
                    TurnChoice::Discard(discarded_tile, index) => {
                        hand.discard(&discarded_tile, index);
                        hand.add_tile_to_discard_pile(&discarded_tile, false);
                    }
                    _ => unimplemented!()
                }

                turn_index = (turn_index + 1) % N_PLAYER;
            }

            // running out, 流局
            MatchResult::RunningOut((0..N_PLAYER)
                .map(|i| (i, hands[i].is_ready()))
                .filter(|(_, r)| *r)
                .map(|(i, _)| i)
                .collect())
        }
    }

    impl<P: Player<Tile=Tile> + Sized> Game for GameJp4s17t<P> {
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
        use std::fmt::{Debug, Error, Formatter};
        use std::rc::Rc;

        use colored::*;
        use itertools::Itertools;

        use crate::game::{Game, MeldChoice, TurnChoice};
        use crate::players::Player;

        use super::GameJp4s17t;
        use super::super::tile::{Suite, Tile};

        pub struct OnlyDiscardFakePlayer;

        impl OnlyDiscardFakePlayer {
            pub fn new() -> OnlyDiscardFakePlayer {
                OnlyDiscardFakePlayer {}
            }
        }

        impl Debug for Tile {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                const NUMBERS: [&str; 9] = ["一", "二", "三", "四", "伍", "六", "七", "八", "九"];
                const CORDS: [&str; 9] = ["１", "２", "３", "４", "５", "６", "７", "８", "９"];
                const COINS: [&str; 9] = ["①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨"];
                const UNKNOWNS: [&str; 9] = ["１⃣", "２⃣", "３⃣", "４⃣", "５⃣", "６⃣", "７⃣", "８⃣", "９⃣"];

                write!(f, "{}", match self {
                    Tile::Number(s, n) => match s {
                        Suite::Red => format!("{}", NUMBERS[*n as usize - 1]).red(),
                        Suite::Green => format!("{}", CORDS[*n as usize - 1]).green().underline(),
                        Suite::White => format!("{}", COINS[*n as usize - 1]).yellow(),
                        Suite::Black => format!("{}", UNKNOWNS[*n as usize - 1]).magenta(),
                    }
                    ,
                    Tile::Wind(s) => match s {
                        Suite::Red => "東",
                        Suite::Green => "南",
                        Suite::White => "西",
                        Suite::Black => "北",
                    }.to_string().cyan(),
                    Tile::Symbol(s) => match s {
                        Suite::Red => "中".red(),
                        Suite::Green => "發".green(),
                        Suite::White => "　⃣".yellow(),
                        Suite::Black => "？".magenta(),
                    },
                })
            }
        }

        impl Player for OnlyDiscardFakePlayer {
            type Tile = Tile;

            fn set_dealt_tiles(&self, tiles: &Vec<Self::Tile>) {
                let mut tiles = tiles.clone();
                tiles.sort_unstable();
                println!("{}", tiles.into_iter().map(|t| format!("{:#?}", t)).join(" "));
            }

            fn draw(&self, drawn_tile: &Self::Tile, _: &Vec<TurnChoice<Tile>>) -> TurnChoice<Tile> {
                TurnChoice::Discard(*drawn_tile, 0)
            }

            fn consider_melding(&self, _: &Self::Tile, _: &Vec<MeldChoice<Tile>>) -> MeldChoice<Tile> {
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
            let mut m = GameJp4s17t::new(players);
            m.start_a_match();
        }

        #[test]
        fn test_match_core() {
            use super::super::tile::Tile::{Number, Wind, Symbol};
            use super::super::tile::Suite::{Green, Red, White, Black};

            let players = [
                Rc::new(OnlyDiscardFakePlayer::new()),
                Rc::new(OnlyDiscardFakePlayer::new()),
                Rc::new(OnlyDiscardFakePlayer::new()),
                Rc::new(OnlyDiscardFakePlayer::new()),
            ];
            let mut m = GameJp4s17t::new(players);
            let mut all_green: Vec<_> = [2, 3, 4, 6, 8].iter().map(|&n| Number(Green, n)).map(|t| vec![t, t, t]).flatten().collect();
            all_green.extend(vec![Symbol(Green)]);
            let sixteen_orphans = vec![
                Number(Green, 1), Number(Green, 9), Number(Red, 1), Number(Red, 9), Number(White, 1), Number(White, 9), Number(Black, 1), Number(Black, 9),
                Wind(Green), Wind(Red), Wind(White), Wind(Black), Number(Green, 1) /*Symbol(Green)*/, Symbol(Red), Symbol(White), Symbol(Black)
            ];
            let mut four_winds: Vec<_> = [Green, Red, White, Black].iter().map(|&s| Wind(s)).map(|t| vec![t, t, t]).flatten().collect();
            four_winds.extend(vec![Number(Red, 2), Number(Red, 2), Number(Red, 2), Number(Red, 3)]);
            let mut four_dragons: Vec<_> = [Green, Red, White, Black].iter().map(|&s| Symbol(s)).map(|t| vec![t, t, t]).flatten().collect();
            four_dragons.extend(vec![Number(White, 2), Number(White, 2), Number(White, 2), Number(White, 3)]);
            m.match_core(&vec![], &vec![], &[&all_green, &sixteen_orphans, &four_winds, &four_dragons]);
        }
    }
}