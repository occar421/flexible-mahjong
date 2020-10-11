use crate::game::def::{
    Action, ActionPolicy, Concept, DealtResult, Seat, TileDealingSpec, PLAYERS_COUNT,
};
use crate::game::player::Player;
use arrayvec::ArrayVec;
use itertools::Itertools;
use std::cell::Cell;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

pub(crate) struct Table<C: Concept>(Rc<TableContent<C>>);

pub(crate) struct TableContent<C: Concept> {
    tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>,
    wall_tiles: RefCell<Vec<C::Tile>>,
    supplemental_tiles: RefCell<Vec<C::Tile>>,
    reward_indication_tiles: RefCell<Vec<C::Tile>>,
    progress: Cell<Progress>,
    participants: RefCell<Option<ArrayVec<[Participant<C>; PLAYERS_COUNT]>>>,
}

impl<C: Concept> Table<C> {
    pub fn new(tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>) -> Table<C> {
        Table(Rc::new(TableContent {
            tile_dealing_spec,
            wall_tiles: RefCell::new(vec![]),
            supplemental_tiles: RefCell::new(vec![]),
            reward_indication_tiles: RefCell::new(vec![]),
            progress: Cell::new(Progress::get_initial()),
            participants: RefCell::new(None),
        }))
    }

    pub fn join_users(&self, players: [(Rc<Box<dyn ActionPolicy<C>>>, Seat); PLAYERS_COUNT]) {
        let players = ArrayVec::from(players);

        {
            let groups = players.iter().group_by(|(_, s)| s);
            let a = groups.into_iter().collect_vec();
            if a.len() != PLAYERS_COUNT {
                panic!("Wrong arg `players`: seats should be unique")
            }
        }

        self.0.participants.replace(Some(
            players
                .iter()
                .map(|(policy, seat)| Participant {
                    player: Rc::new(Player::new(Rc::downgrade(&self.0.clone()), policy.clone())),
                    seat: *seat,
                })
                .sorted_by_key(|p| p.seat)
                .collect(),
        ));
    }
}

impl<C: Concept> TableContent<C> {
    pub(crate) fn player_at(&self, seat: Seat) -> Rc<Player<C>> {
        if let Some(ref participants) = *self.participants.borrow() {
            participants.get(usize::from(seat)).unwrap().player.clone()
        } else {
            panic!();
        }
    }

    pub(crate) fn start_game(&self, initial_point: i32) {
        {
            if let Some(ref participants) = *self.participants.borrow() {
                for participant in participants.iter() {
                    participant.player.set_initial_point(initial_point);
                }
            } else {
                panic!("Should call after join_users")
            }
        }

        self.progress.replace(Progress::get_initial());
    }

    pub(crate) fn do_hand(&self) {
        self.deal_tiles();

        let dealer: Seat = self.progress.get().current_hand.1.into();

        let mut turn = dealer;
        let result = loop {
            if self.wall_tiles.borrow().is_empty() {
                break HandResult::ExhaustiveDraw;
            } else {
                let action = {
                    if let Some(ref participants) = *self.participants.borrow() {
                        let turn = participants.get(usize::from(turn)).unwrap();
                        turn.player.draw()
                    } else {
                        panic!()
                    }
                };

                match action {
                    Action::Discard(tile) => {
                        // TODO 他家の鳴きなど
                        let used_in_meld = false;

                        if let Some(ref participants) = *self.participants.borrow() {
                            let turn = participants.get(usize::from(turn)).unwrap();
                            turn.player.append_to_discarded_tiles(tile, used_in_meld);
                        } else {
                            panic!()
                        };

                        turn = turn.next_seat();
                    }
                    // TODO action による分岐など
                    _ => unimplemented!(),
                }
            }
        };

        match result {
            HandResult::ExhaustiveDraw => {
                if let Some(ref participants) = *self.participants.borrow() {
                    let personal_results = participants
                        .iter()
                        .map(|p| (p.seat, p.player.check_hand_ready()))
                        .collect_vec();

                    let n_ready = personal_results.iter().filter(|(_, b)| *b).count();
                    let points_to_exchange = match n_ready {
                        0 | 4 => (0, 0),
                        1 => (3000, 1000),
                        2 => (1500, 1500),
                        3 => (1000, 3000),
                        _ => panic!(),
                    };

                    for (seat, ready) in personal_results.iter() {
                        let p = participants.get(usize::from(*seat)).unwrap();
                        if *ready {
                            p.player.gain_point(points_to_exchange.0);
                        } else {
                            p.player.lose_point(points_to_exchange.1);
                        }
                    }

                // TODO 流局処理残り
                } else {
                    panic!()
                }
            }
        }
    }

    fn deal_tiles(&self) {
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

        self.wall_tiles.replace(wall_tiles);
        self.supplemental_tiles.replace(supplemental_tiles);
        self.reward_indication_tiles
            .replace(reward_indication_tiles);
        if let Some(ref participants) = *self.participants.borrow() {
            for (i, (tiles, _)) in player_tiles.iter().sorted_by_key(|t| t.1).enumerate() {
                let participant = participants.get(i).unwrap();
                participant.player.accept_deal(tiles.clone());
            }
        }
    }

    pub(crate) fn pop_new_tile(&self) -> Option<C::Tile> {
        self.wall_tiles.borrow_mut().pop()
    }
}

impl<C: Concept> Deref for Table<C> {
    type Target = Rc<TableContent<C>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone)]
struct Progress {
    current_hand: (Round, usize),
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
    player: Rc<Player<C>>,
    seat: Seat,
}

#[derive(Copy, Clone)]
enum Round {
    East,
    South,
    West,
    North,
}

enum HandResult {
    ExhaustiveDraw,
}
