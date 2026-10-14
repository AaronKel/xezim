//! §18.5.4 / §11.4.13: `dist` and `inside` on members wider than 64 bits,
//! the neighbours of #301.
//!
//! The joint solver splits such a member into segments of at most 64 bits.
//! A `dist` over the whole member was not modelled at all (`randomize()`
//! returned 0); it now constrains the member to its items, drawn uniformly
//! among them. `x inside {K1, K2, ...}` over constants was only the
//! disjunction of whole-value equalities, which acts once all but one
//! alternative are excluded, so with a 64-bit segment drawn at random the
//! search ran out of budget; each segment is now also limited to the bits
//! the values hold there.
//!
//! Expected values come from the reference simulator: every probe prints
//! whether its constraints hold, not the random values.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn wide_dist_and_inside_sets() {
    let src = include_str!("randomize_wide_sets.sv");
    let expected: Vec<String> = src
        .lines()
        .filter_map(|l| {
            let i = l.find("$display(\"T|")? + "$display(\"".len();
            let name = l[i..].split_whitespace().next()?;
            Some(format!("{name} r=8 ok=8"))
        })
        .collect();
    assert_eq!(expected.len(), 70);
    assert_eq!(t_lines(src), expected);
}
