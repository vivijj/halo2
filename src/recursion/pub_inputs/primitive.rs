//! Primitives used in packing and unpacking public inputs.

use ff::Field;
use pasta_curves::arithmetic::{CurveAffine, FieldExt};

use std::collections::BTreeSet;
use std::convert::TryInto;


/// Lookup table mapping endoscalars to their corresponding NUM_BITS-bit representations.
fn lookup_table<F: FieldExt, const NUM_BITS: usize>() -> Vec<([bool; NUM_BITS], F)> {
    let mut endoscalars = BTreeSet::new();
    let mut table = Vec::new();
    let num_rows = 1 << NUM_BITS;

    for row in 0..num_rows {
        let bits = i2lebsp(row as u64);
        let scalar = endoscale(bits);

        assert!(endoscalars.insert(scalar));

        table.push((bits, scalar));
    }

    table
}

/// Maps an N-bit bitstring to a scalar.
///
/// This corresponds to Algorithm 1 from [BGH2019], where `F` corresponds to $F_q$, the
/// scalar field of $P$. Where Algorithm 1 computes $Acc = [scalar] P$, this function
/// computes `scalar`.
///
/// [BGH2019]: https://eprint.iacr.org/2019/1021.pdf
pub(crate) fn endoscale<F: FieldExt, const N: usize>(bits: [bool; N]) -> F {
    assert_eq!(N % 2, 0);

    /// Maps a pair of bits to a scalar using endoscaling.
    fn endoscale_pair<F: FieldExt>(bits: (bool, bool)) -> F {
        // [2 * bits.0 - 1]
        let mut scalar = F::from_u64(bits.0.into()).double() - F::one();

        if bits.1 {
            scalar = scalar * F::ZETA;
        }

        scalar
    }

    let mut scalar = (F::ZETA + F::one()).double();

    for j in 0..(N / 2) {
        let pair = (bits[2 * j], bits[2 * j + 1]);
        scalar = endoscale_pair::<F>(pair) + scalar.double();
    }

    scalar
}

pub(crate) fn i2lebsp<const NUM_BITS: usize>(int: u64) -> [bool; NUM_BITS] {
    assert!(NUM_BITS <= 64);

    fn gen_const_array<Output: Copy + Default, const LEN: usize>(
        closure: impl FnMut(usize) -> Output,
    ) -> [Output; LEN] {
        fn gen_const_array_with_default<Output: Copy, const LEN: usize>(
            default_value: Output,
            mut closure: impl FnMut(usize) -> Output,
        ) -> [Output; LEN] {
            let mut ret: [Output; LEN] = [default_value; LEN];
            for (bit, val) in ret.iter_mut().zip((0..LEN).map(|idx| closure(idx))) {
                *bit = val;
            }
            ret
        }
        gen_const_array_with_default(Default::default(), closure)
    }

    gen_const_array(|mask: usize| (int & (1 << mask)) != 0)
}

#[test]
fn test_lookup() {
    use pasta_curves::pallas;

    lookup_table::<pallas::Scalar, 10>();
}
