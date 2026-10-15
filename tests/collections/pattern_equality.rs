//! IEEE 1800-2017 §11.4.5 / §10.9: `==`, `!=`, `===` and `!==` between an
//! unpacked aggregate and an ASSIGNMENT PATTERN compare element by element
//! (member by member for a struct). The pattern is in an assignment-like
//! context, so it takes the other operand's type: `index:`, `default:`,
//! nested patterns and the element-type conversion of each item apply.
//! The pattern used to evaluate to a packed concatenation compared with the
//! array's nonexistent container signal, so `a == '{9, 5, 5, 5}` was 0.
//! Expected values are the reference simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

const PATTERN_EQ: &str = r#"
typedef struct { int x; int y; } st_t;
class C;
  int a[4]; int m[2][2]; int d[]; int q[$]; string sa[2]; st_t s;
  function string run();
    a = '{9, 5, 5, 5}; m = '{'{1, 2}, '{3, 4}}; d = '{1, 2, 3}; q = '{7, 8};
    sa = '{"a", "b"}; s = '{1, 2};
    return $sformatf("%0d%0d%0d%0d%0d %0d%0d%0d%0d%0d%0d %0d%0d%0d%0d%0d", a == '{9, 5, 5, 5}, a != '{9, 5, 5, 6},
      a === '{9, 5, 5, 5}, '{9, 5, 5, 5} == a, a != '{9, 5, 5, 5},
      m == '{'{1, 2}, '{3, 4}}, m == '{'{1, 2}, '{3, 5}}, d == '{1, 2, 3}, d == '{1, 2, 4}, q == '{7, 8}, q != '{7, 8},
      sa == '{"a", "b"}, sa == '{"a", "c"}, s == '{1, 2}, s == '{1, 3}, a == '{default: 5});
  endfunction
endclass
module top;
  int a[4]; int m[2][2]; int d[]; int q[$]; string sa[2]; st_t s; int k;
  int ds[3:1]; st_t stv[2]; logic [3:0] lx[2]; bit [7:0] by[3]; int m3[2][2][2];
  initial begin
    C c;
    a = '{9, 5, 5, 5}; m = '{'{1, 2}, '{3, 4}}; d = '{1, 2, 3}; q = '{7, 8};
    sa = '{"a", "b"}; s = '{1, 2};
    $display("T|m1 %0d%0d%0d%0d%0d %0d%0d%0d%0d%0d%0d %0d%0d%0d%0d%0d", a == '{9, 5, 5, 5}, a != '{9, 5, 5, 6},
      a === '{9, 5, 5, 5}, '{9, 5, 5, 5} == a, a != '{9, 5, 5, 5},
      m == '{'{1, 2}, '{3, 4}}, m == '{'{1, 2}, '{3, 5}}, d == '{1, 2, 3}, d == '{1, 2, 4}, q == '{7, 8}, q != '{7, 8},
      sa == '{"a", "b"}, sa == '{"a", "c"}, s == '{1, 2}, s == '{1, 3}, a == '{default: 5});
    if (a == '{9, 5, 5, 5}) $display("T|m2 if yes"); else $display("T|m2 if no");
    k = 0; while (!(a == '{9, 5, 5, 5}) && k < 3) k++;
    $display("T|m3 k=%0d tern=%0d slice=%0d", k, (a == '{9, 5, 5, 5}) ? 11 : 22, a[1:3] == '{5, 5, 5});
    ds = '{1, 2, 3}; stv = '{'{1, 2}, '{3, 4}}; lx = '{4'b1x01, 4'b0011}; by = '{1, 2, 255};
    m3 = '{'{'{1, 2}, '{3, 4}}, '{'{5, 6}, '{7, 8}}};
    $display("T|m4 ds=%0d%0d stv=%0d%0d lx=%0d%0d%0d%0d", ds == '{1, 2, 3}, ds == '{3, 2, 1}, stv == '{'{1, 2}, '{3, 4}}, stv == '{'{1, 2}, '{3, 5}},
      lx == '{4'b1x01, 4'b0011}, lx != '{4'b1x01, 4'b0011}, lx === '{4'b1x01, 4'b0011}, lx !== '{4'b1x01, 4'b0011});
    $display("T|m5 key=%0d%0d by=%0d%0d m3=%0d%0d sn=%0d%0d%0d", a == '{0: 9, default: 5}, a == '{0: 5, default: 9}, by == '{1, 2, 255}, by == '{1, 2, -1},
      m3 == '{'{'{1, 2}, '{3, 4}}, '{'{5, 6}, '{7, 8}}}, m3 == '{default: 1}, s == '{x: 1, y: 2}, s == '{y: 1, x: 2}, s == '{default: 1});
    c = new;
    repeat (3) $display("T|c1 %s", c.run());
  end
endmodule
"#;

#[test]
fn unpacked_aggregates_compare_with_assignment_patterns() {
    let _ = super::compiled_method_test_env::eager();
    let want = [
        "T|m1 11110 101010 10100",
        "T|m2 if yes",
        "T|m3 k=0 tern=11 slice=1",
        "T|m4 ds=10 stv=10 lx=0010",
        "T|m5 key=10 by=11 m3=10 sn=100",
        "T|c1 11110 101010 10100",
        "T|c1 11110 101010 10100",
        "T|c1 11110 101010 10100",
    ];
    assert_eq!(t_lines(PATTERN_EQ), want);
}

const CONT_ASSIGN: &str = r#"
module top;
  logic [3:0] cc[2]; int cd[2];
  assign cc = '{4'd3, 4'd4};
  assign cd = '{5, 6};
  initial #1 $display("T|ca %0d %0d %0d %0d %0d", cc[0], cc[1], cd[0], cd[1], cd == '{5, 6});
endmodule
"#;

/// §10.3 / §10.9: a continuous assignment of an assignment pattern to an
/// unpacked array (a different path from the comparison above).
#[test]
#[ignore = "continuous assign of an assignment pattern to an unpacked array leaves the elements x/0"]
fn continuous_assign_pattern_to_unpacked_array() {
    let want = ["T|ca 3 4 5 6 1"];
    assert_eq!(t_lines(CONT_ASSIGN), want);
}
