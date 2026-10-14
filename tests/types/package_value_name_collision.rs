//! §26.3 / §26.6 — value names (subroutines, parameters, `let`s) and
//! covergroups declared by more than one package, the neighbours of the
//! type-name collisions of #300.
//!
//! Package functions, tasks, parameters and `let`s share flat tables keyed
//! by the bare name, where the package registered last wins. A package that
//! imported the other one's name (`import pb::*; ... f()`) and a name
//! re-exported by a third package (`export pb::*;`, `import pa::P; export
//! pa::P;`) read the wrong declaration. Bare references are now spelled
//! `pkg::name` by the §26.3 rules; a package covergroup declared twice is
//! renamed like a type. Also: a wildcard import makes a package `let`
//! visible, and the statics of a class whose name another package also
//! declares start at their initializers.
//!
//! Expected values come from the reference simulator.

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
fn imported_and_reexported_subroutines_and_parameters() {
    let src = r#"
package pa;
  typedef logic [3:0] n_t;
  parameter int P = 3;
  function automatic int f(); return 1; endfunction
endpackage
package pb;
  typedef logic [7:0] n_t;
  parameter int P = 5;
  function automatic int f(); return 2; endfunction
endpackage
package pc;
  import pb::*;
  export pb::*;
  n_t keep;
  int u = P;
  function automatic int g(); return f() * 10; endfunction
endpackage
package pd;
  import pa::n_t; import pa::P; import pa::f;
  export pa::n_t; export pa::P; export pa::f;
endpackage
module top;
  import pc::*;
  n_t x;
  pd::n_t y;
  initial begin
    $display("T|bits %0d %0d", $bits(x), $bits(y));
    $display("T|p %0d %0d %0d", P, pd::P, pc::u);
    $display("T|f %0d %0d %0d", f(), pd::f(), pc::g());
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|bits 8 4", "T|p 5 3 5", "T|f 2 1 20"],
        "T| lines differ"
    );
}

#[test]
fn package_body_reads_its_import_not_the_last_package() {
    let src = r#"
package pa;
  parameter int P = 3;
  function automatic int f(); return 1; endfunction
endpackage
package pb;
  parameter int P = 5;
  function automatic int f(); return 2; endfunction
endpackage
package pc;
  import pb::*;
  function automatic int g(); return f() * 10; endfunction
  int u = P;
endpackage
module top;
  initial $display("T|g %0d %0d", pc::g(), pc::u);
endmodule
"#;
    assert_eq!(t_lines(src), ["T|g 20 5"]);
}

#[test]
fn package_lets_by_wildcard_and_qualified() {
    let src = r#"
package pa;
  let mx(a, b) = (a > b) ? a : b;
  let sc(a) = a * 2;
endpackage
package pb;
  let mx(a, b) = (a < b) ? a : b;
  let sc(a) = a * 3;
endpackage
package pu;
  let tw(a) = a + 40;
endpackage
module top;
  import pb::*;
  import pu::*;
  initial begin
    $display("T|q %0d %0d", pa::mx(3, 5), pb::mx(3, 5));
    $display("T|s %0d %0d", pa::sc(7), pb::sc(7));
    $display("T|wild %0d %0d", sc(7), tw(2));
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|q 5 3", "T|s 14 21", "T|wild 21 42"]);
}

#[test]
fn same_named_package_covergroups() {
    let src = r#"
package pa;
  class C;
    rand bit [3:0] v;
    covergroup cg; coverpoint v { bins lo = {[0:7]}; } endgroup
    function new(); cg = new; endfunction
  endclass
  covergroup pcg with function sample(int x); coverpoint x { bins a = {1}; bins b = {2}; } endgroup
endpackage
package pb;
  class C;
    rand bit [3:0] v;
    covergroup cg; coverpoint v { bins lo = {[0:3]}; bins hi = {[12:15]}; } endgroup
    function new(); cg = new; endfunction
  endclass
  covergroup pcg with function sample(int x); coverpoint x { bins a = {1}; bins b = {2}; bins c = {3}; bins d = {4}; } endgroup
endpackage
module top;
  pa::C ca = new; pb::C cb = new;
  pa::pcg ga = new; pb::pcg gb = new;
  initial begin
    ca.v = 2; ca.cg.sample(); cb.v = 2; cb.cg.sample();
    ga.sample(1); gb.sample(1);
    $display("T|cls %0.2f %0.2f", ca.cg.get_inst_coverage(), cb.cg.get_inst_coverage());
    $display("T|pkg %0.2f %0.2f", ga.get_inst_coverage(), gb.get_inst_coverage());
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|cls 100.00 50.00", "T|pkg 50.00 25.00"]);
}

#[test]
fn statics_of_same_named_classes() {
    let src = r#"
package pa;
  class C;
    static int cnt = 1;
    static function int sf(); return cnt + 100; endfunction
    function new(); cnt++; endfunction
  endclass
  class P #(int W = 2);
    static int s = W;
    static function int get(); return s * 10; endfunction
  endclass
endpackage
package pb;
  class C;
    static int cnt = 50;
    static function int sf(); return cnt + 200; endfunction
    function new(); cnt += 5; endfunction
  endclass
  class P #(int W = 3);
    static int s = W + 1;
    static function int get(); return s * 100; endfunction
  endclass
endpackage
module top;
  initial begin
    pa::C a; pb::C b;
    $display("T|init %0d %0d", pa::C::cnt, pb::C::cnt);
    a = new; b = new; b = new;
    $display("T|cnt %0d %0d", pa::C::cnt, pb::C::cnt);
    $display("T|sf %0d %0d", pa::C::sf(), pb::C::sf());
    $display("T|ps %0d %0d %0d %0d", pa::P#()::s, pb::P#()::s, pa::P#(5)::get(), pb::P#(5)::get());
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        [
            "T|init 1 50",
            "T|cnt 2 60",
            "T|sf 102 260",
            "T|ps 2 4 50 600"
        ]
    );
}
