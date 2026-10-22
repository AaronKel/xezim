//! A CLASS-FIELD initializer that READS another field must see that field's
//! computed value, not its default.
//!
//! `construction.rs` evaluates property initializers against a half-built
//! object: the ones containing a call are deferred to construction time and
//! evaluated once each in declaration order, but a plain initializer such as
//! `string h_derived = {h, ".derived"};` is captured at declaration time while
//! `h` is still `""`, so the derived field is born as `".derived"`.  The same
//! hole covers `int r_plus1 = r + 1;` over a `$urandom_range` field, which comes
//! out as 1.  The settlement pass re-evaluates the plain fields that (transitively)
//! read a deferred field, each exactly once, and leaves everything else alone.
//! Verified against the reference simulator.

fn lines(src: &str, top: &str) -> Vec<String> {
    xezim::simulate_multi(
        &[src.to_string()],
        1000,
        Some(top),
        &[],
        &[],
        None,
        false,
        None,
        None,
        &[],
        &[],
        None,
        &[],
        0,
        u64::MAX,
        None,
        &[],
        None,
        None,
        None,
        None,
        false,
    )
    .expect("sim")
    .output
    .iter()
    .map(|o| o.message.clone())
    .collect()
}

const SRC: &str = r#"
class k_c;
  string lit       = "literal";                  // no dependency
  string lit2      = {lit, "-tail"};             // reads a plain field
  int    r         = $urandom_range(1, 9999);    // deferred, no dependency
  string h         = $sformatf("%m");             // deferred
  string h_derived = {h, ".derived"};            // one hop
  string h_deep    = {h_derived, ".deep"};       // two hops
  int    r_plus1   = r + 1;                      // integer chain
  function new();
  endfunction
  function string self_derived();
    return {this.h, ".derived"};                 // `this.` read of a deferred field
  endfunction
endclass

module k_leaf;
  k_c obj = new();
  initial begin
    #1;
    $display("LIT=%s",      obj.lit);
    $display("LIT2=%s",     obj.lit2);
    $display("R=%0d",       obj.r);
    $display("H=%s",        obj.h);
    $display("DERIVED=%s",  obj.h_derived);
    $display("DEEP=%s",     obj.h_deep);
    $display("RPLUS1=%0d",  obj.r_plus1);
    $display("THIS_DERIVED=%s", obj.self_derived());
    $finish;
  end
endmodule

module tb;
  k_leaf u_leaf();
endmodule
"#;

/// Fields derived from a deferred field — one hop, two hops, and the integer
/// chain — hold the dependency's value, not the default captured at
/// declaration time.
#[test]
fn class_field_reader_of_deferred_field_is_re_evaluated() {
    let out = lines(SRC, "tb").join("\n");
    let get = |k: &str| -> String {
        out.lines()
            .find(|l| l.starts_with(&format!("{k}=")))
            .and_then(|l| l.split_once('=').map(|(_, v)| v.to_string()))
            .unwrap_or_else(|| panic!("{k} printed: {out}"))
    };
    let h = get("H");
    assert!(!h.is_empty(), "$sformatf(\"%m\") folded to \"\": {out}");
    assert_eq!(get("DERIVED"), format!("{h}.derived"), "{out}");
    assert_eq!(get("DEEP"), format!("{h}.derived.deep"), "{out}");
    let r: i64 = get("R").parse().expect("R numeric");
    let rp: i64 = get("RPLUS1").parse().expect("RPLUS1 numeric");
    assert!((1..=9999).contains(&r), "$urandom_range folded: {out}");
    assert_eq!(rp, r + 1, "r_plus1 kept the folded dependency: {out}");
    // A method reading `this.h` runs after construction, so it is the oracle
    // for what the field initializer must have captured.
    assert!(
        out.contains(&format!("THIS_DERIVED={}", get("DERIVED"))),
        "`this.h` and the field initializer disagree: {out}"
    );
}

/// Fields with no deferred dependency keep their declaration-order value:
/// literals and a field built from another plain field.
#[test]
fn plain_class_field_initializers_are_untouched() {
    let out = lines(SRC, "tb").join("\n");
    assert!(out.contains("LIT=literal"), "{out}");
    assert!(out.contains("LIT2=literal-tail"), "{out}");
}
#[test]
fn field_initialization_call_after_plain() {
    let source = r#"class C;
 int root=$urandom_range(7,7);
 int derived=root+1;
 int snapshot=take(derived);
 function int take(int n);return n;endfunction
endclass
module top;initial begin C c=new;$display("T|calls %0d %0d %0d",c.root,c.derived,c.snapshot);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|calls 7 8 8"]);
}

#[test]
fn field_initialization_side_effect() {
    let source = r#"package p;int calls=0;endpackage
class C;
 int root=$urandom_range(7,7);
 int derived=root+p::calls++;
endclass
module top;initial begin C c=new;$display("T|effects %0d %0d %0d",c.root,c.derived,p::calls);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|effects 7 7 1"]);
}

