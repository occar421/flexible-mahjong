use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub trait Concept {
    type Tile;
    type Meld;
    type Action;
}

pub trait TileDealingSpec<C: Concept> {}

struct TableFacade<C: Concept>(Rc<RefCell<Table<C>>>);

struct Table<C: Concept> {
    tile_dealing_spec: Box<dyn TileDealingSpec<C>>,
    wall_tiles: Vec<C::Tile>,
    supplemental_tiles: Vec<C::Tile>,
    reward_indication_tiles: Vec<C::Tile>,
    progress: Progress,
    players: Option<[(Player<C>, Seat); 4]>,
}

impl<C: Concept> TableFacade<C> {
    pub fn new(tile_dealing_spec: Box<dyn TileDealingSpec<C>>) -> TableFacade<C> {
        TableFacade(Rc::new(RefCell::new(
            Table {
                tile_dealing_spec,
                wall_tiles: vec![],
                supplemental_tiles: vec![],
                reward_indication_tiles: vec![],
                progress: Progress::get_initial(),
                players: None,
            })))
    }

    fn map_player(&self, player: (Box<dyn ActionPolicy<C>>, Seat)) -> (Player<C>, Seat) {
        let self_ref = Rc::downgrade(&self.0.clone());
        (Player::new(self_ref, player.0), player.1)
    }

    pub fn join_users(&mut self, players: [(Box<dyn ActionPolicy<C>>, Seat); 4]) {
        let [player0, player1, player2, player3] = players;
        self.0.borrow_mut().players = Some([
            self.map_player(player0),
            self.map_player(player1),
            self.map_player(player2),
            self.map_player(player3),
        ]);
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

#[derive(Copy, Clone)]
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
    table: Weak<RefCell<Table<C>>>,
}

impl<C: Concept> Player<C> {
    fn new<'a>(table: Weak<RefCell<Table<C>>>, action_policy: Box<dyn ActionPolicy<C>>) -> Player<C> {
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
