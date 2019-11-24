use crate::tile::Tile;

#[derive(Eq, PartialEq, Debug)]
pub enum HandTestResult<W> {
    Winning(W),
    Nothing,
}

// 役
pub trait Hand {
    type Point;
    type PlayerHand;
    type Tile: Tile;

    fn test_completion_on_drawing(&self, player_hand: &Self::PlayerHand, drawn_tile: &Self::Tile) -> HandTestResult<Self::Point>;
    fn test_completion_when_discarded(&self, player_hand: &Self::PlayerHand, discarded_tile: &Self::Tile) -> HandTestResult<Self::Point>;
}
