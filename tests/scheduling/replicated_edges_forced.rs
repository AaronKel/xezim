//! §10.6.2 / §9.4.2: edge aliases on a vector one `assign v[A +: N] =
//! {N{b}}` replicates (see `replicated_clock_edges`) are shared only while
//! every bit follows that one driver. A procedural `force` of one bit makes
//! it differ from the others, so a vector some `force` targets keeps one
//! alias per bit.

use std::path::PathBuf;
use std::process::Command;

fn run(tag: &str, src: &str, top: &str, env: &[(&str, &str)]) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("replicated_edges_forced");
    std::fs::create_dir_all(&dir).unwrap();
    let sv = dir.join(format!("{tag}.sv"));
    std::fs::write(&sv, src).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_xezim"));
    for (k, v) in env {
        cmd.env(k, v);
    }
    let output = cmd
        .args(["--simulate", "-s", top, "--no-cache", sv.to_str().unwrap()])
        .output()
        .unwrap();
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "run failed:\n{text}");
    text
}

fn t_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|l| l.find("T|").map(|i| &l[i..]))
        .collect()
}

const FORCED: &str = r#"
`timescale 1ns/1ns
module tb;
  logic clk = 0;
  always #5 clk = ~clk;
  wire [3:0] fv;
  assign fv[3:0] = {4{clk}};
  wire [3:0] sv;
  assign sv[3:0] = {4{clk}};
  wire [3:0] fw;
  assign fw = {4{clk}};
  int f [4];
  always @(posedge fv[2]) f[0]++;
  always @(posedge fw[2]) f[1]++;
  always @(fv[2]) f[2]++;
  always @(posedge sv[2]) f[3]++;
  initial begin
    #12 force fv[2] = 1'b0; force fw[2] = 1'b0;
    #23 release fv[2]; release fw[2];
    #28;
    $display("T|force %0d %0d %0d %b", f[0], f[1], f[2], fv);
    $finish;
  end
  initial begin #20 $display("T|mid %b %b", fv, fw); end
endmodule
"#;

#[test]
fn forced_vector_keeps_an_alias_per_bit() {
    let o = run("forced", FORCED, "tb", &[("XEZIM_DUMP_EDGE_SENS", "1")]);
    assert!(
        !o.contains("fv[2] shares"),
        "a forced vector must keep its own aliases:\n{o}"
    );
    assert!(
        o.contains("sv[2] shares the alias of bit 0"),
        "an unforced replicated vector still shares:\n{o}"
    );
    // The whole-vector form already follows the force.
    assert!(
        t_lines(&o)
            .iter()
            .any(|l| l.starts_with("T|force ") && l.split(' ').nth(2) == Some("4")),
        "{o}"
    );
}
