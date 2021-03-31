use std::{fmt::Display, marker::PhantomData, num::NonZeroI64};

use eyre::{eyre, Result};
use num_rational::Rational64;
use num_traits::{CheckedAdd, CheckedSub, FromPrimitive, ToPrimitive, Zero};
use postgres_types::{private::BytesMut, FromSql, ToSql};
use serde::{
    de::{self, Unexpected},
    Deserialize, Serialize,
};
use typenum::Unsigned;

use crate::{currency::Currency, discrete::DiscreteMoney};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DenseMoney<C: Currency> {
    amount: Rational64,
    phantom: PhantomData<C>,
}

impl<C: Currency> DenseMoney<C> {
    pub fn new(numer: i64, denom: NonZeroI64) -> Self {
        DenseMoney {
            amount: Rational64::new(numer, denom.get()),
            phantom: PhantomData,
        }
    }

    pub fn numer(&self) -> i64 {
        *self.amount.numer()
    }

    pub fn denom(&self) -> i64 {
        *self.amount.denom()
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.amount
            .checked_add(&rhs.amount)
            .map(|amount| DenseMoney {
                amount,
                phantom: PhantomData,
            })
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.amount
            .checked_sub(&rhs.amount)
            .map(|amount| DenseMoney {
                amount,
                phantom: PhantomData,
            })
    }

    pub fn round(&self) -> (DiscreteMoney<C>, DenseMoney<C>) {
        let scale = C::MinorUnits::to_i64();
        let approx = (self.amount * scale).round().to_u32().unwrap();
        let approx_r = Rational64::from_u32(approx).unwrap() / scale;

        (
            DiscreteMoney::new(approx),
            DenseMoney {
                amount: self.amount - approx_r,
                phantom: PhantomData,
            },
        )
    }

    pub fn round_exact(&self) -> Result<DiscreteMoney<C>> {
        let (rounded, remainder) = self.round();
        if !remainder.amount.is_zero() {
            return Err(eyre!(
                "tried to round dense money exactly, but {} was left over",
                remainder
            ));
        }

        Ok(rounded)
    }
}

impl<C: Currency> PartialOrd for DenseMoney<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.amount.partial_cmp(&other.amount)
    }
}

impl<C: Currency> Ord for DenseMoney<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.amount.cmp(&other.amount)
    }
}

impl<C: Currency> Display for DenseMoney<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", C::SYMBOL, self.amount.to_f64().unwrap())
    }
}

impl<C: Currency> ToSql for DenseMoney<C> {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        <Vec<&str> as ToSql>::to_sql(
            &vec![
                &self.numer().to_string(),
                &self.denom().to_string(),
                C::SYMBOL,
            ],
            ty,
            out,
        )
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <Vec<&str> as ToSql>::accepts(ty)
    }

    postgres_types::to_sql_checked!();
}

impl<'a, C: Currency> FromSql<'a> for DenseMoney<C> {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let v = <Vec<&str> as FromSql>::from_sql(ty, raw)?;
        let numer: i64 = v
            .get(0)
            .ok_or_else(|| eyre!("invalid money array from DB"))?
            .parse()?;
        let denom: i64 = v
            .get(1)
            .ok_or_else(|| eyre!("invalid money array from DB"))?
            .parse()?;
        let currency = *v
            .get(2)
            .ok_or_else(|| eyre!("invalid money array from DB"))?;

        if currency != C::SYMBOL {
            return Err(eyre!("invalid currency {}, expected {}", currency, C::SYMBOL).into());
        }

        Ok(DenseMoney::new(
            numer,
            NonZeroI64::new(denom).ok_or_else(|| eyre!("zero denom returned from DB"))?,
        ))
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        <Vec<&str> as FromSql>::accepts(ty)
    }
}

#[derive(Serialize, Deserialize)]
struct DenseMoneyDto<'a> {
    amount: AmountDto,
    currency: &'a str,
}

#[derive(Serialize, Deserialize)]
struct AmountDto {
    numer: String,
    denom: String,
}

impl<C: Currency> Serialize for DenseMoney<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let dto = DenseMoneyDto {
            amount: AmountDto {
                numer: self.amount.numer().to_string(),
                denom: self.amount.denom().to_string(),
            },
            currency: C::SYMBOL,
        };

        dto.serialize(serializer)
    }
}

impl<'de, C: Currency> Deserialize<'de> for DenseMoney<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dto = DenseMoneyDto::deserialize(deserializer)?;

        if dto.currency != C::SYMBOL {
            return Err(de::Error::invalid_value(
                Unexpected::Str(dto.currency),
                &C::SYMBOL,
            ));
        }

        let numer: i64 = dto.amount.numer.parse().map_err(de::Error::custom)?;
        let denom: NonZeroI64 = dto.amount.denom.parse().map_err(de::Error::custom)?;

        Ok(DenseMoney::new(numer, denom))
    }
}
