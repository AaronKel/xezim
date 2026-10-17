# xezim Performance Review: Key Findings & Architecture Analysis

## 1. Executive Summary & Live Benchmark Data

Measurements taken on host `Intel(R) Core(TM) i7-9800X @ 3.80GHz` (x86_64, 6 threads) with release profile (`lto = "thin"`, `codegen-units = 1`, `opt-level = 3`):

| Workload | Stressed Axis | Throughput | Latency (ns/cycle) | Status |
|---|---|---|---|---|
| `comb_alu` | Single-thread integer ALU / Comb settle | 755,199 cyc/s | 1,324.2 ns | Baseline |
| `wide_vec` | 512-bit vector ops & memory movement | 994,407 cyc/s | 1,005.6 ns | Good |
| `mem_array` | Unpacked array traversal / cache latency | 1,175,572 cyc/s | 850.6 ns | Fast |
| `event_ctrl` | Event ping-pong (`->` and `@`) | 192,697 cyc/s | 5,189.5 ns | **5.5× slower than ALU** |
| `class_queue` | Class lifecycle, dynamic queue & assoc | 41,963 cyc/s | 23,830.8 ns | **28.0× slower than ALU** |

Synthetic profiling of `b2b_vm_branchy` (50,000 cycles, 512 blocks) revealed:
- **95.9% of simulation time** is spent in edge execution (`edge_exec = 2,019.2 ms` out of `2,194.9 ms`).
- **273,030,862 bytecode instructions** executed (7.4 ns per instruction).
- **179,200,000 `LoadConst` → `CaseEq` → `BranchIfFalse` instruction sequences** executed through the VM.

---

## 2. Key Finding #1: Peephole Def-Kill Bug Blocking Comparison Fusion

