use xezim::simulate;

fn check(source: &str) {
    let result = simulate(source, 100).unwrap();
    let output = result
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

fn policies() -> bool {
    super::compiled_method_test_env::policies(&[("0", "1000"), ("1", "0"), ("1", "1000")])
}

#[test]
fn compiled_enum_case_keeps_lexical_class_constants() {
    if !policies() {
        return;
    }
    check(include_str!("../fixtures/compiled_lexical_enum.sv"));
}

#[test]
fn formal_shadows_compiled_class_enum_constant() {
    if !policies() {
        return;
    }
    check(&include_str!("../fixtures/compiled_lexical_enum.sv")
        .replace("extern function kind_t channel();", "extern function kind_t channel(); function int shadow(int RD_PLAIN); return RD_PLAIN; endfunction")
        .replace("$display(\"PASS\");", "repeat(2005) if(t.shadow(21)!=21) $fatal(1,\"enum shadow\"); $display(\"PASS\");"));
}

#[test]
fn compiled_other_object_string_store_preserves_all_bytes() {
    if !policies() {
        return;
    }
    check(include_str!("../fixtures/compiled_member_string.sv"));
}

#[test]
fn compiled_other_object_wide_store_preserves_all_bits() {
    if !policies() {
        return;
    }
    check(
        &include_str!("../fixtures/compiled_member_string.sv")
            .replace("string", "bit[127:0]")
            .replace(
                "\"read_data_21399\"",
                "128'h123456789abcdef0_fedcba9876543210",
            )
            .replace("\"\"", "'0"),
    );
}

#[test]
fn class_localparam_shadows_inherited_enum_constant() {
    if !policies() {
        return;
    }
    check(
        r#"
class base;
 typedef enum {TOKEN=1} base_kind;
endclass
class child extends base;
 localparam int TOKEN=7;
 function int value(); return TOKEN; endfunction
endclass
class unrelated;
 typedef enum {TOKEN=14} other_kind;
endclass
module top;
 initial begin child c=new();
   repeat(2005) if(c.value()!=7) $fatal(1,"class parameter shadow");
   $display("PASS");$finish;
 end
endmodule
"#,
    );
}
