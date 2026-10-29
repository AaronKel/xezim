//! §23.8, §25.9: inside a function of an interface instance,
//! `parent_if.children[0]` names, through the interface's DEFINITION name,
//! the instance the function runs in. Returned or assigned to a virtual
//! interface it bound nothing (the caller read x / null), while the bare
//! `children[0]` bound correctly. Expected values are the reference
//! simulator's.

use xezim::simulate;

fn t_lines(src: &str) -> Vec<String> {
    let sim = simulate(src, 1_000).expect("simulate failed");
    sim.output
        .iter()
        .map(|o| o.message.clone())
        .filter(|m| m.starts_with("T|"))
        .collect()
}

#[test]
fn vif_bound_through_own_definition_name() {
    const SRC: &str = r#"interface child_if;
  int id;
endinterface
interface parent_if;
  child_if kid();
  child_if kids[2]();
  initial begin kid.id = 4; kids[1].id = 9; end
  function automatic virtual child_if one(); return parent_if.kid; endfunction
  function automatic virtual child_if elem(); return parent_if.kids[1]; endfunction
  function automatic virtual child_if via_local();
    virtual child_if t;
    t = parent_if.kid;
    return t;
  endfunction
endinterface
module top;
  parent_if parent();
  virtual parent_if p;
  virtual child_if c;
  initial begin
    #1;
    p = parent;
    c = p.one();
    $display("T|one null=%0d id=%0d", c == null, c.id);
    c = p.elem();
    $display("T|elem null=%0d id=%0d", c == null, c.id);
    c = parent.via_local();
    $display("T|local null=%0d id=%0d", c == null, c.id);
  end
endmodule
"#;
    assert_eq!(
        t_lines(SRC),
        [
            "T|one null=0 id=4",
            "T|elem null=0 id=9",
            "T|local null=0 id=4",
        ]
    );
}