- **Affected Source Files**: [`src/compiler/bytecode.rs`](file:///home/bondan/repo/fix/oct14/xezim/src/compiler/bytecode.rs#L19852-L19940) and [`src/compiler/bytecode.rs`](file:///home/bondan/repo/fix/oct14/xezim/src/compiler/bytecode.rs#L20476-L20600)
- **Functions**: `BytecodeCompiler::fuse_cmp_branch_move_resize` and `BytecodeCompiler::fuse_binop_const`

### Mechanism & Root Cause
1. In `case` statements, the compiler recycles temporary register indices across arms to avoid register explosion:
   ```rust
   // src/compiler/bytecode.rs:8506
   let arm_reg_start = self.next_reg;
   // ... each arm compiles its expression and statement ...
   self.next_reg = arm_reg_start;
   ```
2. When `fuse_cmp_branch_move_resize` checks if the comparison result register `c` is dead after `BranchIfFalse(c, target)`:
   ```rust
   // src/compiler/bytecode.rs:19917
   let only_reader = if fwd_only {
       insns[j + 1..].iter().all(|x| !Self::insn_reads_reg(x, c))
   } else { ... };
   ```
3. Because every subsequent arm recycles `c` (e.g., `reg 3`), Arm 2, Arm 3, Arm 4, etc. write `reg 3` and then read `reg 3` for their own condition.
4. The linear scan `insns[j + 1..].iter().all(|x| !Self::insn_reads_reg(x, c))` sees Arm 2 reading `reg 3` down the block. It assumes `reg 3` from Arm 1 is still live, **failing to recognize that Arm 2 writes `reg 3` before reading it**.
5. As a result:
   - **Arms 1 through 6 fail fusion**; only Arm 7 (the final arm) is fused into `CmpBranch`.
   - In `fuse_binop_const`, `RegReaders::read_outside` checks `self.reads[r] > own`. Because `self.reads` is a global counter of reads across the whole block, recycling register `r` makes `read_outside` always return `true`, blocking `LoadConst` + `CaseEq` from fusing into `BinOpConst(CaseEq)`.
   - In `b2b_vm_branchy`, **~85% of all comparisons executed unfused** (179M times). In XuanTie C906, over 76M `LoadConst -> CaseEq` pairs were left unfused for this exact reason.

### Remediation
Implement basic-block / definition-kill tracking: when scanning forward from instruction `j + 1`, if an instruction overwrites register `c` without reading it first, register `c` from instruction `i` is dead at that point.

---

## 3. Key Finding #2: `CaseJump` Threshold Disabling Jump Tables on 3-Bit Selectors

- **Affected Source File**: [`src/compiler/bytecode.rs`](file:///home/bondan/repo/fix/oct14/xezim/src/compiler/bytecode.rs#L8481)
- **Function**: `BytecodeCompiler::compile_case_stmt`

### Mechanism & Root Cause
1. Line 8481 enforces an arbitrary threshold:
   ```rust
   if n_entries < 8 {
       return false;
   }
   ```
2. In `b2b_vm_branchy.sv`, `case (lfsr[2:0])` has 7 non-default arms (`3'd0` through `3'd6`). Because `7 < 8`, `CaseJump` **declines to generate a jump table**.
3. All 512 always blocks fall back to a linear compare chain of up to **21 instructions** (`LoadConst`, `CaseEq`, `BranchIfFalse` × 7) per block per clock edge.
4. When `CaseJump` is enabled, 21 instructions collapse into **1 single instruction** (`Insn::CaseJump(sel, table)`), which lowers directly into `TsInsn::CaseJmp` in the 2-state pipeline.

### Remediation
Lower the threshold from `n_entries < 8` to `n_entries < 4 && max_idx <= 2 * n_entries + 16` (standard dense switch heuristic).

---

## 4. Key Finding #3: Event Notification Overhead in `fire_named_event`

- **Affected Source File**: [`src/compiler/simulator.rs`](file:///home/bondan/repo/fix/oct14/xezim/src/compiler/simulator.rs#L90367-L90440)
- **Function**: `Simulator::fire_named_event`
- **Observed Latency**: **5,189.5 ns/cycle** on `event_ctrl`

### Mechanism & Root Cause
On every `-> event` trigger:
1. `raw_name` is cloned into multiple `String`s (`raw`, `trimmed`, `top_prefixed = format!("{}.{}", ...)`).
2. A `Vec<String>` of candidate names is allocated, sorted, and deduplicated.
3. String-based hash map lookups (`self.signal_name_to_id.contains_key(...)`) are performed.
4. Cloned strings are inserted into `event_triggered_time: HashMap<String, u64>`.
5. `self.settle_combinatorial()` is unconditionally invoked across the whole design even when no continuous net or sensitivity depends on the event.

### Remediation
1. Intern named events at elaboration into dense integer identifiers `EventId(u32)`.
2. Convert `event_triggered_time` from `HashMap<String, u64>` to `Vec<u64>` indexed by `EventId`.
3. Gate `settle_combinatorial()` so it only runs if the event has registered combinational listeners or drives clocking blocks.

---

## 5. Key Finding #4: Procedural Process Disqualification & AST Tree-Walking Trap

- **Affected Source File**: [`src/compiler/simulator.rs`](file:///home/bondan/repo/fix/oct14/xezim/src/compiler/simulator.rs#L22659)
- **Function**: `Simulator::try_register_proc_fsm`
- **Observed Latency**: **23,830.8 ns/cycle** on `class_queue` (28× slower than ALU)

### Mechanism & Root Cause
1. Line 22659 explicitly denies processes referencing dynamic collections from compiling into bytecode:
   ```rust
   compiler.set_collection_denies(&self.module.dynamic_arrays, &self.module.queue_vars);
   ```
2. Any procedural block containing a queue (`q[$]`), dynamic array (`dyn[]`), or class `new` is rejected by `try_register_proc_fsm` and stays on the **AST interpreter** (`run_process_stmts`).
3. In `class_queue`, every single cycle:
   - Allocates class properties via `HashMap<String, Value>`.
   - Dispatches methods (`push_back`, `pop_front`, `size`) via string comparison and AST node evaluation.
   - Converts associative array keys to strings.
   - Pushes and pops continuation objects (`ProcCont`) through the timing wheel.
4. Total bytecode instructions executed during the 5,000-cycle run: **only 3 instructions** (`vm_insns_total=3`). 99.7% of time was trapped in AST tree-walking.

### Remediation
1. Permit queue and collection methods in `ProcFsm`: the instruction set already defines bytecode opcodes (`CallCollMethod`, `LoadCollElem`, `StoreCollElem`).
2. Flatten class properties from `HashMap<String, Value>` to a contiguous array indexed by `PropId(u32)`.

---

## 6. Key Finding #5: Signal Memory Footprint (AoS Cache Pressure & Unintegrated SoA)

- **Affected Source Files**:
  - [`xezim-core/src/value.rs`](file:///home/bondan/repo/fix/oct14/xezim-core/src/value.rs#L414) (`struct Value`)
  - [`src/compiler/soa.rs`](file:///home/bondan/repo/fix/oct14/xezim/src/compiler/soa.rs#L458) (`struct SoaSignalStore`)
  - [`src/compiler/simulator.rs`](file:///home/bondan/repo/fix/oct14/xezim/src/compiler/simulator.rs#L18828-L18840)

### Mechanism & Root Cause
1. `Value` is **32 bytes** (`storage: ValueStorage` [24B], `width: u32` [4B], flags [4B]).
2. `signal_table: Vec<Value>` stores every design signal in this format. For large designs (C910 SoC: 465,000 signals; full SoC nets: 35,000,000 signals), `signal_table` consumes **15 MB to 1.1 GB**, overwhelming L2/L3 caches.
3. During 2-state simulation (`settle_combinatorial` / `exec_two_state`), only `u64` value bits are needed. Reading a 32-byte struct to access an 8-byte word wastes **75% of cache bandwidth**.
4. An earlier mirror layout (`signal_inline_bits: Vec<[u64; 2]>`) operated alongside `signal_table`, requiring dual writes on every signal update. The extra memory traffic (560 MB on C906) caused a 2.6% slowdown and was disabled by default.
5. [`SoaSignalStore`](file:///home/bondan/repo/fix/oct14/xezim/src/compiler/soa.rs#L458) was implemented with parallel vectors (`val_bits: Vec<u64>`, `xz_bits: Vec<u64>`, `widths: Vec<u32>`), but is **currently unused by `Simulator`**.

### Remediation
Adopt `InlineSignalArray` as the **primary signal storage** (single source of truth) for all signals ≤ 64 bits. Reconstruct `Value` only on demand as transient views when interacting with DPI/VPI or AST fallbacks.

---

## 7. Key Finding #6: Single-Point While-Match VM Dispatch

- **Affected Source File**: [`src/compiler/simulator.rs`](file:///home/bondan/repo/fix/oct14/xezim/src/compiler/simulator.rs#L34771)
- **Function**: `Simulator::exec_insns`

### Mechanism & Root Cause
- All instructions are executed via a central `while pc < len { match &insns[pc] { ... } }` loop with ~100 match arms.
- When instruction sequences vary dynamically (as in `b2b_vm_branchy`), the hardware Branch Target Buffer (BTB) cannot predict targets through the single indirect branch, dropping IPC from 3.07 to 2.23.
- [`src/compiler/dispatch.rs`](file:///home/bondan/repo/fix/oct14/xezim/src/compiler/dispatch.rs) contains an unintegrated proof-of-concept for direct-threaded dispatch.

---

## 8. Prioritized Optimization Roadmap

| Priority | Optimization | Target Component | Expected Speedup | Complexity |
|---|---|---|---|---|
| **P0** | **Fix Peephole Def-Kill Liveness** | `fuse_cmp_branch_move_resize` & `fuse_binop_const` | **15% – 30%** on branchy RTL | Low |
| **P0** | **Lower `CaseJump` Threshold (8 → 4)** | `compile_case_stmt` | **2× – 5×** on 3-bit / multi-arm cases | Very Low |
| **P1** | **Introduce `CmpBranchConst` Superinstruction** | Bytecode & VM dispatch | **10% – 20%** on control logic | Low |
| **P1** | **Intern Named Events (`EventId`)** | `fire_named_event` & `event_triggered_time` | **3× – 5×** on event synchronization | Medium |
| **P2** | **Support Collections in `ProcFsm`** | `try_register_proc_fsm` | **4× – 8×** on UVM / class testbenches | Medium |
| **P2** | **Primary SoA Signal Storage** | `SoaSignalStore` integration | **15% – 30%** on full-chip SoC runs | High |
| **P3** | **Class Property Vector Layout** | `instantiate_class` & property reads | **2× – 3×** on object allocation | Medium |
| **P3** | **PGO + BOLT Toolchain Pipeline** | `build-pgo.sh` & `build-bolt.sh` | **10% – 15%** whole-run reduction | Low |
