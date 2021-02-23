use crate::game::def::{Action, Concept, DealtResult, SeatOld, TileDealingSpec, PLAYERS_COUNT};
use crate::game::player::PlayerOld;
use arrayvec::ArrayVec;
use itertools::Itertools;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use thiserror::Error;

struct WaitingTable<C: Concept> {
    concept: PhantomData<C>,
    id: TableId,
    participants: Participants,
}

impl<C: Concept> WaitingTable<C> {
    fn setup() -> Self {
        Self {
            id: TableId::generate(),
            participants: Participants::nobody(),
            concept: PhantomData,
        }
    }

    fn accept_participant(self, new_participant: ParticipantId) -> Result<Self, TableError> {
        Ok(Self {
            participants: self.participants.receive(new_participant)?,
            ..self
        })
    }

    fn be_ready(self) -> Option<ReadyTable<C>> {
        self.participants
            .gathered()
            .then(|| ReadyTable::<C>::setup(self.id, self.participants))
    }
}

struct ReadyTable<C: Concept> {
    concept: PhantomData<C>,
    id: TableId,
    participants: Participants,
    seating_list: Option<SeatingList>,
}

impl<C: Concept> ReadyTable<C> {
    fn setup(id: TableId, participants: Participants) -> Self {
        Self {
            id,
            participants,
            seating_list: None,
            concept: PhantomData,
        }
    }

    /// TODO accepts seating_spec
    fn arrange_initial_seating(self) -> Self {
        unimplemented!()
    }

    fn start_game(self) -> Option<HandPreparingTable<C>> {
        if let Some(seating_list) = self.seating_list {
            Some(HandPreparingTable::<C>::new(TableInfo::new(
                self.id,
                self.participants,
                seating_list,
            )))
        } else {
            None
        }
    }
}

// TODO name
struct TableInfo<C: Concept> {
    concept: PhantomData<C>,
    id: TableId,
    players: Players,
    seating_list: SeatingList,
}

static INITIAL_POINT: u16 = 25000; // TODO const generics

impl<C: Concept> TableInfo<C> {
    fn new(id: TableId, participants: Participants, seating_list: SeatingList) -> Self {
        Self {
            concept: PhantomData,
            id,
            players: Players::form(participants, INITIAL_POINT),
            seating_list,
        }
    }
}

// Round = 場
// Hand  = 局

struct HandPreparingTable<C: Concept> {
    table_info: TableInfo<C>,
    progress: Progress, // TODO WIP
}

impl<C: Concept> HandPreparingTable<C> {
    fn new(table_info: TableInfo<C>) -> Self {
        Self {
            table_info,
            progress: Progress::get_initial(),
        }
    }

    // TODO accepts dealing_spec
    fn deal(self) -> HandPlayingTable<C> {
        unimplemented!()
    }
}

struct HandPlayingTable<C: Concept> {
    table_info: TableInfo<C>,
    progress: Progress, // TODO WIP
    turn: Turn,
    wall_tiles: WallTiles<C>,
    supplemental_tiles: SupplementalTiles<C>,
    reward_indication_tiles: RewardIndicationTiles<C>,
    players_hands: PlayersHands<C>,
    players_discards: PlayersDiscards<C>,
}

impl<C: Concept> HandPlayingTable<C> {
    // TODO name
    fn something_new(
        table_info: TableInfo<C>,
        progress: Progress,
        wall_tiles: WallTiles<C>,
        supplemental_tiles: SupplementalTiles<C>,
        reward_indication_tiles: RewardIndicationTiles<C>,
        players_hands: PlayersHands<C>,
    ) -> Self {
        Self {
            table_info,
            progress,
            turn: Turn::get_initial(),
            wall_tiles,
            supplemental_tiles,
            reward_indication_tiles,
            players_hands,
            players_discards: PlayersDiscards::empty(),
        }
    }

    // TODO care Error
    fn draw_tile_by(self, participant_id: ParticipantId) -> Result<Self, TableError> {
        let seat = self
            .table_info
            .seating_list
            .get_seat_of(participant_id)
            .ok_or(TableError::UnknownParticipantError)?;
        if !self.turn.is_turn_of(seat) {
            Err(TableError::NotParticipantsTurnError)?;
        }

        let (wall_tiles, _) = self
            .wall_tiles
            .pick()
            .ok_or(TableError::ExhaustedWallError)?;
        Ok(Self { wall_tiles, ..self }) // FIXME players tile
    }
}

