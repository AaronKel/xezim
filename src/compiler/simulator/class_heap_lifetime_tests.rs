// Baseline semantics for class reclamation: every retained handle must survive
// allocation churn, including handles that are not ordinary scalar globals.
#[test]
fn retained_class_handles_survive_allocation_churn() {
    let sim = crate::simulate(
        include_str!("../../../tests/fixtures/memory/heap_lifetimes.sv"),
        100_000,
    )
    .expect("heap lifetime fixture");
    assert!(sim.finished);
    assert_eq!(sim.error_count, 0);
    assert!(sim
        .output
        .iter()
        .any(|line| { line.message.contains("HEAP_LIFETIMES_PASS checks=5") }));
}

#[test]
fn lifetime_fixture_exercises_hidden_roots_while_processes_are_parked() {
    let sim = crate::simulate(
        include_str!("../../../tests/fixtures/memory/heap_lifetimes.sv"),
        8,
    )
    .expect("parked heap lifetime fixture");
    assert!(!sim.finished);
    assert_eq!(sim.error_count, 0);
    let node = |id| {
        sim.heap
            .iter()
            .enumerate()
            .find_map(|(handle, object)| {
                let object = object.as_ref()?;
                (object.class_name == "Node"
                    && object.properties.get("id").and_then(|v| v.to_u64()) == Some(id))
                .then_some(handle as u64)
            })
            .expect("retained node")
    };
    // These values are captured outside the ordinary current local frame.
    let blocked = node(66);
    assert!(sim
        .mailbox_put_waiters
        .values()
        .flatten()
        .any(|waiter| { waiter.value.to_u64() == Some(blocked) }));
    let shadowed = node(99);
    assert!(sim.process_contexts.values().any(|ctx| {
        ctx.block_scope_saves.iter().any(|scope| {
            scope
                .shadowed
                .iter()
                .any(|(_, value)| value.to_u64() == Some(shadowed))
        })
    }));
}
