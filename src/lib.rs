pub mod currency;
pub mod dense;
pub mod discrete;

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::{currency::*, discrete::DiscreteMoney};

    #[test]
    fn dense() {
        let a = Myr::new(20, 1i64.try_into().unwrap()); // RM 20
        let b = Myr::new(40, 1i64.try_into().unwrap()); // RM 20

        assert_eq!(
            a.checked_add(b).unwrap(),
            Myr::new(60, 1i64.try_into().unwrap())
        ); // RM 60
    }

    #[test]
    fn discrete() {
        let a = DiscreteMoney::<Usd>::new(50).to_dense(); // $0.50

        assert_eq!(a, Usd::new(1, 2i64.try_into().unwrap()));
    }

    #[test]
    fn round_retains_value() {
        let amount = Gbp::new(1243, 1000i64.try_into().unwrap()); // £1.243
        let (approx, rest) = amount.round();

        assert_eq!(approx, DiscreteMoney::<Gbp>::new(124)); // £1.24
        assert_eq!(rest, Gbp::new(3, 1000i64.try_into().unwrap())); // 0.003p
    }
}
