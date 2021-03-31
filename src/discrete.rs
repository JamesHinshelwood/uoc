use std::{marker::PhantomData, num::NonZeroI64};

use bigdecimal::BigDecimal;
use rust_decimal::Decimal;
use typenum::Unsigned;

use crate::{currency::Currency, dense::DenseMoney};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DiscreteMoney<C: Currency> {
    amount: u32,
    phantom: PhantomData<C>,
}

impl<C: Currency> DiscreteMoney<C> {
    pub fn new(amount: u32) -> DiscreteMoney<C> {
        DiscreteMoney {
            amount,
            phantom: PhantomData,
        }
    }

    pub fn to_dense(self) -> DenseMoney<C> {
        self.into()
    }
}

impl<C: Currency> From<DiscreteMoney<C>> for DenseMoney<C> {
    fn from(discrete: DiscreteMoney<C>) -> Self {
        let numer: i64 = discrete.amount.into();
        // Safety: The `NonZero` bound on `C::MinorUnits` guarantees `C::MinorUnits::to_i64() != 0`.
        let denom = unsafe { NonZeroI64::new_unchecked(C::MinorUnits::to_i64()) };
        C::new(numer, denom)
    }
}

impl<C: Currency> From<DiscreteMoney<C>> for Decimal {
    fn from(discrete: DiscreteMoney<C>) -> Self {
        let minor_units = C::MinorUnits::to_u32();
        if minor_units % 10 != 0 {
            todo!("the maths is hard and I'm feeling lazy :)")
        }
        let scale = (minor_units as f32).log10() as u32; // FIXME: Probably a rounding bug here.
        Decimal::new(discrete.amount.into(), scale)
    }
}

impl<C: Currency> From<DiscreteMoney<C>> for BigDecimal {
    fn from(discrete: DiscreteMoney<C>) -> Self {
        let minor_units = C::MinorUnits::to_i64();
        if minor_units % 10 != 0 {
            todo!("the maths is hard and I'm feeling lazy :)")
        }
        let scale = (minor_units as f32).log10() as i64; // FIXME: Probably a rounding bug here.
        BigDecimal::new(discrete.amount.into(), scale)
    }
}
