use std::cmp::Ordering;

#[cfg(test)]
use {
    colored::*,
    std::fmt::{Debug, Error, Formatter},
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum Suite {
    /// Green, East, S
    /// 發、東、索子
    Green,
    /// Red, South, M
    /// 中、南、萬子
    Red,
    /// White, West, B
    /// 囗、西、貝子
    White,
    /// Black, North, P
    /// 治、北、筒子
    Black,
}

impl crate::tile::Suite for Suite {}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) enum Tile {
    /// 数牌
    Number(Suite, u8),
    /// 風牌
    Wind(Suite),
    /// 三元牌相当
    Symbol(Suite), // Dragon
}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Tile::Number(s1, n1) => match other {
                Tile::Number(s2, n2) => match s1.cmp(&s2) {
                    Ordering::Equal => n1.cmp(n2),
                    o => o
                },
                _ => Ordering::Less
            },
            Tile::Wind(s1) => match other {
                Tile::Number(_, _) => Ordering::Greater,
                Tile::Wind(s2) => s1.cmp(&s2),
                Tile::Symbol(_) => Ordering::Less
            },
            Tile::Symbol(s1) => match other {
                Tile::Symbol(s2) => s1.cmp(&s2),
                _ => Ordering::Greater
            }
        }
    }
}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl crate::tile::Tile for Tile { type Suite = Suite; }

#[cfg(test)]
impl Debug for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        const NUMBERS: [&str; 9] = ["一", "二", "三", "四", "伍", "六", "七", "八", "九"];
        const CORDS: [&str; 9] = ["１", "２", "３", "４", "５", "６", "７", "８", "９"];
        const SHELLS: [&str; 9] = ["１⃣", "２⃣", "３⃣", "４⃣", "５⃣", "６⃣", "７⃣", "８⃣", "９⃣"];
        const COINS: [&str; 9] = ["①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨"];

        write!(f, "{}", match self {
            Tile::Number(s, n) => match s {
                Suite::Red => format!("{}", NUMBERS[*n as usize - 1]).red(),
                Suite::Green => format!("{}", CORDS[*n as usize - 1]).green().underline(),
                Suite::White => format!("{}", SHELLS[*n as usize - 1]).yellow(),
                Suite::Black => format!("{}", COINS[*n as usize - 1]).magenta(),
            }
            ,
            Tile::Wind(s) => match s {
                Suite::Red => "東",
                Suite::Green => "南",
                Suite::White => "西",
                Suite::Black => "北",
            }.to_string().cyan(),
            Tile::Symbol(s) => match s {
                Suite::Red => "中".red(),
                Suite::Green => "發".green(),
                Suite::White => "　⃣".yellow(),
                Suite::Black => "治".magenta(),
            },
        })
    }
}
