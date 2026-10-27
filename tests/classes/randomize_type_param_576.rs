//! IEEE 1800-2023 §8.25 / §18.4: class members whose type comes from a
//! class type parameter randomize in the object's own specialization (#576,
//! reported by Ganesh-TNL). A `rand T beats[]` (dynamic, fixed, queue or
//! associative) bound wider than `T`'s default solved its elements at the
//! default width, so a field constraint fell outside them; a `rand T h`
//! bound to a class was drawn as a random number instead of randomizing the
//! object; `o.h = new()` and `o.arr[0] = new()` from outside the class built
//! an object of class `T`; and a packed 2-D property through the parameter
//! dropped element writes. Section B of the report (nested dynamic arrays,
//! no type parameter) is a separate gap and is not covered here.

use std::path::Path;
use std::process::Command;

#[test]
fn type_parameter_members_randomize_in_the_bound_type() {
    let test_file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/classes/randomize_type_param_576.sv");
    let output = Command::new(env!("CARGO_BIN_EXE_xezim"))
        .arg("--simulate")
        .arg("-s")
        .arg("tb_top")
        .arg("--no-cache")
        .arg(test_file.to_str().unwrap())
        .output()
        .expect("Failed to execute xezim");
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!combined.contains("FAIL @"), "{combined}");
    assert!(combined.contains("TEST_PASS"), "no TEST_PASS:\n{combined}");
}
