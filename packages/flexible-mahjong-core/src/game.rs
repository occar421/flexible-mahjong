use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

pub trait Concept {
    type Tile;
    type Meld;
    type Action;
}

const PLAYERS_COUNT: usize = 4;

struct DealtResult<C: Concept> {
    wall_tiles: Vec<C::Tile>,
    supplemental_tiles: Vec<C::Tile>,
    reward_indication_tiles: Vec<C::Tile>,
    player_tiles: [(Vec<C::Tile>, Seat); 4],
}

pub trait TileDealingSpec<C: Concept> {
    fn deal(&self) -> DealtResult<C>;
}

struct Table<C: Concept>(Rc<RefCell<TableContent<C>>>);

struct TableContent<C: Concept> {
    tile_dealing_spec: Box<dyn TileDealingSpec<C>>,
    wall_tiles: Vec<C::Tile>,
    supplemental_tiles: Vec<C::Tile>,
    reward_indication_tiles: Vec<C::Tile>,
    progress: Progress,
    players: RefCell<Option<[(Player<C>, Seat); PLAYERS_COUNT]>>,
}

impl<C: Concept> Table<C> {
    pub fn new(tile_dealing_spec: Box<dyn TileDealingSpec<C>>) -> Table<C> {
        Table(Rc::new(RefCell::new(
            TableContent {
                tile_dealing_spec,
                wall_tiles: vec![],
                supplemental_tiles: vec![],
                reward_indication_tiles: vec![],
                progress: Progress::get_initial(),
                players: RefCell::new(None),
            })))
    }

    fn map_player(&self, player: (Box<dyn ActionPolicy<C>>, Seat)) -> (Player<C>, Seat) {
        let self_ref = Rc::downgrade(&self.0.clone());
        (Player::new(self_ref, player.0), player.1)
    }

    fn join_users(&mut self, players: [(Box<dyn ActionPolicy<C>>, Seat); PLAYERS_COUNT]) {
        let [player0, player1, player2, player3] = players;
        // TODO check duplication
        self.borrow_mut().players.replace(Some([
            self.map_player(player0),
            self.map_player(player1),
            self.map_player(player2),
            self.map_player(player3),
        ]));
    }

    fn deal_tiles(&mut self) {
        let DealtResult { wall_tiles, supplemental_tiles, reward_indication_tiles, player_tiles } = self.borrow().tile_dealing_spec.deal();
        let mut table = self.borrow_mut();
        table.wall_tiles = wall_tiles;
        table.supplemental_tiles = supplemental_tiles;
        table.reward_indication_tiles = reward_indication_tiles;
        let mut players = table.players.borrow_mut();
        if let Some(ref mut players) = *players {
            for (tiles, seat) in player_tiles.iter() {
                let position = players.iter().position(|(_, seat2)| seat2 == seat).unwrap();
                players[position]
            }
        }
        // table.players.borrow_mut().expect("Player should have joined");
        // let mut players = table.players.expect("Player should have joined");
    }
}

impl<C: Concept> Deref for Table<C> {
    type Target = Rc<RefCell<TableContent<C>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: Concept> DerefMut for Table<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone)]
struct Progress {
    current_hand: (Round, u8),
    deals_count: u8,
}

impl Progress {
    pub fn get_initial() -> Progress {
        Progress {
            current_hand: (Round::East, 1),
            deals_count: 0,
        }
    }
}

#[derive(Copy, Clone)]
enum Round {
    East,
    South,
    West,
    North,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Seat {
    East,
    South,
    West,
    North,
}

pub trait ActionPolicy<C: Concept> {}

struct Player<C: Concept> {
    point: u32,
    action_policy: Box<dyn ActionPolicy<C>>,
    concealed_tiles: Vec<C::Tile>,
    exposed_melds: Vec<C::Meld>,
    discarded_tiles: Vec<C::Tile>,
    table: Weak<RefCell<TableContent<C>>>,
}

impl<C: Concept> Player<C> {
    fn new<'a>(table: Weak<RefCell<TableContent<C>>>, action_policy: Box<dyn ActionPolicy<C>>) -> Player<C> {
        Player {
            point: 0,
            action_policy,
            concealed_tiles: vec![],
            exposed_melds: vec![],
            discarded_tiles: vec![],
            table,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::game::{Table, TileDealingSpec, Concept, DealtResult};
    use crate::game::Seat::East;

    struct ConceptMock;

    impl Concept for ConceptMock {
        type Tile = ();
        type Meld = ();
        type Action = ();
    }

    struct SpecMock;

    impl TileDealingSpec<ConceptMock> for SpecMock {
        fn deal(&self) -> DealtResult<ConceptMock> {
            DealtResult {
                wall_tiles: vec![(), ()],
                supplemental_tiles: vec![],
                reward_indication_tiles: vec![],
                player_tiles: [(vec![], East); 4],
            }
        }
    }

    #[test]
    fn a() {
        let spec = Box::new(SpecMock {});
        let mut table = Table::new(spec);
        table.deal_tiles();
    }
}