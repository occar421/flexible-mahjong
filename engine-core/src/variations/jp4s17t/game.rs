pub(crate) mod game_moderator;
pub(crate) mod player_broker;
pub(crate) mod player_hand;

const N_TILES: u8 = 9 * 4 * 4 + 4 * 4 + 4 * 4;

#[derive(Eq, PartialEq, Debug)]
pub(crate) enum WinningPoint {
    // 飜
    Fan(u8),
    // 役満 (could be double Yakuman or triple Yakuman etc.)
    Yakuman(u8),
}
