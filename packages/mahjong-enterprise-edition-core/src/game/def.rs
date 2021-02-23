pub(crate) const PLAYERS_COUNT: usize = 4;

pub trait Concept {
    type Tile: Copy;
    type Meld;
}

pub struct DealtResult<C: Concept> {
    pub(crate) wall_tiles: Vec<C::Tile>,
    pub(crate) supplemental_tiles: Vec<C::Tile>,
    pub(crate) reward_indication_tiles: Vec<C::Tile>,
    pub(crate) player_tiles: [(Vec<C::Tile>, SeatOld); PLAYERS_COUNT],
}

impl<C: Concept> DealtResult<C> {
    pub(crate) fn new(
        wall_tiles: Vec<C::Tile>,
        supplemental_tiles: Vec<C::Tile>,
        reward_indication_tiles: Vec<C::Tile>,
        player_tiles: [(Vec<C::Tile>, SeatOld); PLAYERS_COUNT],
    ) -> DealtResult<C> {
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
pub(crate) enum SeatOld {
    East,
    South,
    West,
    North,
}

impl SeatOld {
    pub(crate) fn next_seat(&self) -> SeatOld {
        ((usize::from(*self) + 1) % 4).into()
    }
}

impl From<usize> for SeatOld {
    fn from(value: usize) -> Self {
        use SeatOld::*;

        match value {
            0 => East,
            1 => South,
            2 => West,
            3 => North,
            _ => panic!(format!("Invalid value: {}", value)),
        }
    }
}

impl From<SeatOld> for usize {
    fn from(seat: SeatOld) -> Self {
        use SeatOld::*;

        match seat {
            East => 0,
            South => 1,
            West => 2,
            North => 3,
        }
    }
}

pub enum Action<C: Concept> {
    Discard(C::Tile),
    Pass,
    MakeMeld(C::Meld),
    DeclareReady(C::Tile),
    DeclareCompletion,
}

pub trait ActionPolicy<C: Concept> {
    fn action_after_draw(&self, drawn_tile: C::Tile) -> Action<C>;
}
