use std::rc::{Rc, Weak};
use std::sync::RwLock;
use crate::game::table::TableContent;
use crate::game::def::{Concept, ActionPolicy};

pub(crate) struct Player<C: Concept> {
    point: i32,
    action_policy: Rc<Box<dyn ActionPolicy<C>>>,
    concealed_tiles: Vec<C::Tile>,
    exposed_melds: Vec<C::Meld>,
    discarded_tiles: Vec<C::Tile>,
    table: Weak<RwLock<TableContent<C>>>,
}

impl<C: Concept> Player<C> {
    pub fn new<'a>(table: Weak<RwLock<TableContent<C>>>, action_policy: Rc<Box<dyn ActionPolicy<C>>>) -> Player<C> {
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

    pub(crate) fn handle_draw(&self, drawn_tile: C::Tile) -> C::Action {
        let table = self.table.upgrade().unwrap();
        // let progress = table.read().unwrap().progress;
        self.action_policy.action_after_draw(drawn_tile)
    }
}
