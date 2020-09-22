pub(crate) const PLAYERS_COUNT: usize = 4;

pub trait Concept {
    type Tile: Copy;
    type Meld;
    type Action;
}

pub struct DealtResult<C: Concept> {
    pub(crate) wall_tiles: Vec<C::Tile>,
    pub(crate) supplemental_tiles: Vec<C::Tile>,
    pub(crate) reward_indication_tiles: Vec<C::Tile>,
    pub(crate) player_tiles: [(Vec<C::Tile>, Seat); PLAYERS_COUNT],
}

impl<C: Concept> DealtResult<C> {
    pub(crate) fn new(
        wall_tiles: Vec<C::Tile>,
        supplemental_tiles: Vec<C::Tile>,
        reward_indication_tiles: Vec<C::Tile>,
        player_tiles: [(Vec<C::Tile>, Seat); PLAYERS_COUNT])
        -> DealtResult<C> {
        DealtResult {
            wall_tiles,
            supplemental_tiles,
            reward_indication_tiles,
            player_tiles,
        }
    }
}

pub trait TileDealingSpec<C: Concept> {
    fn deal(&self) -> DealtResult<C>;
}


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Seat {
    East,
    South,
    West,
    North,
}

pub trait ActionPolicy<C: Concept> {
    fn action_after_draw(&self, drawn_tile: C::Tile) -> C::Action;
}
