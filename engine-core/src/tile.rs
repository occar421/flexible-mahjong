pub trait Suite: Ord + PartialOrd {}

pub trait Tile: Ord + PartialOrd {
    type Suite: Suite;
}
