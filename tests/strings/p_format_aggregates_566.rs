//! IEEE 1800-2023 §21.2.1.7: `%p` of every aggregate shape (#566, reported
//! by Ganesh-TNL with a three-simulator comparison). The `'{...}` form is
//! printed in every context — a class property through a handle, `this` or
//! bare, a static or specialized-class property, an interface or
//! virtual-interface member, an element of a queue, array or associative
//! array of structs, a queue slice, a function result and a task formal or
//! local — where each printed one flat decimal; an enum member prints its
//! label, also for an enum declared in a package.
//!
//! Member radix (decimal), the packed-union member set, `%0p` and the space
//! before an associative array's closing brace stay as the project's
//! reference simulator prints them (the `lrm` suite's goldens); the issue's
//! majority vote on those is an open policy question.

use xezim::simulate;

const SRC: &str = r#"

package mwe_pkg;
  // Shape of an internal IP config struct (member names redacted),
  // single-dimension members, all `bit`.
  typedef struct packed {
    bit [3:0]  f01;
    bit [12:0] f02;
    bit [3:0]  f03;
    bit [13:0] f04;
    bit [31:0] f05;
    bit [5:0]  f06;
    bit [2:0]  f07;
    bit [39:0] f08;
    bit [5:0]  f09;
    bit [12:0] f10;
    bit f11;
    bit f12;
    bit f13;
    bit f14;
  } cfg_params_t;

  // Kitchen-sink packed struct: every integral member flavor.
  typedef struct packed {
    bit [3:0]    a;
    int          b;
    int unsigned ub;
    shortint     sh;
    longint      lg;
    integer      it;
    byte         y;
    logic [7:0]  l;
    reg [7:0]    r;
  } kitchen_ps_t;

  typedef struct packed { logic [3:0] hi; logic [3:0] lo; } small_ps_t;
  typedef struct packed { small_ps_t inner; logic [7:0] tail; } nested_ps_t;
  typedef enum logic [2:0] { A, B, C } e_t;
  typedef struct packed { e_t e; logic [3:0] x; } enum_member_ps_t;
  typedef struct packed { logic [63:0] w; bit [3:0] t; } wide_ps_t;
  typedef struct packed { bit [7:0] [3:0] dom; logic [3:0] t; } arrmem_ps_t;
  typedef union packed { logic [7:0] b; small_ps_t p; } pu_t;

  // Unpacked counterparts.
  typedef struct {
    int a; shortint sh; longint lg; integer it; int unsigned ub;
    byte y; bit [7:0] bv; logic [7:0] lv; string s;
  } kitchen_us_t;
  typedef struct { real r; e_t e; } re_us_t;
  typedef struct { small_ps_t p; int arr[2]; } nest_us_t;
  typedef union { int i; real r; } uu_t;
endpackage

import mwe_pkg::*;

interface mwe_if;
  import mwe_pkg::*;
  cfg_params_t cfg_arr [0:3];
  small_ps_t sarr [0:1];
  cfg_params_t single;
endinterface

// Original shape: class holding a packed-struct property as a plain property.
class seq_base;
  cfg_params_t cfg;
  small_ps_t sml;
  kitchen_us_t unpacked;
  cfg_params_t cfg_q[$];
  cfg_params_t cfg_arr[0:1];
  static small_ps_t sstat;
  function void show();
    $display("CHK CL.02-this|%p", this.cfg);
    $display("CHK CL.03-bare|%p", cfg);          // exact original stmt
    $display("CHK CL.04-unpacked|%p", unpacked);
    $display("CHK CL.05-q|%p", cfg_q);
    $display("CHK CL.06-qelem|%p", cfg_q[1]);
    $display("CHK CL.07-arr|%p", cfg_arr);
    $display("CHK CL.08-arrelem|%p", cfg_arr[1]);
    $display("CHK CL.12-member|%0d", cfg.f06);
  endfunction
endclass

// Specialized class: static + instance property of a type-parameter type.
class gen #(type T = small_ps_t);
  static T sprop;
  T prop;
endclass

