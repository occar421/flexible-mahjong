use crate::tile::Tile;
use crate::game::PlayerHand;

#[derive(Eq, PartialEq, Debug)]
pub enum HandTestResult {
    Winning(u8),
    Nothing,
}

// 役
pub trait Hand<TPlayerHand: PlayerHand<Self::Tile>> {
    type Tile: Tile;

    fn test_with_drawn_tile(&self, player_hand: &TPlayerHand, drawn_tile: &Self::Tile) -> HandTestResult;
    fn test_with_discarded_tile(&self, player_hand: &TPlayerHand, discarded_tile: &Self::Tile) -> HandTestResult;
}
