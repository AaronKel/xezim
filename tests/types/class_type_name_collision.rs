//! §8.3, §6.18 — a typedef declared inside a class is scoped to that class
//! (issue #307, reported by Ganesh-TNL).
//!
//! Two classes each declaring a typedef `r` with different packed-struct
//! layouts shared one entry keyed by the bare name, so the later class's
//! layout replaced the earlier one's. A `rand` member of the first class's
//! `r` was then read through the wrong struct: its fields read x, every
//! trial failed `c_sel`, and `randomize()` exhausted its budget and
//! returned 0. This is the class-scoped residual of #300.
//!
//! Expected values come from the reference simulator.

use xezim::simulate;

fn lines(src: &str) -> Vec<String> {
    simulate(src, 100)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .collect()
}

fn t_lines(src: &str) -> Vec<String> {
    lines(src)
        .into_iter()
        .filter(|m| m.starts_with("T|"))
        .collect()
}

/// The issue's reproducer: `leaf_cfg::r` is randomized while a later,
/// never-instantiated `other_cfg` declares its own `r`.
#[test]
fn issue_307_same_typedef_name_in_two_classes() {
    let out = lines(include_str!("issue_307_class_typedef_collision.sv"));
    assert!(
        out.iter().any(|m| m.starts_with("MWE-BUDGET: PASS")),
        "{out:?}"
    );
    assert!(
        !out.iter().any(|m| m.contains("budget exhausted")),
        "{out:?}"
    );
}

/// `$bits(leaf_cfg::r)` against `$bits(other_cfg::r)`, inside and outside the classes.
#[test]
fn typedefs_of_two_classes_by_scope() {
    let src = r#"
class leaf_cfg;
  typedef struct packed { bit [1:0] intlv; bit [3:0] num_hosts; bit [3:0] first_host; } r;
  rand r regs_r;
  function int w(); return $bits(r); endfunction
  function int wm(); return $bits(regs_r); endfunction
endclass
class other_cfg;
  typedef struct packed { bit [7:0] other_field; bit [15:0] another; } r;
  r m;
  function int w(); return $bits(r); endfunction
  function int wm(); return $bits(m); endfunction
endclass
module top;
  initial begin
    leaf_cfg a = new; other_cfg b = new;
    a.regs_r = '1; a.regs_r.num_hosts = 4'h5;
    b.m = '0; b.m.another = 16'hBEEF;
    $display("T|bits %0d %0d %0d %0d", $bits(leaf_cfg::r), $bits(other_cfg::r), a.w(), b.w());
    $display("T|mem %0d %0d %h %h %h", a.wm(), b.wm(), a.regs_r, a.regs_r.num_hosts, b.m.another);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|bits 10 24 10 24", "T|mem 10 24 35f 5 beef"]
    );
}

/// A typedef of a nested class, of its outer class and of another class.
#[test]
fn nested_class_typedef() {
    let src = r#"
class outer;
  class inner;
    typedef struct packed { bit [2:0] a; bit [4:0] b; } r;
    r v;
    function int w(); return $bits(r); endfunction
  endclass
  typedef logic [11:0] r;
  r o;
endclass
class other;
  typedef struct packed { bit [15:0] x; } r;
  r m;
endclass
module top;
  initial begin
    outer::inner i = new; outer t = new; other o = new;
    i.v = '0; i.v.b = 5'h1F;
    $display("T|nest %0d %0d %0d %0d %h", i.w(), $bits(i.v), $bits(t.o), $bits(o.m), i.v);
    $display("T|q %0d %0d %0d", $bits(outer::inner::r), $bits(outer::r), $bits(other::r));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|nest 8 8 12 16 1f", "T|q 8 12 16"]);
}

/// A typedef sized by a class parameter (§8.25), next to another class's `r`.
#[test]
fn parameterized_class_typedef() {
    let src = r#"
class P #(int W = 4);
  typedef logic [W-1:0] r;
  r v;
  function int w(); return $bits(r); endfunction
endclass
class Q;
  typedef struct packed { bit [9:0] f; } r;
  r v;
  function int w(); return $bits(r); endfunction
endclass
module top;
  initial begin
    P #(6) p6 = new; P p4 = new; Q q = new;
    $display("T|param %0d %0d %0d %0d %0d %0d", p6.w(), $bits(p6.v), p4.w(), $bits(p4.v), q.w(), $bits(q.v));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|param 6 6 4 4 10 10"]);
}

/// A base and a derived class each declaring `r`; a derived class that inherits the base's.
#[test]
fn base_and_derived_typedefs() {
    let src = r#"
class base;
  typedef struct packed { bit [3:0] lo; } r;
  r bv;
  function int bw(); return $bits(r); endfunction
endclass
class derived extends base;
  typedef struct packed { bit [7:0] hi; bit [7:0] lo; } r;
  r dv;
  function int dw(); return $bits(r); endfunction
endclass
class derived2 extends base;
  r iv;
  function int iw(); return $bits(r); endfunction
endclass
module top;
  initial begin
    derived d = new; derived2 e = new;
    d.dv = '0; d.dv.hi = 8'hA5; d.bv = '1;
    $display("T|inh %0d %0d %0d %0d %h %h", d.bw(), d.dw(), $bits(d.bv), $bits(d.dv), d.dv, d.bv);
    $display("T|inh2 %0d %0d %0d", e.iw(), $bits(e.iv), $bits(derived::r));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|inh 4 16 4 16 a500 f", "T|inh2 4 4 16"]);
}