module t;
  import mwe_pkg::*;
  mwe_if tc_vif();
  virtual mwe_if vif;
  seq_base seq;
  gen #(small_ps_t) g1;

  cfg_params_t cfg_a, cfg_b;
  kitchen_ps_t    kx;
  small_ps_t      sp, spz, spx;
  nested_ps_t     np;
  enum_member_ps_t ep;
  wide_ps_t       wp;
  arrmem_ps_t     dm;
  pu_t            pu;
  kitchen_us_t    us;
  re_us_t         ru;
  nest_us_t       nu;
  uu_t            uu;
  small_ps_t      sarr [0:1];
  byte            barr [0:1];
  bit [7:0]       varr [0:1];
  int             iarr [0:1];
  int             m2 [2][2];
  small_ps_t      sq[$];
  small_ps_t      sd[];
  small_ps_t      sa[int];

  function automatic small_ps_t mk(input logic [7:0] v);
    mk = v;
  endfunction

  task automatic show_formal(input cfg_params_t cp);
    cfg_params_t loc;
    loc = cp;
    $display("CHK FN.02|%p", cp);
    $display("CHK FN.03|%p", loc);
  endtask

  initial begin
    // ---- values -----------------------------------------------------------
    cfg_a = '{3, 'h1130, 9, 63, 1447, 0, 'h5, 508360320, 5, 128, 1, 0, 1, 0};
    cfg_b = '{5, 'h2244, 7, 33,  999, 3, 'ha, 12345,     2,  64, 0, 1, 0, 1};
    kx   = '{3, -5, 300, -2, 'h123456789, -7, -1, 'h5a, 'h5b};
    sp   = 8'h12;
    spz  = '0;
    spx  = 8'b10xz0101;
    np   = '{8'hab, 'hcd};
    ep   = '{B, 'h5};
    wp   = '{64'hdeadbeefcafe1234, 'hf};
    dm   = '{32'h12345678, 'hf};
    pu.b = 8'ha5;
    us   = '{1, 2, 3, 4, 5, 6, 'h5a, 'h5b, "str"};
    ru   = '{3.25, C};
    nu.p = 8'h34; nu.arr = '{5, 6};
    uu.i = 65;
    sarr[0] = 8'h12; sarr[1] = 8'h34;
    barr = '{'h11, 'h22};
    varr = '{'h33, 'h44};
    iarr = '{1, 2};
    m2   = '{'{1, 2}, '{3, 4}};
    sq.push_back(8'h12); sq.push_back(8'h34); sq.push_back(8'h56);
    sd = new[2]; sd[0] = 8'h9a; sd[1] = 8'hbc;
    sa[0] = 8'h12; sa[5] = 8'h34;
    tc_vif.cfg_arr[0] = cfg_a;  tc_vif.cfg_arr[1] = cfg_b;
    tc_vif.cfg_arr[2] = cfg_a;  tc_vif.cfg_arr[3] = cfg_b;
    tc_vif.sarr[0] = 8'h12; tc_vif.sarr[1] = 8'h34;
    tc_vif.single  = cfg_b;
    vif = tc_vif;
    seq = new();
    seq.cfg = cfg_a;
    seq.sml    = 8'h12;
    seq.unpacked = us;
    seq.cfg_q.push_back(cfg_a); seq.cfg_q.push_back(cfg_b);
    seq.cfg_arr[0] = cfg_a; seq.cfg_arr[1] = cfg_b;
    seq_base::sstat = 8'h34;
    gen #(small_ps_t)::sprop = 8'h56;
    g1 = new(); g1.prop = 8'h78;

    // ---- PS: packed structs, member radix ----------------------------------
    $display("CHK PS.01-kitchen|%p", kx);
    $display("CHK PS.02-cfg|%p", cfg_a);
    $display("CHK PS.03-nested|%p", np);
    $display("CHK PS.04-enum-member|%p", ep);
    $display("CHK PS.05-xz|%p", spx);
    $display("CHK PS.06-zero|%p", spz);
    $display("CHK PS.07-wide|%p", wp);
    $display("CHK PS.08-packed-array-member|%p", dm);
    $display("CHK PS.09-scalar-top|%p %p %p %p", kx.b, kx.y, kx.l, sp);

    // ---- US: unpacked structs ----------------------------------------------
    $display("CHK US.01-kitchen|%p", us);
    $display("CHK US.02-real-enum|%p", ru);
    $display("CHK US.03-packed-in-unpacked|%p", nu);

    // ---- UN: unions ---------------------------------------------------------
    $display("CHK UN.01-packed|%p", pu);
    $display("CHK UN.02-unpacked|%p", uu);

    // ---- AR: unpacked arrays ------------------------------------------------
    $display("CHK AR.01-ps-array|%p", sarr);
    $display("CHK AR.02-ps-elem|%p", sarr[1]);
    $display("CHK AR.03-byte-array|%p", barr);
    $display("CHK AR.04-bit-array|%p", varr);
    $display("CHK AR.05-int-array|%p", iarr);
    $display("CHK AR.06-2d-int|%p", m2);

    // ---- IF: hierarchical / interface ---------------------------------------
    $display("CHK IF.01-if-array|%p", tc_vif.cfg_arr);
    $display("CHK IF.02-if-array-elem|%p", tc_vif.cfg_arr[2]);
    $display("CHK IF.03-if-single|%p", tc_vif.single);
    $display("CHK IF.04-vif-array-elem|%p", vif.cfg_arr[1]);
    $display("CHK IF.05-vif-single|%p", vif.single);
    $display("CHK IF.06-if-elem-member|%0d", tc_vif.cfg_arr[2].f01);

    // ---- CL: class properties ------------------------------------------------
    $display("CHK CL.01-handle|%p", seq.cfg);
    seq.show();
    $display("CHK CL.09-static|%p", seq_base::sstat);
    $display("CHK CL.10-spec-static|%p", gen #(small_ps_t)::sprop);
    $display("CHK CL.11-spec-prop|%p", g1.prop);

    // ---- QC: queues / dynamic / associative ----------------------------------
    $display("CHK QC.01-ps-queue|%p", sq);
    $display("CHK QC.02-ps-dyn|%p", sd);
    $display("CHK QC.03-ps-assoc|%p", sa);
    $display("CHK QC.04-byte-queue|%p", sq);
    $display("CHK QC.05-queue-slice|%p", sq[0:1]);

    // ---- ZP: %0p ---------------------------------------------------------------
    $display("CHK ZP.01-packed|%0p", sp);
    $display("CHK ZP.02-unpacked|%0p", us);
    $display("CHK ZP.03-int-array|%0p", iarr);
    $display("CHK ZP.04-scalar|%0p", kx.b);

    // ---- FN: functions / task formals / locals ---------------------------------
    $display("CHK FN.01-return|%p", mk(8'h9a));
    show_formal(cfg_a);
  end
endmodule
"#;

const EXPECTED: &[&str] = &[
    "CHK PS.01-kitchen|'{a:3, b:-5, ub:300, sh:-2, lg:4886718345, it:-7, y:-1, l:90, r:91}",
    "CHK PS.02-cfg|'{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}",
    "CHK PS.03-nested|'{inner:'{hi:10, lo:11}, tail:205}",
    "CHK PS.04-enum-member|'{e:B, x:5}",
    "CHK PS.05-xz|'{hi:X, lo:5}",
    "CHK PS.06-zero|'{hi:0, lo:0}",
    "CHK PS.07-wide|'{w:16045690984503054900, t:15}",
    "CHK PS.08-packed-array-member|'{dom:305419896, t:15}",
    "CHK PS.09-scalar-top|-5 -1 90 '{hi:1, lo:2}",
    "CHK US.01-kitchen|'{a:1, sh:2, lg:3, it:4, ub:5, y:6, bv:90, lv:91, s:\"str\"}",
    "CHK US.02-real-enum|'{r:3.25, e:C}",
    "CHK US.03-packed-in-unpacked|'{p:'{hi:3, lo:4}, arr:'{5, 6}}",
    "CHK UN.01-packed|'{b:165, p:'{hi:10, lo:5}}",
    "CHK UN.02-unpacked|'{i:65, r:65}",
    "CHK AR.01-ps-array|'{'{hi:1, lo:2}, '{hi:3, lo:4}}",
    "CHK AR.02-ps-elem|'{hi:3, lo:4}",
    "CHK AR.03-byte-array|'{17, 34}",
    "CHK AR.04-bit-array|'{51, 68}",
    "CHK AR.05-int-array|'{1, 2}",
    "CHK AR.06-2d-int|'{'{1, 2}, '{3, 4}}",
    "CHK IF.01-if-array|'{'{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}, '{f01:5, f02:580, f03:7, f04:33, f05:999, f06:3, f07:2, f08:12345, f09:2, f10:64, f11:0, f12:1, f13:0, f14:1}, '{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}, '{f01:5, f02:580, f03:7, f04:33, f05:999, f06:3, f07:2, f08:12345, f09:2, f10:64, f11:0, f12:1, f13:0, f14:1}}",
    "CHK IF.02-if-array-elem|'{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}",
    "CHK IF.03-if-single|'{f01:5, f02:580, f03:7, f04:33, f05:999, f06:3, f07:2, f08:12345, f09:2, f10:64, f11:0, f12:1, f13:0, f14:1}",
    "CHK IF.04-vif-array-elem|'{f01:5, f02:580, f03:7, f04:33, f05:999, f06:3, f07:2, f08:12345, f09:2, f10:64, f11:0, f12:1, f13:0, f14:1}",
    "CHK IF.05-vif-single|'{f01:5, f02:580, f03:7, f04:33, f05:999, f06:3, f07:2, f08:12345, f09:2, f10:64, f11:0, f12:1, f13:0, f14:1}",
    "CHK IF.06-if-elem-member|3",
    "CHK CL.01-handle|'{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}",
    "CHK CL.02-this|'{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}",
    "CHK CL.03-bare|'{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}",
    "CHK CL.04-unpacked|'{a:1, sh:2, lg:3, it:4, ub:5, y:6, bv:90, lv:91, s:\"str\"}",
    "CHK CL.05-q|'{'{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}, '{f01:5, f02:580, f03:7, f04:33, f05:999, f06:3, f07:2, f08:12345, f09:2, f10:64, f11:0, f12:1, f13:0, f14:1}}",
    "CHK CL.06-qelem|'{f01:5, f02:580, f03:7, f04:33, f05:999, f06:3, f07:2, f08:12345, f09:2, f10:64, f11:0, f12:1, f13:0, f14:1}",
    "CHK CL.07-arr|'{'{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}, '{f01:5, f02:580, f03:7, f04:33, f05:999, f06:3, f07:2, f08:12345, f09:2, f10:64, f11:0, f12:1, f13:0, f14:1}}",
    "CHK CL.08-arrelem|'{f01:5, f02:580, f03:7, f04:33, f05:999, f06:3, f07:2, f08:12345, f09:2, f10:64, f11:0, f12:1, f13:0, f14:1}",
    "CHK CL.12-member|0",
    "CHK CL.09-static|'{hi:3, lo:4}",
    "CHK CL.10-spec-static|'{hi:5, lo:6}",
    "CHK CL.11-spec-prop|'{hi:7, lo:8}",
    "CHK QC.01-ps-queue|'{'{hi:1, lo:2}, '{hi:3, lo:4}, '{hi:5, lo:6}}",
    "CHK QC.02-ps-dyn|'{'{hi:9, lo:10}, '{hi:11, lo:12}}",
    "CHK QC.03-ps-assoc|'{0:'{hi:1, lo:2}, 5:'{hi:3, lo:4} }",
    "CHK QC.04-byte-queue|'{'{hi:1, lo:2}, '{hi:3, lo:4}, '{hi:5, lo:6}}",
    "CHK QC.05-queue-slice|'{'{hi:1, lo:2}, '{hi:3, lo:4}}",
    "CHK ZP.01-packed|1 2",
    "CHK ZP.02-unpacked|1 2 3 4 5 6 90 91 \"str\"",
    "CHK ZP.03-int-array|1 2",
    "CHK ZP.04-scalar|-5",
    "CHK FN.01-return|'{hi:9, lo:10}",
    "CHK FN.02|'{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}",
    "CHK FN.03|'{f01:3, f02:4400, f03:9, f04:63, f05:1447, f06:0, f07:5, f08:508360320, f09:5, f10:128, f11:1, f12:0, f13:1, f14:0}",
];

#[test]
fn p_format_prints_every_aggregate_shape() {
    let sim = simulate(SRC, 100).expect("simulate");
    let msgs: Vec<&str> = sim.output.iter().map(|o| o.message.as_str()).collect();
    for want in EXPECTED {
        assert!(msgs.contains(want), "missing line:\n  {want}\ngot:\n  {}", msgs.join("\n  "));
    }
}
