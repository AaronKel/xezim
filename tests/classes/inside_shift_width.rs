//! IEEE 1800 §11.4.10/§11.4.13: oversized shifts and joint inside types.
use xezim::simulate;

fn check(source: &str) {
    let sim = simulate(source, 1000).unwrap();
    let output = sim
        .output
        .iter()
        .map(|o| o.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        output.contains("PASS") && !output.contains("Fatal"),
        "{output}"
    );
}
#[test]
fn inside_shift_to_full_unsigned_address_range() {
    check(include_str!("../fixtures/inside_shift_width.sv"));
}
#[test]
fn oversized_shifts_keep_logical_zero_and_arithmetic_sign_fill() {
    check(
        r#"
class item;
 rand longint signed a;
 rand longint unsigned b;
 constraint c { a == (-64'sd1 >>> 64); b == (64'hffff_ffff_ffff_ffff >> 64); }
endclass
module top; initial begin item x=new();
 if(!x.randomize())$fatal(1,"shift solve failed");
 if(x.a!=-1 || x.b!=0)$fatal(1,"shift fill wrong");
 $display("PASS");$finish;end endmodule
"#,
    );
}
#[test]
fn scope_randomize_uses_unsigned_inside_context() {
    check(
        r#"
module top;bit[63:0] x;int width=64;initial begin
 if(!std::randomize(x) with {x inside {[0:((1<<width)-1)]}; x==64'hfedcba9876543210;})$fatal(1,"scope solve failed");
 if(x!=64'hfedcba9876543210)$fatal(1,"scope value wrong");
 $display("PASS");$finish;end endmodule
"#,
    );
}

#[test]
fn procedural_inside_sizes_64_and_128_bit_integer_bounds_and_preserves_reals() {
    check(include_str!("../fixtures/inside_procedural_width.sv"));
}
