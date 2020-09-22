use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use arrayvec::ArrayVec;
use itertools::Itertools;

pub trait Concept {
    type Tile: Copy;
    type Meld;
    type Action;
}

const PLAYERS_COUNT: usize = 4;

struct DealtResult<C: Concept> {
    wall_tiles: Vec<C::Tile>,
    supplemental_tiles: Vec<C::Tile>,
    reward_indication_tiles: Vec<C::Tile>,
    player_tiles: [(Vec<C::Tile>, Seat); PLAYERS_COUNT],
}

pub trait TileDealingSpec<C: Concept> {
    fn deal(&self) -> DealtResult<C>;
}

struct Table<C: Concept>(Rc<RefCell<TableContent<C>>>);

struct TableContent<C: Concept> {
    tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>,
    wall_tiles: Vec<C::Tile>,
    supplemental_tiles: Vec<C::Tile>,
    reward_indication_tiles: Vec<C::Tile>,
    progress: Progress,
    players: RefCell<Option<ArrayVec<[(Player<C>, Seat); PLAYERS_COUNT]>>>,
}

impl<C: Concept> Table<C> {
    pub fn new(tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>) -> Table<C> {
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

    fn map_player(&self, player: (Rc<Box<dyn ActionPolicy<C>>>, Seat)) -> (Player<C>, Seat) {
        let self_ref = Rc::downgrade(&self.0.clone());
        (Player::new(self_ref, player.0), player.1)
    }

    fn join_users(&mut self, players: [(Rc<Box<dyn ActionPolicy<C>>>, Seat); PLAYERS_COUNT]) {
        let players = ArrayVec::from(players);

        {
            let groups = players.iter().group_by(|(_, s)| s);
            let a = groups.into_iter().collect_vec();
            if a.len() != PLAYERS_COUNT {
                panic!("Wrong arg `players`: seats should be unique")
            }
        }

        self.borrow_mut().players.replace(
            Some(players.iter().map(|(p, s)| self.map_player((p.clone(), *s))).collect())
        );
    }

    fn deal_tiles(&mut self) {
        {
            if self.borrow().players.borrow().is_none() {
                panic!("Should call after join_users")
            }
        }

        let DealtResult { wall_tiles, supplemental_tiles, reward_indication_tiles, player_tiles } = self.borrow().tile_dealing_spec.deal();

        {
            let groups = player_tiles.iter().group_by(|(_, s)| s);
            let a = groups.into_iter().collect_vec();
            if a.len() != PLAYERS_COUNT {
                panic!("Wrong arg `player_tiles`: seats should be unique")
            }
        }

        let mut table = self.borrow_mut();
        table.wall_tiles = wall_tiles;
        table.supplemental_tiles = supplemental_tiles;
        table.reward_indication_tiles = reward_indication_tiles;
        let mut players = table.players.borrow_mut();
        if let Some(ref mut players) = *players {
            for (tiles, seat) in player_tiles.iter() {
                let position = players.iter().position(|(_, seat2)| seat2 == seat).unwrap();
                let player = players.get_mut(position).unwrap();
                player.0.concealed_tiles = tiles.clone();
                player.0.exposed_melds = vec![];
                player.0.discarded_tiles = vec![];
            }
        }
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
    action_policy: Rc<Box<dyn ActionPolicy<C>>>,
    concealed_tiles: Vec<C::Tile>,
    exposed_melds: Vec<C::Meld>,
    discarded_tiles: Vec<C::Tile>,
    table: Weak<RefCell<TableContent<C>>>,
}

impl<C: Concept> Player<C> {
    fn new<'a>(table: Weak<RefCell<TableContent<C>>>, action_policy: Rc<Box<dyn ActionPolicy<C>>>) -> Player<C> {
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
    use crate::game::{Table, TileDealingSpec, Concept, DealtResult, ActionPolicy, Seat};
    use std::rc::Rc;

    struct MockConcept;

    impl Concept for MockConcept {
        type Tile = ();
        type Meld = ();
        type Action = ();
    }

    struct MockTileDealingSpec;

    impl TileDealingSpec<MockConcept> for MockTileDealingSpec {
        fn deal(&self) -> DealtResult<MockConcept> {
            DealtResult {
                wall_tiles: vec![(), ()],
                supplemental_tiles: vec![],
                reward_indication_tiles: vec![],
                player_tiles: [
                    (vec![], Seat::East),
                    (vec![], Seat::South),
                    (vec![], Seat::West),
                    (vec![], Seat::North),
                ],
            }
        }
    }

    struct MockActionPolicy;

    impl ActionPolicy<MockConcept> for MockActionPolicy {}

    #[test]
    fn a() {
        let tile_dealing_spec = {
            let spec: Box<dyn TileDealingSpec<MockConcept>> = Box::new(MockTileDealingSpec {});
            Rc::new(spec)
        };

        let mut table = Table::new(tile_dealing_spec);

        let action_policy = {
            let policy: Box<dyn ActionPolicy<MockConcept>> = Box::new(MockActionPolicy {});
            Rc::new(policy)
        };

        let mock_user_seeds = [
            (action_policy.clone(), Seat::East),
            (action_policy.clone(), Seat::South),
            (action_policy.clone(), Seat::West),
            (action_policy.clone(), Seat::North),
        ];

        table.join_users(mock_user_seeds);
        table.deal_tiles();
    }
}