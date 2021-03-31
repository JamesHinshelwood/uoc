use std::{fmt::Debug, num::NonZeroI64};

use typenum::{NonZero, Unsigned};

use crate::dense::DenseMoney;

pub trait Currency: Sized + Copy + Clone + Debug + PartialEq + Eq {
    const SYMBOL: &'static str;
    type MinorUnits: Unsigned + NonZero;

    fn new(numer: i64, denom: NonZeroI64) -> DenseMoney<Self> {
        DenseMoney::new(numer, denom)
    }
}

macro_rules! currency {
    ($name:ident, $symbol:literal, $minor_units:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub struct $name;

        impl Currency for $name {
            const SYMBOL: &'static str = $symbol;
            type MinorUnits = typenum::$minor_units;
        }
    };
}

currency!(Gbp, "GBP", U100, "Pound sterling");
currency!(Mga, "MGA", U5, "Malagasy ariary");
currency!(Myr, "MYR", U100, "Malaysian ringgit");
currency!(Sgd, "SGD", U100, "Singapore dollar");
currency!(Usd, "USD", U100, "United States dollar");
