use std::ops::{Add, Neg};

pub trait AddToLimit: Add<Output=Self> + PartialOrd + Copy + Sized {
    fn add_to_limit(&mut self, b: &Self, l: &Self) {
        let attempt: Self = *self + *b;
        if &attempt > l {
            *self = *l;
        } else {
            *self = attempt;
        }
    }

    fn add_to_max_limit(&mut self, b: &Self, l: &Self)
        where Self: Neg<Output=Self> + Default {
        let attempt: Self = *self + *b;
        if attempt > Self::default() {
            if &attempt > l {
                *self = *l;
            } else {
                *self = attempt;
            }
        } else {
            if attempt < l.neg() {
                *self = l.neg();
            } else {
                *self = attempt;
            }
        }
    }
}


impl<N> AddToLimit for N
    where N: Add<Output=Self> + PartialOrd + Copy + Sized {}
