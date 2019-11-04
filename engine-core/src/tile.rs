use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Suite {
    /// Red, East, M
    /// 中、東、萬子
    Red,
    /// Green, South, S
    /// 發、南、索子
    Green,
    /// White, West, P
    /// 白、西、筒子
    White,
    /// Black, North, ?
    /// 黒、北、？子
    Black,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Tile {
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
