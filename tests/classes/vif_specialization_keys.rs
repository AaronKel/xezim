//! §25.9/§8.25/§6.22.1: a parameterized class specialized with a virtual
//! interface type keys its statics by that TYPE, however it is spelled:
//! with the enclosing class's parameters (`db#(virtual my_if#(T))` inside
//! `mon#(int)`), through a class-local or module typedef, with the
//! `interface` keyword, named or defaulted interface parameters, or reached
//! through `extends db#(virtual my_if#(T))`. The argument text used to be
//! keyed verbatim, so each spelling had its own empty table (issue #320,
//! reported by Ganesh-TNL). Expected values come from the reference
//! simulator.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    simulate(src, 1000)
        .expect("simulate failed")
        .output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn vif_spec_keys_through_class_params() {
    let expected = [
        "T|m_size_vifT|1",
        "T|m_size_vif_t|1",
        "T|m_size_vif_int|1",
        "T|m_size_T|1",
        "T|m_cnt_vifT|1",
        "T|m_size_vifTN|1",
        "T|m_size_typedef|1",
        "T|mb_size_vifT|1",
        "T|mb_size_vif_t|1",
        "T|mb_size_vif_int|2",
        "T|mb_size_T|0",
        "T|mb_cnt_vifT|1",
        "T|mb_size_vifTN|1",
        "T|mb_size_typedef|2",
        "T|m8_size_vifT|2",
        "T|m8_size_vif_t|2",
        "T|m8_size_vif_int|2",
        "T|m8_size_T|1",
        "T|m8_cnt_vifT|2",
        "T|m8_size_vifTN|1",
        "T|m8_size_typedef|2",
        "T|child_size_vifT|3",
        "T|child_size_vif_t|3",
        "T|child_size_vif_int|3",
        "T|child_size_T|1",
        "T|child_cnt_vifT|3",
        "T|child_size_vifTN|3",
        "T|child_size_typedef|3",
        "T|wrap|4",
        "T|wrapm|1",
        "T|mod_after|4",
        "T|mod_after_typedef|4",
    ];
    assert_eq!(
        t_lines(include_str!("vif_spec_keys_class_params.sv")),
        expected
    );
}

#[test]
fn vif_spec_keys_across_spellings() {
    let expected = [
        "T|noargs|1",
        "T|kw|1",
        "T|named|1",
        "T|named_both|1",
        "T|full|1",
        "T|other_n|0",
        "T|other_t|0",
        "T|kw_typedef|1",
        "T|modport|0",
        "T|modport_typedef|0",
        "T|after_inst|2",
        "T|inst_view|2",
        "T|after_ext|3",
        "T|after_ext2|4",
        "T|ext2_view|4",
    ];
    assert_eq!(
        t_lines(include_str!("vif_spec_keys_spellings.sv")),
        expected
    );
}

/// Objects declared `db#(virtual my_if#(T))` (a property, a typedef'd
/// local) inside `mon#(int)` share `db#(virtual my_if#(int))`'s statics.
#[test]
fn vif_spec_keys_of_declared_objects() {
    let expected = [
        "T|int_after_m|3",
        "T|byte_after_m|0",
        "T|int_after_mb|3",
        "T|byte_after_mb|3",
    ];
    assert_eq!(t_lines(include_str!("vif_spec_keys_declared.sv")), expected);
}

/// §8.25: value parameters match by VALUE — `P#(2+2)` is `P#(4)`. The
/// specialization text is keyed unevaluated.
#[test]
#[ignore = "specialization args keyed as unevaluated text: P#(2+2) is not P#(4)"]
fn spec_value_args_match_by_value() {
    const SRC: &str = r#"
class P #(int N = 4); static int c; endclass
interface vi_if #(int W = 4); logic [W-1:0] d; endinterface
class D #(type T = int); static int c; endclass
module tb;
  initial begin
    P#(4)::c++;
    $display("T|cls_expr|%0d", P#(2+2)::c);
    D#(virtual vi_if#(4))::c++;
    $display("T|vif_expr|%0d", D#(virtual vi_if#(2+2))::c);
    $display("T|vif_hex|%0d", D#(virtual vi_if#('h4))::c);
    $finish;
  end
endmodule
"#;
    let expected = ["T|cls_expr|1", "T|vif_expr|1", "T|vif_hex|1"];
    assert_eq!(t_lines(SRC), expected);
}

/// §6.22.1: two typedefs of the same queue type (`int[$]`, one through a
/// class type parameter) are the same specialization argument.
#[test]
#[ignore = "typedefs of unpacked array types are keyed by name, not by type"]
fn spec_typedef_queue_args_match() {
    const SRC: &str = r#"
class db #(type T = int);
  static int cnt;
endclass
typedef int intq_t[$];
class mon #(type T = int);
  typedef T tq_t[$];
  function int n(); return db#(tq_t)::cnt; endfunction
endclass
module tb;
  initial begin
    mon#(int) m = new;
    db#(intq_t)::cnt++;
    $display("T|typedef_queue|%0d", m.n());
    $finish;
  end
endmodule
"#;
    let expected = ["T|typedef_queue|1"];
    assert_eq!(t_lines(SRC), expected);
}

/// §25.9/§8.9: a virtual interface held in a STATIC class property (plain,
/// queue element, function return, `static T` of `G#(virtual pif)`).
#[test]
#[ignore = "virtual interface values are not stored in static class properties"]
fn static_class_vif_properties() {
    const SRC: &str = r#"
interface pif; int data; endinterface
class S;
  static virtual pif spv;
  static virtual pif pq[$];
  static function virtual pif getp(); return spv; endfunction
endclass
class G #(type T = int);
  static T one;
  static function void put(T v); one = v; endfunction
endclass
module tb;
  pif pi();
  initial begin
    virtual pif p;
    pi.data = 9;
    S::spv = pi; p = S::spv; $display("T|static_vif|%0d", p.data);
    p = S::getp(); $display("T|static_fn_ret|%0d", p.data);
    S::pq.push_back(pi); p = S::pq[0]; $display("T|static_q|%0d", p.data);
    G#(virtual pif)::put(pi);
    p = G#(virtual pif)::one; $display("T|gen_one|%0d", p.data);
    $finish;
  end
endmodule
"#;
    let expected = [
        "T|static_vif|9",
        "T|static_fn_ret|9",
        "T|static_q|9",
        "T|gen_one|9",
    ];
    assert_eq!(t_lines(SRC), expected);
}
