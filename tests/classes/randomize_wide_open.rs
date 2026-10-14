//! Open `randomize()` gaps found next to #301 (all ignored). Each probe of
//! `randomize_wide_open.sv` prints whether its constraints hold; the
//! reference simulator prints `r=8 ok=8` on every line.

use xezim::simulate;

fn check(prefix: &str) {
    let src = include_str!("randomize_wide_open.sv");
    let got: Vec<String> = simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with(&format!("T|{prefix}")))
        .collect();
    let want: Vec<String> = src
        .lines()
        .filter_map(|l| {
            let i = l.find(&format!("$display(\"T|{prefix}"))? + "$display(\"".len();
            let name = l[i..].split_whitespace().next()?;
            Some(format!("{name} r=8 ok=8"))
        })
        .collect();
    assert!(!want.is_empty());
    assert_eq!(got, want);
}

#[test]
#[ignore = "the joint solver keeps out a class whose rand sub-object holds a rand sub-object (`a.m -> a.b.c.x == K`); the trial loop then fails"]
fn constraint_through_two_levels_of_sub_objects() {
    check("nest3_");
}

#[test]
#[ignore = "a `rand` unpacked struct member keeps the class out of the joint solver; the trial loop cannot meet constraints on its wide fields"]
fn rand_unpacked_struct_with_wide_fields() {
    check("ustruct_");
}

#[test]
#[ignore = "`z == (x ^ y)` over wide elements of a rand dynamic array is not modelled"]
fn wide_dynamic_array_elements_through_xor() {
    check("darr_");
}

#[test]
#[ignore = "std::randomize() of locals wider than 64 bits is not modelled by the scope solver"]
fn std_randomize_of_wide_locals() {
    check("std_");
}
