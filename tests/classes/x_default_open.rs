//! Open initial-value and default gaps found next to the aggregate class
//! property x defaults (all ignored). Expected values come from the
//! reference simulator.

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
#[ignore = "§7.10.1: a class queue read past its end gives a 32-bit 0, not the element type's default"]
fn class_queue_read_past_the_end() {
    let src = r#"
typedef struct {logic [3:0] d; bit [3:0] f; integer n;} rec_t;
class holder; logic [3:0] dq[$]; rec_t rq[$]; endclass
module top;
  holder h;
  initial begin
    h = new;
    $display("T|cls_q %b %0d %b %0d", h.dq[3], h.rq[1].n === 'x, h.rq[1].d, h.rq[1].f);
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|cls_q xxxx 1 xxxx 0"]);
}

#[test]
#[ignore = "§7.9.11: a class associative-array property's `'{default: v}` initializer is dropped"]
fn class_assoc_default_initializer() {
    let src = r#"
class holder; logic [7:0] wa[int] = '{default: 8'h5A}; endclass
module top;
  holder h;
  initial begin
    h = new;
    $display("T|cls_def %h %h %0d", h.wa[9], h.wa[1], h.wa.num());
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|cls_def 5a 5a 0"]);
}

#[test]
#[ignore = "§7.8.6: a missing element of a class two-dimensional associative array reads a 32-bit 0"]
fn class_two_dimensional_assoc_missing_element() {
    let src = r#"
class holder; logic [3:0] la2[int][int]; endclass
module top;
  holder h;
  initial begin
    h = new;
    $display("T|cls_2d %b %0d", h.la2[1][2], h.la2.num());
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|cls_2d xxxx 0"]);
}

#[test]
#[ignore = "§7.8.6/§7.10.1: a 2-state member of a missing struct element of a module associative array or queue reads x, not 0"]
fn module_struct_collection_defaults() {
    let src = r#"
typedef struct {logic [3:0] d; bit [3:0] f; integer n;} rec_t;
module top;
  rec_t mra[string]; rec_t mq[$];
  initial begin
    $display("T|mod %b %b %0d", mra["x"].d, mra["x"].f, mra["x"].n === 'x);
    $display("T|modq %b %b %0d", mq[0].d, mq[0].f, mq.size());
  end
endmodule
"#;
    assert_eq!(t_lines(src), ["T|mod xxxx 0000 1", "T|modq xxxx 0000 0"]);
}

#[test]
#[ignore = "§21.2.1.7: %p of a class struct property holding unpacked arrays and a handle prints each member as one x"]
fn percent_p_of_struct_property_with_arrays() {
    let src = r#"
class inner; int v = 7; endclass
typedef struct {logic [3:0] arr[3]; bit [3:0] barr[2]; inner h; integer n;} box_t;
class holder; box_t box; endclass
module top;
  holder h;
  initial begin
    h = new;
    $display("T|cls_p %p", h.box);
  end
endmodule
"#;
    assert_eq!(
        t_lines(src),
        ["T|cls_p '{arr:'{x, x, x}, barr:'{0, 0}, h:null, n:x}"]
    );
}

#[test]
#[ignore = "§6.8: a static unpacked-struct class property reads 1-bit x members instead of its member defaults"]
fn static_unpacked_struct_property() {
    let src = r#"
typedef struct {logic [3:0] d; bit [3:0] f;} rec_t;
class sc; static rec_t sr; endclass
module top;
  initial $display("T|static %b %b", sc::sr.d, sc::sr.f);
endmodule
"#;
    assert_eq!(t_lines(src), ["T|static xxxx 0000"]);
}

#[test]
#[ignore = "§7.2.1: a packed struct with a 4-state member starts all x and converts its 2-state members on read; its bit fields start 0"]
fn mixed_packed_struct_starts_unknown() {
    let src = r#"
typedef struct packed {bit [3:0] b; logic [3:0] l;} mix_t;
interface ifc; mix_t m; endinterface
module top;
  ifc u();
  mix_t mx;
  initial $display("T|mix %b %b %b %b", mx, mx.b, mx.l, u.m);
endmodule
"#;
    assert_eq!(t_lines(src), ["T|mix xxxxxxxx 0000 xxxx xxxxxxxx"]);
}

#[test]
#[ignore = "§13.4.1: the unassigned return value of a function of unpacked struct type reads x for its 2-state members"]
fn function_struct_return_default() {
    let src = r#"
typedef struct {logic [3:0] d; bit [3:0] f;} rec_t;
module top;
  function automatic rec_t fr(); endfunction
  initial $display("T|fret %b %b", fr().d, fr().f);
endmodule
"#;
    assert_eq!(t_lines(src), ["T|fret xxxx 0000"]);
}

#[test]
#[ignore = "§24.3: the initial blocks of an instantiated program run in the active region, before module initial blocks"]
fn instantiated_program_initial_runs_reactive() {
    let src = r#"
program prg;
  logic [3:0] pv; int pi;
  initial $display("T|prog %b %0d", pv, pi);
endprogram
module top;
  prg pp();
  initial $display("T|mod");
endmodule
"#;
    assert_eq!(t_lines(src), ["T|mod", "T|prog xxxx 0"]);
}
