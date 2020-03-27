use crate::{Affine, Jacobian, BETA};
use proptest::prelude::*;
use zkp_primefield::{FieldElement, Pow, SquareRoot};

impl Arbitrary for Affine {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        FieldElement::arbitrary()
            .prop_filter_map("x not on curve", |x| {
                let y = (x.pow(3_usize) + &x + BETA).square_root();
                match y {
                    Some(y) => Some(Affine::new(x, y)),
                    None => None,
                }
            })
            .boxed()
    }
}

impl Arbitrary for Jacobian {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        Affine::arbitrary().prop_map(Self::from).boxed()
    }
}

mod tests {
    use super::*;

    proptest!(
        #[test]
        fn affine_on_curve(a: Affine) {
            prop_assert!(a.on_curve());
        }
    );

    proptest!(
        #[test]
        fn jacobian_on_curve(j: Jacobian) {
            prop_assert!(j.on_curve());
        }
    );
}
