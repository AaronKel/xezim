#[test]
fn completed_fork_children_release_execution_caches() {
    let sim = crate::simulate(
        r#"
        module top;
          int count;
          initial begin
            for (int n=0; n<200; n++) begin
              fork begin
                for (int i=0; i<8; i++) count++;
                #1;
              end join
            end
            if (count != 1600) $fatal(1, "child execution changed");
            $finish;
          end
        endmodule
    "#,
        1000,
    )
    .expect("fork cache lifetime fixture");
    assert_eq!(sim.error_count, 0);
    assert!(sim.finished_pids.len() >= 200);
    assert!(sim
        .process_loop_bytecode
        .keys()
        .all(|(p, _, _)| !sim.finished_pids.contains(p)));
    assert!(sim
        .process_cond_bytecode
        .keys()
        .all(|(p, _, _)| !sim.finished_pids.contains(p)));
    assert!(sim
        .forever_cont_cache
        .keys()
        .all(|(p, _, _, _, _)| !sim.finished_pids.contains(p)));
    assert!(sim
        .forever_sens_cache
        .keys()
        .all(|(p, _)| !sim.finished_pids.contains(p)));
}

#[test]
fn killed_process_releases_caches_but_live_process_keeps_them() {
    let source = r#"
        module top;
          process victim;
          int count;
          initial begin
            fork begin
              victim = process::self();
              forever begin
                for (int i=0; i<8; i++) count++;
                #1;
              end
            end join_none
            #5; victim.kill();
            #2;
            if (count != 40) $fatal(1, "killed process continued");
            $finish;
          end
        endmodule
    "#;
    let live = crate::simulate(source, 3).expect("live process cache fixture");
    assert!(live.killed_pids.is_empty());
    assert!(!live.forever_cont_cache.is_empty());
    let killed = crate::simulate(source, 100).expect("killed process cache fixture");
    assert_eq!(killed.error_count, 0);
    assert!(!killed.killed_pids.is_empty());
    assert!(killed
        .process_loop_bytecode
        .keys()
        .all(|(p, _, _)| !killed.killed_pids.contains(p)));
    assert!(killed
        .process_cond_bytecode
        .keys()
        .all(|(p, _, _)| !killed.killed_pids.contains(p)));
    assert!(killed
        .forever_cont_cache
        .keys()
        .all(|(p, _, _, _, _)| !killed.killed_pids.contains(p)));
    assert!(killed
        .forever_sens_cache
        .keys()
        .all(|(p, _)| !killed.killed_pids.contains(p)));
}

#[test]
fn disabled_fork_children_release_execution_caches() {
    let sim = crate::simulate(
        r#"
        module top;
          int count;
          initial begin
            for (int n=0; n<50; n++) begin
              fork
                forever begin
                  for (int i=0; i<8; i++) count++;
                  #1;
                end
              join_none
              #2;
              disable fork;
            end
            if (count != 800) $fatal(1, "disabled children ran on: %0d", count);
            $finish;
          end
        endmodule
    "#,
        1000,
    )
    .expect("disable fork cache fixture");
    assert_eq!(sim.error_count, 0);
    assert!(sim.killed_pids.len() >= 50);
    assert!(sim
        .process_loop_bytecode
        .keys()
        .all(|(p, _, _)| !sim.killed_pids.contains(p)));
    assert!(sim
        .forever_cont_cache
        .keys()
        .all(|(p, _, _, _, _)| !sim.killed_pids.contains(p)));
    assert!(sim
        .forever_sens_cache
        .keys()
        .all(|(p, _)| !sim.killed_pids.contains(p)));
    assert!(sim.exec_cache_pids.iter().all(|p| !sim.killed_pids.contains(p)));
}
