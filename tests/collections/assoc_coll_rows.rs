//! IEEE 1800-2017 §7.8, §7.10, §21.2.1.7: an associative array whose
//! elements are collections. A write into an absent row (`aq[6][0] = 9`)
//! creates the row as its declared kind, `%p` prints each row as a nested
//! list, `foreach (aq[k][j])` walks the row, and a string-valued row takes
//! `m[k]["x"] = "hi"` as an element write (it was a character write and was
//! dropped). Expected values are Questa's, with the house `%p` spacing (no
//! space before an associative array's `}`).
use xezim::simulate;

/// The `$display` lines of `src` that start with one of `prefixes`.
fn shown(src: &str, prefixes: &[&str]) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate")
        .output
        .iter()
        .map(|l| l.message.clone())
        .filter(|m| prefixes.iter().any(|p| m.starts_with(p)))
        .collect()
}

/// Queue, associative and string-keyed rows.
#[test]
fn absent_rows_are_created_and_printed() {
    let src = r#"module tb;
  int aq [int][$];
  int aa [int][int];
  int ad [string][];
  string as [int][string];
  initial begin
    aq[6][0] = 9; $display("A %p size=%0d n=%0d", aq, aq[6].size(), aq.num());
    aq[6][1] = 4; aq[2].push_back(1); $display("B %p", aq);
    aa[1][2] = 3; aa[4][5] = 6; aa[1][7] = 8; $display("C %p %0d %0d", aa, aa.num(), aa[1].num());
    as[3]["x"] = "hi"; $display("D %p", as);
    $display("E %0d %0d", aa[1][7], aq[6][1]);
  end
endmodule
"#;
    let want = r#"A '{6:'{9}} size=1 n=1
B '{2:'{1}, 6:'{9, 4}}
C '{1:'{2:3, 7:8}, 4:'{5:6}} 2 2
D '{3:'{"x":"hi"}}
E 8 4"#;
    let got = shown(src, &["A ", "B ", "C ", "D ", "E "]);
    assert_eq!(got.join("\n"), want, "{got:#?}");
}

/// `foreach (aq[6][j])` binds `j` to the row's indices.
#[test]
fn foreach_walks_an_associative_row() {
    let src = r#"module tb;
  int aq [int][$];
  initial begin
    aq[6][0] = 9;
    $display("A %0d %0d %0d", aq[6].size(), aq[6][0], aq.num());
    foreach (aq[k]) $display("K %0d", k);
    foreach (aq[6][j]) $display("J %0d %0d", j, aq[6][j]);
  end
endmodule
"#;
    let want = r#"A 1 9 1
K 6
J 0 9"#;
    let got = shown(src, &["A ", "K ", "J "]);
    assert_eq!(got.join("\n"), want, "{got:#?}");
}

/// String elements under integer and string keys, and a queue row.
#[test]
fn string_rows_take_element_writes() {
    let src = r#"module tb;
  string a1 [int][string];
  int    a2 [int][string];
  string a3 [string][int];
  string a4 [int][int];
  string a5 [int][$];
  initial begin
    a1[3]["x"] = "hi"; a2[3]["x"] = 5; a3["k"][2] = "s3"; a4[1][2] = "s4"; a5[1].push_back("q");
    $display("A %s %0d | %0d %0d | %s %0d | %s %0d | %s %0d", a1[3]["x"], a1[3].num(), a2[3]["x"], a2[3].num(), a3["k"][2], a3["k"].num(), a4[1][2], a4[1].num(), a5[1][0], a5[1].size());
    $display("P %p %p %p %p %p", a1, a2, a3, a4, a5);
  end
endmodule
"#;
    let want = r#"A hi 1 | 5 1 | s3 1 | s4 1 | q 1
P '{3:'{"x":"hi"}} '{3:'{"x":5}} '{"k":'{2:"s3"}} '{1:'{2:"s4"}} '{1:'{"q"}}"#;
    let got = shown(src, &["A ", "P "]);
    assert_eq!(got.join("\n"), want, "{got:#?}");
}
