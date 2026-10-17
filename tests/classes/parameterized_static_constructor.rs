//! IEEE 1800-2023 §6.8/§8.25: each referenced class specialization
//! initializes its static constructor once, before initial processes.
use std::process::Command;

fn run(source: &str, tag: &str) -> String {
    let dir =
        std::env::temp_dir().join(format!("xezim_static_ctor_{}_{}", std::process::id(), tag));
    std::fs::create_dir_all(&dir).unwrap();
    let source_path = dir.join("design.sv");
    std::fs::write(&source_path, source).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .args(["--simulate", "-s", "top"])
        .arg(&source_path)
        .output()
        .expect("run xezim");
    let stdout = String::from_utf8_lossy(&result.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        result.status.success(),
        "simulation failed:\n{stdout}\n{stderr}"
    );
    std::fs::remove_dir_all(dir).unwrap();
    stdout
}

const ORIGINAL: &str = r#"// static_init_param_class.sv
module top;

    int checked  = 0;
    int failures = 0;

    function automatic void report(string what, bit ok);
        checked++;
        if (ok) $display("PASS: %s", what);
        else begin
            failures++;
            $display("FAIL: %s", what);
        end
    endfunction

    // ---- (1) the UVM pattern: a static queue filled from a static initializer
    class lib_base;
        static lib_base registered[$];
    endclass

    class lib_adder #(type T = int) extends lib_base;
        function new();
            lib_base::registered.push_back(this);
        endfunction
        static lib_adder#(T) inst = new();
    endclass

    typedef lib_adder#(int) lib_adder_int;

    // ---- (2) control: the same shape, not parameterized
    class plain_base;
        static plain_base registered[$];
    endclass

    class plain_adder extends plain_base;
        function new();
            plain_base::registered.push_back(this);
        endfunction
        static plain_adder inst = new();
    endclass

    // ---- (3) counter form of (1)
    class cnt_base;
        static int made = 0;
    endclass

    class cnt_param #(type T = int) extends cnt_base;
        function new();
            cnt_base::made++;
        endfunction
        static cnt_param#(T) inst = new();
    endclass

    typedef cnt_param#(int) cnt_param_int;

    // ---- (4) control: the same counter shape, not parameterized
    class cnt2_base;
        static int made = 0;
    endclass

    class cnt2_plain extends cnt2_base;
        function new();
            cnt2_base::made++;
        endfunction
        static cnt2_plain inst = new();
    endclass

    // ---- (5) parameterized, and an instance of the specialization is built
    class cnt3_base;
        static int made = 0;
    endclass

    class cnt3_param #(type T = int) extends cnt3_base;
        function new();
            cnt3_base::made++;
        endfunction
        static cnt3_param#(T) inst = new();
    endclass

    // ---- (6) contrast: a function-call initializer in a parameterized class
    class fn_base;
        static int calls = 0;
    endclass

    class fn_param #(type T = int) extends fn_base;
        static function int mk();
            fn_base::calls++;
            return 7;
        endfunction
        static int n = mk();
    endclass

    typedef fn_param#(int) fn_param_int;

    initial begin
        // (1) and (2)
        $display("       parameterized  : registered.size()=%0d", lib_base::registered.size());
        $display("       control        : registered.size()=%0d", plain_base::registered.size());
        report("parameterized class: static initializer registered itself",
               lib_base::registered.size() == 1);
        report("control, non-parameterized class: static initializer registered itself",
               plain_base::registered.size() == 1);

        // (3) and (4)
        report("parameterized class: static initializer ran its constructor",
               cnt_base::made == 1);
        report("control, non-parameterized class: static initializer ran its constructor",
               cnt2_base::made == 1);

        // (5)
        begin
            cnt3_param#(int) a;
            a = new();                       // one construction here
        end
        $display("       with an instance built: made=%0d (2 = static init also ran)", cnt3_base::made);
        report("parameterized class: static initializer also runs when an instance is built",
               cnt3_base::made == 2);

        // (6) read the value first, so a side effect that only happens when the
        // value is first needed cannot be mistaken for a lost one
        begin
            int v;
            v = fn_param#(int)::n;
            $display("       function-call init: value=%0d calls=%0d", v, fn_base::calls);
            report("contrast: function-call initializer in a parameterized class produced the value",
                   v == 7);
            report("contrast: function-call initializer in a parameterized class ran its side effect",
                   fn_base::calls == 1);
        end

        $display("=== %0d check(s), %0d failure(s) ===", checked, failures);
    end

endmodule
"#;

#[test]
fn original_static_constructor_initialization() {
    let output = run(ORIGINAL, "original");
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("PASS:"))
            .count(),
        7,
        "{output}"
    );
    assert!(
        output.contains("=== 7 check(s), 0 failure(s) ==="),
        "{output}"
    );
    assert!(!output.contains("FAIL:"), "{output}");
}

const NEIGHBORS: &str = r#"class ledger;
  static int valued = 0;
  static int typed = 0;
  static int aliased = 0;
  static int functions = 0;
  static int recursive = 0;
endclass
class value_entry #(int N=1);
  int payload;
  function new(int n);
    payload=n;
    ledger::valued += n;
  endfunction
  static value_entry#(N) inst = new(N);
endclass
typedef value_entry#(2) value_two;
typedef value_entry#(5) value_five;
class item;
  int payload;
  function new(int n); payload=n; ledger::typed++; endfunction
endclass
class type_entry #(type T=int);
  static item inst = new(19);
endclass
typedef type_entry#(int) type_int;
typedef type_entry#(byte) type_byte;
class alias_entry #(int N=1);
  typedef alias_entry#(N) own_type;
  int payload;
  function new(int n); payload=n; ledger::aliased++; endfunction
  static own_type inst = new(N);
endclass
typedef alias_entry#(11) alias_eleven;
class function_entry #(int N=1);
  static function int make_value(); ledger::functions++; return N+3; endfunction
  static int value=make_value();
endclass
typedef function_entry#(17) function_seventeen;
class recursive_entry #(type T=int);
  static recursive_entry#(T) inst = new;
  function new(); if(inst==null) ledger::recursive++; endfunction
endclass
typedef recursive_entry#(int) recursive_int;
module top;
  int checked=0;
  int failures=0;
  task check(input string label, input bit ok);
    checked++;
    if(ok) $display("T|PASS %s",label);
    else begin failures++; $display("T|FAIL %s",label); end
  endtask
  initial begin
    value_two extra;
    check("value constructors precede initial",ledger::valued==7);
    check("distinct value specialization payloads",value_two::inst.payload==2 && value_five::inst.payload==5);
    check("different member class and type specializations",ledger::typed==2 && type_int::inst.payload==19 && type_byte::inst.payload==19);
    check("self typedef and constructor argument",ledger::aliased==1 && alias_eleven::inst.payload==11);
    check("function initializer precedes initial",ledger::functions==1);
    check("function initializer value",function_seventeen::value==20);
    check("recursive static read sees default",ledger::recursive==1 && recursive_int::inst!=null);
    extra=new(2);
    check("explicit construction and one time statics",ledger::valued==9 && value_two::inst.payload==2 && ledger::typed==2 && ledger::functions==1);
    $display("T|SUMMARY checks=%0d failures=%0d",checked,failures);
  end
endmodule
"#;

#[test]
fn static_constructor_specialization_neighbors() {
    let output = run(NEIGHBORS, "neighbors");
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("T|PASS"))
            .count(),
        8,
        "{output}"
    );
    assert!(output.contains("T|SUMMARY checks=8 failures=0"), "{output}");
    assert!(!output.contains("T|FAIL"), "{output}");
}
