//! §20.6.1: `$typename` of a real-valued expression is `real`. A time
//! literal is realtime (§5.8), so `$typename(1ns)` is `real` too (#324,
//! reported by Dragon-git); it, and every other expression that was not a
//! bare name, read `logic`. §6.12 makes `realtime` a synonym for `real`, and
//! the reference simulator names it so. Expected lines are the reference
//! simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 100).expect("simulate failed");
    sim.output
        .iter()
        .flat_map(|o| o.message.lines())
        .filter(|l| l.starts_with("T|"))
        .map(str::to_string)
        .collect()
}

#[test]
fn typename_of_real_expressions_is_real() {
    const SRC: &str = r#"`timescale 1ns/1ps
module top;
  typedef realtime rt_t;
  real r; realtime rt; shortreal sr; int i; rt_t v;
  function real fr(); return 1; endfunction
  initial begin
    $display("T|lit 1ns=%s 1.5=%s 1ps=%s 2.5ns=%s neg=%s", $typename(1ns), $typename(1.5), $typename(1ps), $typename(2.5ns), $typename(-1.5));
    $display("T|ops r2=%s div=%s plus=%s cond=%s", $typename(r*2), $typename(7ns/2ns), $typename(1ns + 1), $typename(1 ? 1.5 : 2.5));
    $display("T|calls fr=%s itor=%s", $typename(fr()), $typename($itor(3)));
    $display("T|vars r=%s rt=%s sr=%s i=%s v=%s kw=%s td=%s", $typename(r), $typename(rt), $typename(sr), $typename(i), $typename(v), $typename(realtime), $typename(rt_t));
    $display("T|values 7ns/2ns=%f t/2ns=%f 1ns=%f", 7ns/2ns, rt/2ns, 1ns);
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|lit 1ns=real 1.5=real 1ps=real 2.5ns=real neg=real",
            "T|ops r2=real div=real plus=real cond=real",
            "T|calls fr=real itor=real",
            "T|vars r=real rt=real sr=shortreal i=int v=real kw=real td=real",
            "T|values 7ns/2ns=3.500000 t/2ns=0.000000 1ns=1.000000",
        ]
    );
}

/// The type of an INTEGRAL expression (`$typename(7)`, `$typename(i + 1)`)
/// and the unpacked dimensions of an array variable. Both need the
/// self-determined type of an arbitrary expression (§11.6, §11.8.1), which
/// `$typename` does not infer: anything but a declared name reads `logic`,
/// and an array reads as its element type.
#[test]
#[ignore = "$typename does not infer integral expression types or render unpacked dimensions"]
fn typename_of_integral_expressions_and_arrays() {
    const SRC: &str = r#"module top;
  int ia[2]; logic [7:0] la[0:3]; real ra[3]; byte b; logic [7:0] l; int i;
  initial begin
    $display("T|ia=%s la=%s ra=%s", $typename(ia), $typename(la), $typename(ra));
    $display("T|e7=%s ip1=%s bp1=%s bb=%s lp1=%s h=%s b1=%s cat=%s", $typename(7), $typename(i+1), $typename(b+1), $typename(b+b), $typename(l+1), $typename(8'hff), $typename(1'b1), $typename({l,l}));
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|ia=int$[0:1] la=reg[7:0]$[0:3] ra=real$[0:2]",
            "T|e7=bit signed[31:0] ip1=int bp1=bit signed[31:0] bb=byte lp1=reg[31:0] h=bit[7:0] b1=bit cat=reg[15:0]",
        ]
    );
}
