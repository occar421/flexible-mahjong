use crate::game::def::{Action, ActionPolicy, Concept};
use crate::game::table::TableContent;
use std::rc::{Rc, Weak};
use std::sync::RwLock;

pub(crate) struct Player<C: Concept> {
    point: i32,
    action_policy: Rc<Box<dyn ActionPolicy<C>>>,
    concealed_tiles: Vec<C::Tile>,
    exposed_melds: Vec<C::Meld>,
    discarded_tiles: Vec<(C::Tile, bool)>,
    table: Weak<RwLock<TableContent<C>>>,
}

impl<C: Concept> Player<C> {
    pub fn new<'a>(
        table: Weak<RwLock<TableContent<C>>>,
        action_policy: Rc<Box<dyn ActionPolicy<C>>>,
    ) -> Player<C> {
        Player {
            point: 0,
            action_policy,
            concealed_tiles: vec![],
            exposed_melds: vec![],
            discarded_tiles: vec![],
            table,
        }
    }

    pub(crate) fn set_initial_point(&mut self, point: i32) {
        self.point = point;
    }

    pub(crate) fn accept_deal(&mut self, tiles: Vec<C::Tile>) {
        self.concealed_tiles = tiles; // TODO 理牌？
        self.exposed_melds = vec![];
        self.discarded_tiles = vec![];
    }

    pub(crate) fn draw(&self) -> Action<C> {
        let table = self.table.upgrade().unwrap();
        table.read().unwrap();
        // TODO 卓の状況をチェックして Policy が action を決める
        // let progress = table.read().unwrap().progress;
        self.action_policy.action_after_draw(drawn_tile)
    }

    pub(crate) fn discard(&mut self, tile: C::Tile, used_in_meld: bool) {
        self.discarded_tiles.push((tile, used_in_meld));
    }
}