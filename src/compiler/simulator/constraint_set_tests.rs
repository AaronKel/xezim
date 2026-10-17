use super::*;

#[test]
fn inside_sampling_fits_signed_and_unsigned_comparison_domains() {
    let fit = Simulator::fit_cons_ranges_for_target;
    assert_eq!(
        fit(
            vec![(-1, -1), (0, 0), (255, 255)],
            Some((32, true)),
            8,
            true
        ),
        vec![(-1, -1), (0, 0)]
    );
    assert_eq!(
        fit(vec![(255, 255), (0, 0)], Some((8, false)), 8, true),
        vec![(-1, -1), (0, 0)]
    );
    assert_eq!(
        fit(vec![(255, 255), (0, 0)], Some((16, false)), 8, true),
        vec![(0, 0)]
    );
    assert_eq!(
        fit(vec![(0, u64::MAX as i128)], Some((64, false)), 64, true),
        vec![(0, i64::MAX as i128), (i64::MIN as i128, -1)]
    );
}

#[test]
fn interval_union_sampling_handles_full_i128_cardinality() {
    let mut sim = crate::simulate("module top; endmodule", 0).expect("simulate");
    let mut negative = false;
    let mut positive = false;
    let mut high = false;
    for _ in 0..64 {
        // Adjacent intervals normalize to the full 2^128-value domain.
        let v = sim
            .pick_i128_range(&[(i128::MIN, -1), (0, i128::MAX)], 128, true)
            .expect("pick");
        let raw = v.to_u128();
        negative |= raw >> 127 != 0;
        positive |= raw >> 127 == 0;
        high |= raw >> 64 != 0;
    }
    assert!(
        negative && positive && high,
        "128-bit sampling lost its upper bits"
    );
    // The next-largest possible cardinality must also avoid signed overflow.
    assert!(
        sim.pick_i128_range(
            &[(i128::MIN, i128::MIN), (i128::MIN + 2, i128::MAX)],
            128,
            true
        )
        .is_some()
    );
    assert!(sim.pick_i128_range(&[(2, 1)], 64, false).is_none());
}
