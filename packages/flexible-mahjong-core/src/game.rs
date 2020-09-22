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

pub struct DealtResult<C: Concept> {
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
    participants: RefCell<Option<ArrayVec<[Participant<C>; PLAYERS_COUNT]>>>,
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
                participants: RefCell::new(None),
            })))
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

        self.borrow_mut().participants.replace(
            Some(players.iter()
                .map(|(policy, seat)|
                    Participant { player: Player::new(Rc::downgrade(&self.clone()), policy.clone()), seat: *seat }
                )
                .sorted_by_key(|p| p.seat)
                .collect()
            )
        );
    }

    fn start_game(&mut self, initial_point: i32) {
        {
            let table = self.borrow();
            let mut participants = table.participants.borrow_mut();
            if let Some(ref mut participants) = *participants {
                for participant in participants.iter_mut() {
                    participant.player.set_initial_point(initial_point);
                }
            } else {
                panic!("Should call after join_users")
            }
        }

        self.borrow_mut().progress = Progress::get_initial();
    }

    fn do_hand(&mut self) {
        let starter: Seat = self.borrow().progress.current_hand.1.into();

        self.deal_tiles();

        let mut turn = starter;
        while let Some(tile) = self.borrow_mut().wall_tiles.pop() {
            let table = self.borrow();
            let participants = table.participants.borrow();
            let turn =
            if let Some(ref participants) = *participants {
                let turn: u8 = turn.into();
                participants.get(turn as usize).unwrap()
            } else {
                panic!()
            };

            let action = turn.player.handle_draw(tile);
            unimplemented!()

        }

        // loop {
        //     if let Some(tile) = { self.borrow_mut().wall_tiles.pop() } {
        //         let participants = self.borrow().bo
        //     } else { break; }
        // }
    }

    fn deal_tiles(&mut self) {
        {
            if self.borrow().participants.borrow().is_none() {
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
        let mut participants = table.participants.borrow_mut();
        if let Some(ref mut participants) = *participants {
            for (i, (tiles, _)) in player_tiles.iter().sorted_by_key(|t| t.1).enumerate() {
                let participant = participants.get_mut(i).unwrap();
                participant.player.accept_deal(tiles.clone());
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

struct Participant<C: Concept> {
    player: Player<C>,
    seat: Seat,
}

#[derive(Copy, Clone)]
enum Round {
    East,
    South,
    West,
    North,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Seat {
    East,
    South,
    West,
    North,
}

impl From<u8> for Seat {
    fn from(value: u8) -> Self {
        use Seat::*;

        match value {
            0 => East,
            1 => South,
            2 => West,
            3 => North,
            _ => panic!(format!("Invalid value: {}", value))
        }
    }
}

impl From<Seat> for u8 {
    fn from(seat: Seat) -> Self {
        use Seat::*;

        match seat {
            East => 0,
            South => 1,
            West => 2,
            North => 3,
        }
    }
}

pub trait ActionPolicy<C: Concept> {
    fn action_after_draw(&self, drawn_tile: C::Tile) -> C::Action;
}

struct Player<C: Concept> {
    point: i32,
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

    fn set_initial_point(&mut self, point: i32) {
        self.point = point;
    }

    fn accept_deal(&mut self, tiles: Vec<C::Tile>) {
        self.concealed_tiles = tiles; // TODO 理牌？
        self.exposed_melds = vec![];
        self.discarded_tiles = vec![];
    }

    fn handle_draw(&self, drawn_tile: C::Tile) -> C::Action {
        let table = self.table.upgrade().unwrap();
        let progress = table.borrow().progress;
        self.action_policy.action_after_draw(drawn_tile);
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use crate::game::{Table, TileDealingSpec, Concept, DealtResult, ActionPolicy, Seat};
    use std::rc::Rc;

    struct MockConcept;

    impl Concept for MockConcept {
        type Tile = char;
        type Meld = ();
        type Action = ();
    }

    struct MockTileDealingSpec;

    impl TileDealingSpec<MockConcept> for MockTileDealingSpec {
        fn deal(&self) -> DealtResult<MockConcept> {
            DealtResult {
                wall_tiles: vec!['a', 'b'],
                supplemental_tiles: vec!['c'],
                reward_indication_tiles: vec!['d'],
                player_tiles: [
                    (vec!['e'], Seat::East),
                    (vec!['f'], Seat::South),
                    (vec!['g'], Seat::West),
                    (vec!['h'], Seat::North),
                ],
            }
        }
    }

    struct MockActionPolicy;

    impl ActionPolicy<MockConcept> for MockActionPolicy {
        fn action_after_draw(&self, drawn_tile: char) -> () {
            unimplemented!()
        }
    }

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
        table.start_game(1000);
        table.do_hand();
    }
}