#[test]
fn field_initialization_cast() {
    let source = r#"class C;
 int root=$urandom_range(7,7);
 int derived=int'(root)+1;
endclass
module top;initial begin C c=new;$display("T|cast %0d %0d",c.root,c.derived);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|cast 7 8"]);
}

#[test]
fn field_initialization_range() {
    let source = r#"class C;
 int root=$urandom_range(7,7);
 int derived=root[7:0]+1;
endclass
module top;initial begin C c=new;$display("T|range %0d %0d",c.root,c.derived);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|range 7 8"]);
}

#[test]
fn field_initialization_multiple_dependencies() {
    let source = r#"class C;
 int root=$urandom_range(7,7);
 int aa=root+1;
 int bb=aa+1;
 int cc=bb+1;
 int dd=cc+1;
 int ee=dd+1;
 int ff=ee+1;
 int gg=ff+1;
 int hh=gg+1;
 int ii=hh+1;
 int jj=ii+1;
 int kk=jj+1;
 int ll=kk+1;
 int mm=ll+1;
 int nn=mm+1;
 int result=root+nn;
endclass
module top;initial begin C c=new;$display("T|multiple %0d %0d %0d",c.root,c.nn,c.result);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|multiple 7 21 28"]);
}

#[test]
fn field_initialization_explicit_this() {
    let source = r#"class C;
 int root=$urandom_range(7,7);
 int derived=this.root+1;
endclass
module top;initial begin C c=new;$display("T|this %0d %0d",c.root,c.derived);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|this 7 8"]);
}

#[test]
fn field_initialization_inherited() {
    let source = r#"class B;
 int root=$urandom_range(7,7);
endclass
class C extends B;
 int derived=root+1;
endclass
module top;initial begin C c=new;$display("T|inherit %0d %0d",c.root,c.derived);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|inherit 7 8"]);
}

#[test]
fn field_initialization_function_result_types() {
    let source = r#"package p;function int take();return -1;endfunction endpackage
class C;
 bit [3:0] narrow=p::take();
 int unsigned u=p::take();
 int signed s=p::take();
 int snapshot=narrow+1;
endclass
module top;initial begin C c=new;$display("T|fit %0d %0d %0d %0d",c.narrow,c.u,c.s,c.snapshot);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|fit 15 4294967295 -1 16"]);
}

#[test]
fn field_initialization_plain_context_width() {
    let source = r#"class C;
 int sum=1'b1+1'b1;
 int integral=1.6;
 real fractional=2;
endclass
module top;initial begin C c=new;$display("T|context %0d %0d %0.1f",c.sum,c.integral,c.fractional+0.5);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|context 2 2 2.5"]);
}

#[test]
fn field_initialization_specialized_state() {
    let source = r#"class C#(type T=int);
 T v='x;
 T next=v;
endclass
module top;initial begin C#(int) a=new;C#(integer) b=new;
 $display("T|state %0d %0d %0d %0d",$isunknown(a.v),$isunknown(a.next),$isunknown(b.v),$isunknown(b.next));end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|state 0 0 1 1"]);
}

#[test]
fn field_initialization_constructor_call_count() {
    let source = r#"package p;int calls=0;function int tick();calls++;return calls;endfunction endpackage
class C;
 int first=p::tick();
 int copied=first+1;
 int last=p::tick()+copied;
endclass
module top;initial begin C a=new;C b=new;
 $display("T|count %0d %0d %0d %0d %0d %0d %0d",a.first,a.copied,a.last,b.first,b.copied,b.last,p::calls);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|count 1 2 4 3 4 8 4"]);
}

#[test]
fn field_initialization_array_call_count() {
    let source = r#"package p;int calls=0;function int tick();calls++;return calls;endfunction endpackage
class C;
 int a[2]='{p::tick(),p::tick()};
 int copied=a[0]+a[1];
endclass
module top;initial begin C c=new;
 $display("T|arraycalls %0d %0d %0d %0d",c.a[0],c.a[1],c.copied,p::calls);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|arraycalls 1 2 3 2"]);
}

#[test]
fn field_initialization_hidden_base() {
    let source = r#"class B;
 int root=$urandom_range(7,7);
 int derived=root+1;
 function int base_value();return derived;endfunction
endclass
class C extends B;
 int root=$urandom_range(20,20);
 int derived=root+2;
endclass
module top;initial begin C c=new;
 $display("T|hidden %0d %0d",c.base_value(),c.derived);end endmodule
"#;
    let got = lines(source, "top")
        .into_iter()
        .filter(|s| s.starts_with("T|"))
        .collect::<Vec<_>>();
    assert_eq!(got, ["T|hidden 8 22"]);
}
