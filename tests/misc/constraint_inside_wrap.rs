//! §11.4.13/§11.8.2: a constraint operand computed at its own width wraps
//! there (`c + 4000`, `c - 5`, `2 * c` on an `int`). `inside`, `dist` and
//! the relations over such an operand must keep propagating once all but
//! one of its variables are fixed — the solver used to wait for the
//! evaluator until every variable was guessed and gave up. Also §11.4.13:
//! a range bound is computed at the operand's width (`-2147483648` is
//! INT_MIN, not +2^31).

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1_000_000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.trim_end().to_string())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// `expr inside {[lo:hi]}` and the matching `>=`/`<=` pair over an operand
/// that may overflow, with rand or state bounds (the review's M4 shapes).
#[test]
fn inside_window_over_wrapping_sum() {
    let src = r#"module top;
  int v0, c, d;
  int unsigned u;
  shortint s;
  int ok;
  initial begin
    ok = std::randomize(v0, c) with { v0 == 4000; c + v0 inside {[v0 + 3:v0 + 4]}; };
    $display("T|N1 ok=%0d v=%0d", ok, c inside {[3:4]});
    ok = std::randomize(v0, c) with { v0 == 4000; c + v0 >= v0 + 3; c + v0 <= v0 + 4; };
    $display("T|N2 ok=%0d v=%0d", ok, c inside {[3:4]});
    v0 = 4000;
    ok = std::randomize(c) with { c + v0 inside {[v0 + 3:v0 + 4]}; };
    $display("T|N4 ok=%0d v=%0d", ok, c inside {[3:4]});
    ok = std::randomize(v0, c) with { v0 inside {[0:10000]}; c + v0 inside {[v0 + 3:v0 + 4]}; };
    $display("T|N5 ok=%0d v=%0d", ok, c inside {[3:4]});
    ok = std::randomize(v0, c, d) with { v0 == 4000; d == 5; c + d inside {[v0:v0 + 1]}; };
    $display("T|N6 ok=%0d v=%0d", ok, c inside {[3995:3996]});
    ok = std::randomize(c) with { c + 4000 inside {[4003:4004]}; };
    $display("T|L3 ok=%0d v=%0d", ok, c inside {[3:4]});
    ok = std::randomize(u) with { u + 4000 inside {[4003:4004]}; };
    $display("T|L5 ok=%0d v=%0d", ok, u inside {[3:4]});
    ok = std::randomize(c) with { c - 5 inside {[0:1]}; };
    $display("T|L7 ok=%0d v=%0d", ok, c inside {[5:6]});
    ok = std::randomize(c) with { 2 * c inside {[10:11]}; };
    $display("T|L8 ok=%0d v=%0d", ok, (2 * c) inside {[10:11]});
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|N1 ok=1 v=1",
        "T|N2 ok=1 v=1",
        "T|N4 ok=1 v=1",
        "T|N5 ok=1 v=1",
        "T|N6 ok=1 v=1",
        "T|L3 ok=1 v=1",
        "T|L5 ok=1 v=1",
        "T|L7 ok=1 v=1",
        "T|L8 ok=1 v=1",
    ];
    assert_eq!(t_lines(src), expected);
}

