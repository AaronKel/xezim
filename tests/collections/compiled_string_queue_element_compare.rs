//! IEEE 1800-2017 §6.16 / §7.10: an element of a LOCAL string queue is a
//! string. In a compiled method body (`XEZIM_METHOD_TIER=0`) the element read
//! took the nominal width of the comparison context — one bit for
//! `q[0] == b` with a string formal `b` — so the compare was always 0, and
//! `q[0] == q[1]` compared bit 0 of each. A string element now keeps its
//! text and compares as a string. Expected values are the reference
//! simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.trim().to_string())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

const ISSUE: &str = r#"
class C;
  function int a4(string a, string b); string q[$]; q.push_back(a); return q[0] == b; endfunction
  function int g1(string a); string q[$]; q.push_back(a); return q[0] == "e"; endfunction
  function int g4(string a); string q[$]; q.push_back(a); return q[0] == "ae"; endfunction
  function int g5(string a); string q[$]; q.push_back(a); return q[0] < "f"; endfunction
endclass
module top;
  initial begin
    C c; c = new;
    repeat (2) $display("T|%0d %0d %0d %0d %0d | %0d %0d %0d %0d %0d %0d", c.a4("[0]", "e"), c.a4("ab", "e"), c.a4("xe", "e"), c.a4("e", "e"), c.a4("ab", "ab"),
      c.g1("e"), c.g1("ae"), c.g4("ae"), c.g4("e"), c.g5("e"), c.g5("g"));
  end
endmodule
"#;

#[test]
fn local_string_queue_element_compares_as_string() {
    let _ = super::compiled_method_test_env::policies(&[("1", "0"), ("0", "0")]);
    let want = ["T|0 0 0 1 1 | 1 0 1 0 1 0", "T|0 0 0 1 1 | 1 0 1 0 1 0"];
    assert_eq!(t_lines(ISSUE), want);
}

const SHAPES: &str = r#"
class C;
  string mq[$];
  function int f1(string a, string b); string q[$]; q.push_back(a); return q[0] == b; endfunction
  function int f2(string a, string b); string q[$]; q.push_back(a); return q[0] == "e"; endfunction
  function int f3(string a, string b); string q[$]; string t; t = b; q.push_back(a); return q[0] == t; endfunction
  function int f4(string a, string b); string q[$]; q.push_back(a); return q[0] != b; endfunction
  function int f5(string a, string b); string d[]; d = new[1]; d[0] = a; return d[0] == b; endfunction
  function int f6(string a, string b); mq.delete(); mq.push_back(a); return mq[0] == b; endfunction
  function int f7(string a, string b); string q[$]; q.push_back(a); return b == q[0]; endfunction
  function int f8(string a, string b); string q[$]; q.push_back(a); return q[0] < b; endfunction
  function int f9(string a, string b); string aa[string]; aa["k"] = a; return aa["k"] == b; endfunction
  function int f10(string a, string b); string q[$]; q.push_back(a); q.push_back(b); return q[0] == q[1]; endfunction
  function int f11(string a, string b); string q[$]; q.push_back(a); return (q[0] == b) ? 5 : 6; endfunction
  function int f12(string a, string b); string q[$]; q.push_back(a); if (q[0] == b) return 1; return 0; endfunction
  function int f13(string a, string b); string q[$]; q.push_back(a); return q[0].compare(b); endfunction
  function int f14(string a, string b); string q[$]; q.push_back(a); return q[0] inside {b}; endfunction
endclass
module top;
  initial begin
    C c; c = new;
    repeat (2) begin
      $display("T|f1 %0d %0d %0d %0d %0d", c.f1("[0]", "e"), c.f1("ab", "e"), c.f1("xe", "e"), c.f1("e", "e"), c.f1("ab", "ab"));
      $display("T|f2 %0d %0d", c.f2("e", "e"), c.f2("ab", "e"));
      $display("T|f3 %0d %0d %0d", c.f3("e", "e"), c.f3("ab", "ab"), c.f3("xe", "e"));
      $display("T|f4 %0d %0d %0d", c.f4("e", "e"), c.f4("ab", "ab"), c.f4("xe", "e"));
      $display("T|f5 %0d %0d %0d", c.f5("e", "e"), c.f5("ab", "ab"), c.f5("xe", "e"));
      $display("T|f6 %0d %0d %0d", c.f6("e", "e"), c.f6("ab", "ab"), c.f6("xe", "e"));
      $display("T|f7 %0d %0d %0d", c.f7("e", "e"), c.f7("ab", "ab"), c.f7("xe", "e"));
      $display("T|f8 %0d %0d %0d", c.f8("e", "f"), c.f8("ab", "ab"), c.f8("b", "ab"));
      $display("T|f9 %0d %0d %0d", c.f9("e", "e"), c.f9("ab", "ab"), c.f9("xe", "e"));
      $display("T|f10 %0d %0d %0d", c.f10("e", "e"), c.f10("ab", "ab"), c.f10("xe", "e"));
      $display("T|f11 %0d %0d %0d", c.f11("e", "e"), c.f11("ab", "ab"), c.f11("xe", "e"));
      $display("T|f12 %0d %0d %0d", c.f12("e", "e"), c.f12("ab", "ab"), c.f12("xe", "e"));
      $display("T|f13 %0d %0d %0d", c.f13("e", "e"), c.f13("ab", "ac"), c.f13("xe", "e"));
      $display("T|f14 %0d %0d %0d", c.f14("e", "e"), c.f14("ab", "ab"), c.f14("xe", "e"));
    end
  end
endmodule
"#;

/// Neighbours: `!=`, `<`, reversed operands, two elements, `if`, `?:`,
/// `compare()`, dynamic-array / member / associative elements.
#[test]
fn string_collection_element_compare_shapes() {
    let _ = super::compiled_method_test_env::policies(&[("1", "0"), ("0", "0")]);
    let want = [
        "T|f1 0 0 0 1 1",
        "T|f2 1 0",
        "T|f3 1 1 0",
        "T|f4 0 0 1",
        "T|f5 1 1 0",
        "T|f6 1 1 0",
        "T|f7 1 1 0",
        "T|f8 1 0 0",
        "T|f9 1 1 0",
        "T|f10 1 1 0",
        "T|f11 5 5 6",
        "T|f12 1 1 0",
        "T|f13 0 -1 19",
        "T|f14 1 1 0",
        "T|f1 0 0 0 1 1",
        "T|f2 1 0",
        "T|f3 1 1 0",
        "T|f4 0 0 1",
        "T|f5 1 1 0",
        "T|f6 1 1 0",
        "T|f7 1 1 0",
        "T|f8 1 0 0",
        "T|f9 1 1 0",
        "T|f10 1 1 0",
        "T|f11 5 5 6",
        "T|f12 1 1 0",
        "T|f13 0 -1 19",
        "T|f14 1 1 0",
    ];
    assert_eq!(t_lines(SHAPES), want);
}
