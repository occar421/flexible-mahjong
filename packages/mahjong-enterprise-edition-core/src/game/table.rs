use crate::game::def::{
    Action, ActionPolicy, Concept, DealtResult, Seat, TileDealingSpec, PLAYERS_COUNT,
};
use crate::game::player::Player;
use arrayvec::ArrayVec;
use itertools::Itertools;
use std::cell::Cell;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub(crate) struct Table<C: Concept>(Rc<RefCell<TableContent<C>>>);

pub(crate) struct TableContent<C: Concept> {
    tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>,
    wall_tiles: RefCell<Vec<C::Tile>>,
    supplemental_tiles: RefCell<Vec<C::Tile>>,
    reward_indication_tiles: RefCell<Vec<C::Tile>>,
    progress: Cell<Progress>,
    participants: RefCell<Option<ArrayVec<[Participant<C>; PLAYERS_COUNT]>>>,
}

impl<C: Concept> Table<C> {
    pub(crate) fn new(tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>) -> Table<C> {
        Table(Rc::new(RefCell::new(TableContent {
            tile_dealing_spec,
            wall_tiles: RefCell::new(vec![]),
            supplemental_tiles: RefCell::new(vec![]),
            reward_indication_tiles: RefCell::new(vec![]),
            progress: Cell::new(Progress::get_initial()),
            participants: RefCell::new(None),
        })))
    }

    pub(crate) fn join_users(
        &mut self,
        players: [(Rc<Box<dyn ActionPolicy<C>>>, Seat); PLAYERS_COUNT],
    ) {
        let players = ArrayVec::from(players);

        {
            let groups = players.iter().group_by(|(_, s)| s);
            let a = groups.into_iter().collect_vec();
            if a.len() != PLAYERS_COUNT {
                panic!("Wrong arg `players`: seats should be unique")
            }
        }

        self.participants.replace(Some(
            players
                .iter()
                .map(|(policy, seat)| Participant {
                    player: Player::new(Rc::downgrade(&self.0.clone()), policy.clone()),
                    seat: *seat,
                })
                .sorted_by_key(|p| p.seat)
                .collect(),
        ));
    }
}

impl<C: Concept> TableContent<C> {
    pub(crate) fn start_game(&mut self, initial_point: i32) {
        {
            let mut participants = self.participants.borrow_mut();
            if let Some(ref mut participants) = *participants {
                for participant in participants.iter_mut() {
                    participant.player.set_initial_point(initial_point);
                }
            } else {
                panic!("Should call after join_users")
            }
        }

        self.progress.replace(Progress::get_initial());
    }

    pub(crate) fn do_hand(&mut self) {
        let starter: Seat = self.progress.get().current_hand.1.into();

        self.deal_tiles();

        let mut turn = starter;

        let result = loop {
            {
                let action = {
                    let participants = self.participants.borrow();
                    if let Some(ref participants) = *participants {
                        let turn: u8 = turn.into();
                        let turn = participants.get(turn as usize).unwrap();
                        turn.player.draw()
                    } else {
                        panic!()
                    }
                };

                match action {
                    Action::Discard(tile) => {
                        // TODO 他家の鳴きなど

                        if let Some(ref mut participants) = *self.participants.borrow_mut() {
                            let turn: u8 = turn.into();
                            let turn = participants.get_mut(turn as usize).unwrap();
                            turn.player.discard(tile, false);
                        } else {
                            panic!()
                        };
                    }
                    _ => unimplemented!(),
                }
                // TODO action による分岐など
            }
        };
    }

    pub(crate) fn provide_new_tile(&mut self) {
        self.provide_new_tile();
    }

    fn deal_tiles(&mut self) {
        {
            if self.participants.borrow().is_none() {
                panic!("Should call after join_users")
            }
        }

        let DealtResult {
            wall_tiles,
            supplemental_tiles,
            reward_indication_tiles,
            player_tiles,
        } = self.tile_dealing_spec.deal();

        {
            let groups = player_tiles.iter().group_by(|(_, s)| s);
            let a = groups.into_iter().collect_vec();
            if a.len() != PLAYERS_COUNT {
                panic!("Wrong arg `player_tiles`: seats should be unique")
            }
        }

        let table = self;
        table.wall_tiles.replace(wall_tiles);
        table.supplemental_tiles.replace(supplemental_tiles);
        table
            .reward_indication_tiles
            .replace(reward_indication_tiles);
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
    type Target = TableContent<C>;

    fn deref(&self) -> &Self::Target {
        &*self.0.borrow()
    }
}

impl<C: Concept> DerefMut for Table<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.borrow()
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
            _ => panic!(format!("Invalid value: {}", value)),
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
