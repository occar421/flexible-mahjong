use std::rc::Rc;

pub trait Concept {
    type Tile;
    type Meld;
    type Action;
}

pub trait TileDealingSpec<C: Concept> {}

struct Table<'a, C: Concept> {
    tile_dealing_spec: Box<dyn TileDealingSpec<C>>,
    wall_tiles: Vec<C::Tile>,
    supplemental_tiles: Vec<C::Tile>,
    reward_indication_tiles: Vec<C::Tile>,
    progress: Progress,
    players: Option<[(Player<'a, C>, Seat); 4]>,
}

impl<C: Concept> Table<'_, C> {
    pub fn new(tile_dealing_spec: Box<dyn TileDealingSpec<C>>) -> Table<C> {
        Table {
            tile_dealing_spec,
            wall_tiles: vec![],
            supplemental_tiles: vec![],
            reward_indication_tiles: vec![],
            progress: Progress::get_initial(),
            players: None,
        }
    }

    fn map_player<'a>(table: Rc<&'a Table<'a, C>>, player: (Box<dyn ActionPolicy<C>>, Seat)) -> (Player<'a, C, >, Seat) {
        (Player::new(table, player.0), player.1)
    }

    pub fn join_users<'a>(&'a mut self, players: [(Box<dyn ActionPolicy<C>>, Seat); 4]) {
        let table: Rc<&'a _> = Rc::new(self);
        self.players = Some([
            Table::map_player(table.clone(), players[0]),
            Table::map_player(table.clone(), players[1]),
            Table::map_player(table.clone(), players[2]),
            Table::map_player(table.clone(), players[3]),
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

struct Player<'a, C: Concept> {
    point: u32,
    action_policy: Box<dyn ActionPolicy<C>>,
    concealed_tiles: Vec<C::Tile>,
    exposed_melds: Vec<C::Meld>,
    discarded_tiles: Vec<C::Tile>,
    table: Rc<&'a Table<'a, C>>,
}

impl<C: Concept> Player<'_, C> {
    fn new<'a>(table: Rc<&'a Table<C>>, action_policy: Box<dyn ActionPolicy<C>>) -> Player<'a, C> {
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