// TODO メソッド毎に細かく分ける
#[derive(Error, Debug)]
enum TableError {
    #[error("")] // TODO
    ParticipantsExceededError(u8),
    #[error("")] // TODO
    UnknownParticipantError,
    #[error("")] // TODO
    NotParticipantsTurnError,
    #[error("")] // TODO
    ExhaustedWallError,
}

struct TableId(uuid::Uuid);

impl TableId {
    fn generate() -> TableId {
        TableId(uuid::Uuid::new_v4())
    }
}

struct Turn(Seat);

impl Turn {
    fn get_initial() -> Self {
        Self(Seat::East)
    }

    fn is_turn_of(&self, seat: Seat) -> bool {
        self.0 == seat
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Seat {
    East,
    South,
    West,
    North,
}

struct WallTiles<C: Concept>(Vec<C::Tile>);

impl<C: Concept> WallTiles<C> {
    fn pick(self) -> Option<(Self, C::Tile)> {
        let mut this = self;
        this.0.pop().map(|tile| (Self(this.0), tile))
    }
}

struct SupplementalTiles<C: Concept>(Vec<C::Tile>);

struct RewardIndicationTiles<C: Concept>(Vec<C::Tile>);

// TODO with consist of Closed Hand, Melds
// TODO with condition （海底、槍槓）
struct PlayerHand<C: Concept>(Vec<C::Tile>);

struct PlayersHands<C: Concept>(HashMap<ParticipantId, PlayerHand<C>>);

struct Discards<C: Concept>(Vec<C::Tile>);

struct PlayersDiscards<C: Concept>(HashMap<ParticipantId, Discards<C>>);

impl<C: Concept> PlayersDiscards<C> {
    fn empty() -> Self {
        Self(HashMap::new())
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct ParticipantId(uuid::Uuid); // TODO make this external

struct Participants(Vec<ParticipantId>);

static MAX_PARTICIPANT: u8 = 4; // TODO const generics

impl Participants {
    fn nobody() -> Self {
        Self(vec![])
    }

    fn receive(self, new_participant: ParticipantId) -> Result<Self, TableError> {
        (self.0.len() >= MAX_PARTICIPANT as usize)
            .then(|| Self([self.0, vec![new_participant]].concat()))
            .ok_or(TableError::ParticipantsExceededError(MAX_PARTICIPANT).into())
    }

    fn send_off(self, a_participant: ParticipantId) -> Result<Self, TableError> {
        unimplemented!()
    }

    fn gathered(&self) -> bool {
        // TODO 三麻
        self.0.len() == MAX_PARTICIPANT as usize
    }
}

struct Player {
    id: ParticipantId,
    point: u16, // TODO use Point VO
}

impl Player {
    // TODO name
    fn something_new(id: ParticipantId, initial_point: u16) -> Self {
        Self {
            id,
            point: initial_point,
        }
    }
}

struct Players(Vec<Player>);

impl Players {
    // TODO name
    fn form(participants: Participants, initial_point: u16) -> Self {
        // TODO refactor
        Self(
            participants
                .0
                .iter()
                .map(|p_id| Player::something_new(p_id.clone(), initial_point))
                .collect(),
        )
    }
}

#[derive(Clone)]
struct SeatingList(HashMap<ParticipantId, Seat>);

impl SeatingList {
    fn get_seat_of(&self, participant_id: ParticipantId) -> Option<Seat> {
        self.0.get(&participant_id).map(|s| *s)
    }
}

/**
 * 以上、作り直し部分のおわり。
 */

pub(crate) struct TableContent<C: Concept> {
    tile_dealing_spec: Rc<Box<dyn TileDealingSpec<C>>>,
    wall_tiles: RefCell<Vec<C::Tile>>,
    supplemental_tiles: RefCell<Vec<C::Tile>>,
    reward_indication_tiles: RefCell<Vec<C::Tile>>,
    progress: Cell<Progress>,
    participants: RefCell<Option<ArrayVec<[ParticipantOld<C>; PLAYERS_COUNT]>>>,
}

impl<C: Concept> TableContent<C> {
    pub(crate) fn player_at(&self, seat: SeatOld) -> Rc<PlayerOld<C>> {
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

        let dealer: SeatOld = self.progress.get().current_hand.1.into();

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
    player: Rc<PlayerOld<C>>,
    seat: SeatOld,
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
