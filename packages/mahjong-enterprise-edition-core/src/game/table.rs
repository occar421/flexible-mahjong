use crate::game::def::{
    Action, ActionPolicy, Concept, DealtResult, Seat, TileDealingSpec, PLAYERS_COUNT,
};
use crate::game::player::Player;
use arrayvec::ArrayVec;
use itertools::Itertools;
use std::cell::Cell;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
enum TableError {
    ParticipantsExceededError(u8),
    UnknownParticipantError,
}

impl Display for TableError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ParticipantsExceededError(limit) => {
                write!(f, "ParticipantsExceededError: {}", limit)
            }
            Self::UnknownParticipantError => {
                write!(f, "UnknownParticipantError")
            }
        }
    }
}

impl Error for TableError {}

struct TableId(String); // TODO WIP uuid

impl TableId {
    fn generate() -> TableId {
        TableId("foo".to_string())
    }
}

struct WallTiles<C: Concept>(Vec<C::Tile>);

impl<C: Concept> WallTiles<C> {
    fn empty() -> Self {
        Self(vec![])
    }
}

struct SupplementalTiles<C: Concept>(Vec<C::Tile>);

impl<C: Concept> SupplementalTiles<C> {
    fn empty() -> Self {
        Self(vec![])
    }
}

struct RewardIndicationTiles<C: Concept>(Vec<C::Tile>);

impl<C: Concept> RewardIndicationTiles<C> {
    fn empty() -> Self {
        Self(vec![])
    }
}

#[derive(Clone)]
struct ParticipantId(String); // TODO WIP uuid and external

struct Participants(Vec<ParticipantId>);

static MAX_PARTICIPANT: u8 = 4; // TODO const generics

impl Participants {
    fn nobody() -> Self {
        Self(vec![])
    }

    fn receive(self, new_participant: ParticipantId) -> Result<Self, Box<dyn Error>> {
        (self.0.len() >= MAX_PARTICIPANT as usize)
            .then(|| Self([self.0, vec![new_participant]].concat()))
            .ok_or(TableError::ParticipantsExceededError(MAX_PARTICIPANT).into())
    }

    fn send_off(self, a_participant: ParticipantId) -> Result<Self, Box<dyn Error>> {
        unimplemented!()
    }
}

struct SeatingList(); // TODO map between ParticipantId and Seat

impl SeatingList {
    fn empty() -> Self {
        Self()
    }
}

pub(crate) struct Table<C: Concept> {
    id: TableId,
    wall_tiles: WallTiles<C>,
    supplemental_tiles: SupplementalTiles<C>,
    reward_indication_tiles: RewardIndicationTiles<C>,
    progress: Progress, // TODO WIP
    participants: Participants,
    seating_list: SeatingList,
}

impl<C: Concept> Table<C> {
    fn setup() -> Table<C> {
        Table {
            id: TableId::generate(),
            wall_tiles: WallTiles::empty(),
            supplemental_tiles: SupplementalTiles::empty(),
            reward_indication_tiles: RewardIndicationTiles::empty(),
            progress: Progress::initial(),
            participants: Participants::nobody(),
            seating_list: SeatingList::empty(),
        }
    }

    fn accept_participant(self, new_participant: ParticipantId) -> Result<Self, Box<dyn Error>> {
        Ok(Table {
            participants: self.participants.receive(new_participant)?,
            ..self
        })
    }

    fn decide_initial_seating_according_to(self, seating_spec: ()) -> Self {
        unimplemented!()
    }
}

/**
 * 以上、作り直し部分のおわり。
 */

pub(crate) struct TableOld<C: Concept>(Rc<TableContent<C>>);

pub(crate) struct TableContent<C: Concept> {
    tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>,
    wall_tiles: RefCell<Vec<C::Tile>>,
    supplemental_tiles: RefCell<Vec<C::Tile>>,
    reward_indication_tiles: RefCell<Vec<C::Tile>>,
    progress: Cell<Progress>,
    participants: RefCell<Option<ArrayVec<[ParticipantOld<C>; PLAYERS_COUNT]>>>,
}

impl<C: Concept> TableOld<C> {
    pub fn new(tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>) -> TableOld<C> {
        TableOld(Rc::new(TableContent {
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
                .map(|(policy, seat)| ParticipantOld {
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

impl<C: Concept> Deref for TableOld<C> {
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
    pub fn initial() -> Progress {
        Progress {
            current_hand: (Round::East, 1),
            deals_count: 0,
        }
    }

    pub fn get_initial() -> Progress {
        Progress {
            current_hand: (Round::East, 1),
            deals_count: 0,
        }
    }
}

struct ParticipantOld<C: Concept> {
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
