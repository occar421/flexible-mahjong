use self::super::Player;

pub struct InteractivePlayer {}

impl InteractivePlayer {
    pub fn new() -> InteractivePlayer {
        return InteractivePlayer {};
    }
}

impl Player for InteractivePlayer {}