mod eight_pairs_and_half;
mod all_in_triplets;
mod sixteen_orphans;

pub(crate) use eight_pairs_and_half::EightPairsAndHalf;
pub(crate) use all_in_triplets::AllInTriplets;
pub(crate) use sixteen_orphans::SixteenOrphans;

use std::marker::PhantomData;

pub(crate) struct FanHand<T> {
    closed_han: u8,
    open_han: u8,
    phantom: PhantomData<T>,
}

pub(crate) struct YakumanHand<T> {
    config: T
}
