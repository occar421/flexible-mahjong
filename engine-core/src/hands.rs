use crate::tile::Tile;
use crate::game::PlayerHand;

#[derive(Eq, PartialEq, Debug)]
pub enum HandTestResult<W> {
    Winning(W),
    Nothing,
}

// 役
pub trait Hand<TPlayerHand: PlayerHand<Self::Tile>> {
    type Point;
    type Tile: Tile;

    fn test_completion_on_drawing(&self, player_hand: &TPlayerHand, drawn_tile: &Self::Tile) -> HandTestResult<Self::Point>;
    fn test_completion_when_discarded(&self, player_hand: &TPlayerHand, discarded_tile: &Self::Tile) -> HandTestResult<Self::Point>;
}