/// Same-named enum typedefs (and aliases of them) in two classes.
#[test]
fn class_enum_typedefs() {
    let src = r#"
class A;
  typedef enum logic [1:0] {RED=1, GRN=2} col;
  col c = GRN;
  typedef col alias_t;
  alias_t ac = RED;
  function string nm(); return c.name(); endfunction
endclass
class B;
  typedef enum logic [3:0] {GRN=7, RED=9, BLU=3} col;
  col c = RED;
  typedef col alias_t;
  alias_t ac = BLU;
  function string nm(); return c.name(); endfunction
endclass
module top;
  initial begin
    A a = new; B b = new;
    $display("T|en %0d %0d %s %s %0d %0d", a.c, b.c, a.nm(), b.nm(), a.ac, b.ac);
    $display("T|eb %0d %0d %0d %0d", $bits(A::col), $bits(B::col), $bits(a.ac), $bits(b.ac));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|en 2 9 GRN RED 1 3", "T|eb 2 4 2 4"]);
}

/// Two classes each declaring a nested class `N`.
#[test]
fn nested_classes_of_one_name() {
    let src = r#"
class A;
  class N; int v = 1; function int id(); return 10; endfunction endclass
  N n;
  function new(); n = new; endfunction
endclass
class B;
  class N; int v = 2; logic [7:0] w = 8'h33; function int id(); return 20; endfunction endclass
  N n;
  function new(); n = new; endfunction
endclass
module top;
  initial begin
    A a = new; B b = new;
    A::N an = new; B::N bn = new;
    $display("T|nested %0d %0d %0d %0d", a.n.v, b.n.v, a.n.id(), b.n.id());
    $display("T|q %0d %0d %0d %0d", an.v, bn.v, an.id(), bn.id());
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|nested 1 2 10 20", "T|q 1 2 10 20"]);
}

/// A class typedef beside a package and a `$unit` typedef of the same name.
#[test]
fn class_package_unit_typedefs() {
    let src = r#"
package pk;
  typedef logic [5:0] r;
  class K;
    typedef logic [13:0] r;
    r v;
    function int w(); return $bits(r); endfunction
  endclass
endpackage
typedef logic [2:0] r;
class U;
  r u;
  typedef logic [20:0] t;
  t tt;
endclass
class V;
  typedef logic [7:0] t;
  t tt;
endclass
module top;
  import pk::*;
  initial begin
    K k = new; U u = new; V v = new;
    $display("T|mix %0d %0d %0d %0d %0d %0d", k.w(), $bits(k.v), $bits(pk::r), $bits(u.u), $bits(u.tt), $bits(v.tt));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|mix 14 14 6 3 21 8"]);
}

/// Each class randomizes its own `r` layout.
#[test]
fn randomize_two_class_layouts() {
    let src = r#"
class C1;
  typedef struct packed { bit [1:0] a; bit [3:0] b; } r;
  rand r x;
  constraint c { x.b == 4'h9; x.a == 2'd1; }
endclass
class C2;
  typedef struct packed { bit [11:0] b; bit [3:0] a; } r;
  rand r x;
  constraint c { x.b == 12'hABC; x.a == 4'h7; }
endclass
module top;
  initial begin
    C1 c1 = new; C2 c2 = new; int r1, r2;
    r1 = c1.randomize(); r2 = c2.randomize();
    $display("T|rand %0d %0d %h %h %h %h", r1, r2, c1.x.b, c1.x.a, c2.x.b, c2.x.a);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|rand 1 1 9 1 abc 7"]);
}

/// Out-of-block methods, `pkg::C::r`, casts, arrays, a class-handle typedef and `Base::r` from a derived class.
#[test]
fn qualified_and_out_of_block_uses() {
    let src = r#"
package pk;
  class K;
    typedef struct packed { bit [5:0] a; bit [1:0] b; } r;
    r v;
    static r sv;
    extern function r mk(bit [5:0] x);
    extern function int w(r p);
  endclass
  function K::r K::mk(bit [5:0] x); r t; t = '0; t.a = x; return t; endfunction
  function int K::w(r p); r q[$]; q.push_back(p); return $bits(q[0]) + q.size(); endfunction
endpackage
class L;
  typedef struct packed { bit [11:0] a; } r;
  r arr[2];
  function r cast_it(int x); return r'(x); endfunction
endclass
class M;
  typedef L h_t;
  typedef bit [2:0] r;
  h_t h;
  r m;
  function new(); h = new; endfunction
endclass
class N extends L;
  L::r n;
  function int wn(); return $bits(n) + $bits(L::r); endfunction
endclass
module top;
  initial begin
    pk::K k = new; L l = new; M m = new; N n = new;
    k.v = k.mk(6'h2A); pk::K::sv = k.v;
    $display("T|pk %h %h %0d %0d %0d", k.v, pk::K::sv, k.w(k.v), $bits(pk::K::r), $bits(k.v));
    l.arr[1] = l.cast_it(32'h1234);
    $display("T|L %h %0d %0d", l.arr[1], $bits(l.arr[0]), $bits(L::r));
    $display("T|M %0d %0d %0d", $bits(m.m), $bits(M::r), $bits(m.h.arr[0]));
    $display("T|N %0d", n.wn());
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|pk a8 a8 9 8 8", "T|L 234 12 12", "T|M 3 3 12", "T|N 24"]
    );
}