/// Several ranges, value lists, `!(… inside …)`, widths 8/16/32/64,
/// signed and unsigned, arrays, `dist` over an expression, and an INT_MIN
/// range bound.
#[test]
fn inside_value_lists_widths_and_negation() {
    let src = r#"module top;
  int c, d;
  int unsigned u;
  byte b;
  byte unsigned ub;
  bit [7:0] a8;
  shortint s;
  longint l;
  longint unsigned ul;
  int arr [4];
  int ok;
  initial begin
    ok = std::randomize(c) with { c + 4000 inside {[4003:4004], [5000:5000]}; };
    $display("T|M1 ok=%0d v=%0d", ok, (c + 4000) inside {[4003:4004], [5000:5000]});
    ok = std::randomize(c) with { c + 4000 inside {4010, [4003:4003]}; c != 3; };
    $display("T|M2 ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { !(c + 4000 inside {[-2147483648:4002], [4004:2147483647]}); };
    $display("T|M3 ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { c - 5 inside {[0:0]}; };
    $display("T|M4 ok=%0d c=%0d", ok, c);
    ok = std::randomize(c, d) with { c + d inside {[7:7]}; d == 1000000; };
    $display("T|M6 ok=%0d c=%0d", ok, c);
    ok = std::randomize(u) with { u + 4000 inside {[4003:4003]}; };
    $display("T|M7 ok=%0d u=%0d", ok, u);
    ok = std::randomize(u) with { u - 1 inside {[0:0]}; };
    $display("T|M8 ok=%0d u=%0d", ok, u);
    ok = std::randomize(ub) with { ub + 8'd200 inside {[8'd10:8'd10]}; };
    $display("T|M10 ok=%0d ub=%0d", ok, ub);
    ok = std::randomize(s) with { s - 16'sd5 inside {[16'sd0:16'sd0]}; };
    $display("T|M12 ok=%0d s=%0d", ok, s);
    ok = std::randomize(l) with { l + 64'sd4000 inside {[64'sd4003:64'sd4003]}; };
    $display("T|M13 ok=%0d l=%0d", ok, l);
    ok = std::randomize(ul) with { ul + 64'd7 inside {[64'd9:64'd9]}; };
    $display("T|M14 ok=%0d ul=%0d", ok, ul);
    ok = std::randomize(arr) with { foreach (arr[i]) arr[i] + 100 inside {[100 + i:100 + i]}; };
    $display("T|M15 ok=%0d a=%0d %0d %0d %0d", ok, arr[0], arr[1], arr[2], arr[3]);
    ok = std::randomize(c) with { c + 4000 dist {[4003:4003] := 1}; };
    $display("T|M16 ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { c * 2 + 1 inside {[11:11]}; c >= 0; };
    $display("T|M18 ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { (c + 4000) inside {[4003:4004]}; (c + 4000) inside {[4004:4010]}; };
    $display("T|M19 ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { c inside {[-2147483648:-2147483640]}; };
    $display("T|R3 ok=%0d v=%0d", ok, c <= -2147483640);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|M1 ok=1 v=1",
        "T|M2 ok=1 c=10",
        "T|M3 ok=1 c=3",
        "T|M4 ok=1 c=5",
        "T|M6 ok=1 c=-999993",
        "T|M7 ok=1 u=3",
        "T|M8 ok=1 u=1",
        "T|M10 ok=1 ub=66",
        "T|M12 ok=1 s=5",
        "T|M13 ok=1 l=3",
        "T|M14 ok=1 ul=2",
        "T|M15 ok=1 a=0 1 2 3",
        "T|M16 ok=1 c=3",
        "T|M18 ok=1 c=5",
        "T|M19 ok=1 c=4",
        "T|R3 ok=1 v=1",
    ];
    assert_eq!(t_lines(src), expected);
}

/// Sibling audit: `dist`, value lists, `unique`, implication and if/else,
/// `solve before`, foreach bodies (fixed, dynamic, chained), two-variable
/// sums, odd multipliers, strict and `!=` relations, class members.
#[test]
fn wrapping_operand_in_every_constraint_form() {
    let src = r#"// Constraint forms over operands that can wrap.
class cfg;
  rand int a, b;
  rand int unsigned u;
  rand bit sel;
  rand int arr [3];
  rand int q [$];
endclass
module top;
  int c, d, e;
  int unsigned u;
  int arr [3];
  int da [];
  bit sel;
  int ok;
  cfg k;
  initial begin
    ok = std::randomize(c) with { c + 4000 dist {4003 := 1, 4004 := 0}; };
    $display("T|S1 dist-val ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { c + 4000 dist {[4003:4003] :/ 1}; };
    $display("T|S2 dist-range ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { c - 10 dist {[0:0] := 1, [100:100] := 0}; };
    $display("T|S3 dist-sub ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { c + 4000 inside {4003, 4010}; c < 5; };
    $display("T|S4 in-vals ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { !(c + 4000 inside {4003}); c inside {[3:4]}; };
    $display("T|S5 not-in-val ok=%0d c=%0d", ok, c);
    ok = std::randomize(c, d) with { unique {c, d}; c + 4000 inside {[4003:4004]}; d == 3; };
    $display("T|S7 unique+in ok=%0d c=%0d", ok, c);
    ok = std::randomize(sel, c) with { sel == 1; sel -> c + 4000 inside {[4003:4003]}; };
    $display("T|S8 impl-in ok=%0d c=%0d", ok, c);
    ok = std::randomize(sel, c) with { sel == 0; if (sel) c == 1; else c + 4000 inside {[4005:4005]}; };
    $display("T|S9 ifelse-in ok=%0d c=%0d", ok, c);
    ok = std::randomize(sel, c) with { c + 4000 inside {[4003:4003]} -> sel == 1; c == 3; };
    $display("T|S10 in-antecedent ok=%0d sel=%0d", ok, sel);
    ok = std::randomize(c, d) with { solve d before c; d inside {[1:1]}; c + d inside {[4003:4003]}; };
    $display("T|S11 solve-before ok=%0d c=%0d", ok, c);
    ok = std::randomize(arr) with { foreach (arr[i]) arr[i] + 4000 inside {[4000 + i:4000 + i]}; };
    $display("T|S12 foreach-in ok=%0d a=%0d,%0d,%0d", ok, arr[0], arr[1], arr[2]);
    ok = std::randomize(arr) with { foreach (arr[i]) arr[i] - 3 dist {[i:i] := 1}; };
    $display("T|S13 foreach-dist ok=%0d a=%0d,%0d,%0d", ok, arr[0], arr[1], arr[2]);
    ok = std::randomize(da) with { da.size() == 2; foreach (da[i]) da[i] + 100 inside {[100 + 2 * i:100 + 2 * i]}; };
    $display("T|S14 foreach-dyn ok=%0d a=%0d,%0d", ok, da[0], da[1]);
    ok = std::randomize(arr) with { foreach (arr[i]) if (i > 0) arr[i] + 4000 inside {[arr[i-1] + 4001:arr[i-1] + 4001]}; arr[0] == 7; };
    $display("T|S15 foreach-chain ok=%0d a=%0d,%0d,%0d", ok, arr[0], arr[1], arr[2]);
    ok = std::randomize(u) with { u + 32'd5 dist {[32'd2:32'd2] := 1}; };
    $display("T|S16 dist-uns ok=%0d u=%0d", ok, u);
    ok = std::randomize(c, d) with { c + d inside {[7:7]}; d inside {[2147483647:2147483647]}; };
    $display("T|S17 two-var ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { c * 3 inside {[9:9]}; };
    $display("T|S18 mul-odd ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { c + 4000 > 4002; c + 4000 < 4004; };
    $display("T|S19 rel-strict ok=%0d c=%0d", ok, c);
    ok = std::randomize(c) with { c + 4000 != 4003; c inside {[3:4]}; };
    $display("T|S20 rel-ne ok=%0d c=%0d", ok, c);
    k = new;
    ok = k.randomize() with { a + 4000 inside {[4003:4003]}; b - 2 dist {[0:0] := 1}; u + 1 inside {[0:0]}; };
    $display("T|S21 class ok=%0d a=%0d b=%0d u=%0d", ok, k.a, k.b, k.u);
    ok = k.randomize() with { foreach (arr[i]) arr[i] + 4000 inside {[4010 + i:4010 + i]}; };
    $display("T|S22 class-foreach ok=%0d a=%0d,%0d,%0d", ok, k.arr[0], k.arr[1], k.arr[2]);
    ok = k.randomize() with { q.size() == 2; foreach (q[i]) q[i] + 4000 inside {[4020 + i:4020 + i]}; };
    $display("T|S23 class-queue ok=%0d q=%0d,%0d", ok, k.q[0], k.q[1]);
    ok = k.randomize() with { sel -> a + 4000 inside {[4003:4003]}; !sel -> a + 4000 inside {[4004:4004]}; sel == 1; };
    $display("T|S24 class-impl ok=%0d a=%0d", ok, k.a);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|S1 dist-val ok=1 c=3",
        "T|S2 dist-range ok=1 c=3",
        "T|S3 dist-sub ok=1 c=10",
        "T|S4 in-vals ok=1 c=3",
        "T|S5 not-in-val ok=1 c=4",
        "T|S7 unique+in ok=1 c=4",
        "T|S8 impl-in ok=1 c=3",
        "T|S9 ifelse-in ok=1 c=5",
        "T|S10 in-antecedent ok=1 sel=1",
        "T|S11 solve-before ok=1 c=4002",
        "T|S12 foreach-in ok=1 a=0,1,2",
        "T|S13 foreach-dist ok=1 a=3,4,5",
        "T|S14 foreach-dyn ok=1 a=0,2",
        "T|S15 foreach-chain ok=1 a=7,8,9",
        "T|S16 dist-uns ok=1 u=4294967293",
        "T|S17 two-var ok=1 c=-2147483640",
        "T|S18 mul-odd ok=1 c=3",
        "T|S19 rel-strict ok=1 c=3",
        "T|S20 rel-ne ok=1 c=4",
        "T|S21 class ok=1 a=3 b=2 u=4294967295",
        "T|S22 class-foreach ok=1 a=10,11,12",
        "T|S23 class-queue ok=1 q=20,21",
        "T|S24 class-impl ok=1 a=3",
    ];
    assert_eq!(t_lines(src), expected);
}

/// The same shapes in class constraint blocks (`randomize()` and
/// `randomize() with`).
#[test]
fn wrapping_operand_in_class_constraints() {
    let src = r#"class pkt;
  rand int c;
  rand int unsigned u;
  rand int q [3];
  rand bit [7:0] n;
  constraint c1 { c + 4000 inside {[4003:4003]}; }
  constraint c2 { u + 32'd10 inside {[32'd4:32'd4]}; }
  constraint c4 { foreach (q[i]) q[i] - 7 inside {[i:i]}; }
  constraint c5 { n - 8'd3 inside {[8'd255:8'd255]}; }
endclass
module top;
  initial begin
    pkt p = new;
    int ok;
    ok = p.randomize();
    $display("T|C1 ok=%0d c=%0d u=%0d q=%0d,%0d,%0d n=%0d", ok, p.c, p.u, p.q[0], p.q[1], p.q[2], p.n);
    ok = p.randomize() with { c + 1 inside {[4004:4004]}; };
    $display("T|C2 ok=%0d", ok);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = ["T|C1 ok=1 c=3 u=4294967290 q=7,8,9 n=2", "T|C2 ok=0"];
    assert_eq!(t_lines(src), expected);
}

/// Pinned: arithmetic over a variable wider than 64 bits (`w + 80'd5 ==
/// 80'd2`) is not modelled by the wide-variable solver.
#[test]
#[ignore = "wide (>64-bit) arithmetic in constraints is not modelled; randomize() fails"]
fn wide_operand_arithmetic_in_constraints() {
    let src = r#"// Wide (>64-bit) operands that may wrap.
module top;
  bit [79:0] w;
  bit [127:0] x;
  int ok;
  initial begin
    ok = std::randomize(w) with { w + 80'd5 == 80'd2; };
    $display("T|W1 ok=%0d w=%0h", ok, w);
    ok = std::randomize(w) with { w + 80'd5 inside {[80'd2:80'd2]}; };
    $display("T|W2 ok=%0d w=%0h", ok, w);
    ok = std::randomize(w) with { w + 80'd5 >= 80'd2; w + 80'd5 <= 80'd2; };
    $display("T|W3 ok=%0d w=%0h", ok, w);
    ok = std::randomize(x) with { x - 128'd1 inside {[128'd0:128'd0]}; };
    $display("T|W4 ok=%0d x=%0h", ok, x);
    ok = std::randomize(w) with { w inside {[80'd2:80'd3]}; };
    $display("T|W5 ok=%0d v=%0d", ok, w inside {[80'd2:80'd3]});
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = [
        "T|W1 ok=1 w=fffffffffffffffffffd",
        "T|W2 ok=1 w=fffffffffffffffffffd",
        "T|W3 ok=1 w=fffffffffffffffffffd",
        "T|W4 ok=1 x=1",
        "T|W5 ok=1 v=1",
    ];
    assert_eq!(t_lines(src), expected);
}

/// Pinned: §11.4.13 evaluates `inside` like `==`, at the width of all
/// operands; xezim evaluates the left operand self-determined.
#[test]
#[ignore = "inside evaluates its left operand at its own width, not the width of all operands (procedural and constraint)"]
fn inside_operands_take_the_widest_operand_width() {
    let src = r#"module top;
  bit [7:0] a8;
  byte b;
  int ok;
  initial begin
    a8 = 255;
    $display("T|P1 %0d %0d", (a8 + 8'd1) inside {[0:0]}, (a8 + 8'd1) inside {[256:256]});
    ok = std::randomize(a8) with { a8 + 8'd1 inside {[256:256]}; };
    $display("T|R4 ok=%0d a8=%0d", ok, a8);
    ok = std::randomize(a8) with { a8 + 8'd1 inside {[0:0]}; };
    $display("T|R5 ok=%0d", ok);
    ok = std::randomize(b) with { b + 8'sd127 inside {[-2:-2]}; };
    $display("T|R6 ok=%0d", ok);
  end
endmodule
"#;
    // Expected values from the reference simulator.
    let expected = ["T|P1 0 1", "T|R4 ok=1 a8=255", "T|R5 ok=0", "T|R6 ok=0"];
    assert_eq!(t_lines(src), expected);
}
