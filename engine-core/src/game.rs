use rand::prelude::*;
use std::rc::Rc;
use crate::players::Player;
use crate::tile::Tile;
use crate::hands::Hand;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TurnChoice<TTile: Tile> {
    /// 打牌
    Discard(TTile, usize),
    /// 暗槓
    MakeConcealedKong(TTile),
    /// 加槓
    MakeKongFromPong(TTile),
    /// 立直
    DeclareReady(TTile, usize),
    /// 自摸
    Complete,
}

pub enum MeldChoice<TTile: Tile> {
    /// 大明槓
    MakeExposedKong,
    /// ポン
    MakePong,
    /// チー
    MakeChow([TTile; 2]),
    /// 搶槓
    RobKong,
    /// ロン
    Complete,
    DoNothing,
}

pub enum Side {
    /// 上家
    Left,
    /// 対面
    Opposite,
    /// 下家
    Right,
}

pub(crate) enum Meld<TTile: Tile> {
    Pong([TTile; 3], Side),
    Chow([TTile; 3], Side),
    Kong([TTile; 4], Option<Side>),
}

pub trait PlayerHand<TTile: Tile> {
    type Point; // FIXME
    // trait ? = Hand<Self, Point=Self::Point, Tile=TTile>;

    fn get_options_on_drawing(&self, possible_hands: &Vec<&dyn Hand<Self, Point=Self::Point, Tile=TTile>>, drawn_tile: &TTile) -> Vec<TurnChoice<TTile>>;

    fn get_options_for_meld(&self, discarded_tile: &TTile) -> Vec<MeldChoice<TTile>>;

    fn discard(&mut self, drawn_tile: &TTile, tile: &TTile, index: usize);

    fn add_tile_to_discard_pile(&mut self, tile: &TTile, is_used_in_meld: bool);

    fn is_ready(&self) -> bool;
}

pub trait Game {
    fn start_a_match(&mut self) {
        let mut rng = thread_rng();
        self.do_a_match_with_rng(&mut rng);
    }
    fn do_a_match_with_rng<R: Rng + ?Sized>(&mut self, rng: &mut R);
}

pub const N_PLAYER: usize = 4;

pub(crate) struct GameState<P: Player + Sized> {
    round: u8,
    r#match: u8,
    pub(crate) players: [Rc<P>; N_PLAYER],
    pub(crate) dealer_index: usize,
}

impl<P: Player + Sized> GameState<P> {
    pub(crate) fn new(players: [Rc<P>; N_PLAYER]) -> GameState<P> {
        GameState {
            round: 0,
            r#match: 0,
            players,
            dealer_index: 0,
        }
    }
}
