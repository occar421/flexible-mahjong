use crate::game::def::{Action, ActionPolicy, Concept};
use crate::game::table::TableContent;
use std::cell::{Cell, Ref, RefCell};
use std::rc::{Rc, Weak};

pub(crate) struct Player<C: Concept> {
    point: Cell<i32>,
    action_policy: Rc<Box<dyn ActionPolicy<C>>>,
    concealed_tiles: RefCell<Vec<C::Tile>>,
    exposed_melds: RefCell<Vec<C::Meld>>,
    discarded_tiles: RefCell<Vec<(C::Tile, bool)>>,
    table: Weak<TableContent<C>>,
}

impl<C: Concept> Player<C> {
    pub fn new(
        table: Weak<TableContent<C>>,
        action_policy: Rc<Box<dyn ActionPolicy<C>>>,
    ) -> Player<C> {
        Player {
            point: Cell::new(0),
            action_policy,
            concealed_tiles: RefCell::new(vec![]),
            exposed_melds: RefCell::new(vec![]),
            discarded_tiles: RefCell::new(vec![]),
            table,
        }
    }

    pub(crate) fn point(&self) -> i32 {
        self.point.get()
    }

    pub(crate) fn exposed_melds(&self) -> Ref<Vec<C::Meld>> {
        self.exposed_melds.borrow()
    }

    pub(crate) fn discarded_tiles(&self) -> Ref<Vec<(C::Tile, bool)>> {
        self.discarded_tiles.borrow()
    }

    pub(crate) fn set_initial_point(&self, point: i32) {
        self.point.replace(point);
    }

    pub(crate) fn gain_point(&self, point: i32) {
        self.point.replace(self.point.get() + point);
    }

    pub(crate) fn lose_point(&self, point: i32) {
        self.point.replace(self.point.get() - point);
    }

    pub(crate) fn accept_deal(&self, tiles: Vec<C::Tile>) {
        self.concealed_tiles.replace(tiles); // TODO 理牌？
        self.exposed_melds.replace(vec![]);
        self.discarded_tiles.replace(vec![]);
    }

    pub(crate) fn draw(&self) -> Action<C> {
        let table = self.table.upgrade().unwrap();
        let drawn_tile = table.pop_new_tile().unwrap();
        self.action_policy.action_after_draw(drawn_tile)
    }

    pub(crate) fn append_to_discarded_tiles(&self, tile: C::Tile, used_in_meld: bool) {
        self.discarded_tiles.borrow_mut().push((tile, used_in_meld));
    }

    pub(crate) fn check_hand_ready(&self) -> bool {
        false
    }
}
