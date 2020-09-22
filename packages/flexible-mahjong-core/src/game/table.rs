use std::rc::Rc;
use std::sync::RwLock;
use std::cell::RefCell;
use arrayvec::ArrayVec;
use crate::game::def::{PLAYERS_COUNT, Seat, DealtResult, Concept, ActionPolicy, TileDealingSpec};
use itertools::Itertools;
use std::ops::{Deref, DerefMut};
use crate::game::player::Player;

pub(crate) struct Table<C: Concept>(Rc<RwLock<TableContent<C>>>);

pub(crate) struct TableContent<C: Concept> {
    tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>,
    wall_tiles: Vec<C::Tile>,
    supplemental_tiles: Vec<C::Tile>,
    reward_indication_tiles: Vec<C::Tile>,
    progress: Progress,
    participants: RefCell<Option<ArrayVec<[Participant<C>; PLAYERS_COUNT]>>>,
}

impl<C: Concept> Table<C> {
    pub fn new(tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>) -> Table<C> {
        Table(Rc::new(RwLock::new(
            TableContent {
                tile_dealing_spec,
                wall_tiles: vec![],
                supplemental_tiles: vec![],
                reward_indication_tiles: vec![],
                progress: Progress::get_initial(),
                participants: RefCell::new(None),
            })))
    }

    pub(crate) fn join_users(&mut self, players: [(Rc<Box<dyn ActionPolicy<C>>>, Seat); PLAYERS_COUNT]) {
        let players = ArrayVec::from(players);

        {
            let groups = players.iter().group_by(|(_, s)| s);
            let a = groups.into_iter().collect_vec();
            if a.len() != PLAYERS_COUNT {
                panic!("Wrong arg `players`: seats should be unique")
            }
        }

        self.write().unwrap().participants.replace(
            Some(players.iter()
                .map(|(policy, seat)|
                    Participant { player: Player::new(Rc::downgrade(&self.clone()), policy.clone()), seat: *seat }
                )
                .sorted_by_key(|p| p.seat)
                .collect()
            )
        );
    }

    pub(crate) fn start_game(&mut self, initial_point: i32) {
        {
            let table = self.read().unwrap();
            let mut participants = table.participants.borrow_mut();
            if let Some(ref mut participants) = *participants {
                for participant in participants.iter_mut() {
                    participant.player.set_initial_point(initial_point);
                }
            } else {
                panic!("Should call after join_users")
            }
        }

        self.write().unwrap().progress = Progress::get_initial();
    }

    pub(crate) fn do_hand(&mut self) {
        let starter: Seat = self.read().unwrap().progress.current_hand.1.into();

        self.deal_tiles();

        let mut turn = starter;

        let result = loop {
            let pop_result = { self.write().unwrap().wall_tiles.pop() };
            if let Some(tile) = pop_result {
                {
                    let table = self.read().unwrap();
                    let participants = table.participants.borrow();
                    let turn =
                        if let Some(ref participants) = *participants {
                            let turn: u8 = turn.into();
                            participants.get(turn as usize).unwrap()
                        } else {
                            panic!()
                        };

                    let action = turn.player.handle_draw(tile);
                    // unimplemented!()
                }
            } else {
                break 1; // TODO
            }
        };
    }

    fn deal_tiles(&mut self) {
        {
            if self.read().unwrap().participants.borrow().is_none() {
                panic!("Should call after join_users")
            }
        }

        let DealtResult { wall_tiles, supplemental_tiles, reward_indication_tiles, player_tiles } = self.read().unwrap().tile_dealing_spec.deal();

        {
            let groups = player_tiles.iter().group_by(|(_, s)| s);
            let a = groups.into_iter().collect_vec();
            if a.len() != PLAYERS_COUNT {
                panic!("Wrong arg `player_tiles`: seats should be unique")
            }
        }

        let mut table = self.write().unwrap();
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
    type Target = Rc<RwLock<TableContent<C>>>;

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