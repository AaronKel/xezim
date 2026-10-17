use super::*;
use crate::compiler::bytecode::TsInsn;

type Executor = fn(&mut Simulator, &[TsInsn], u32) -> bool;

fn executors() -> [(&'static str, Executor); 2] {
    [
        ("line", Simulator::exec_two_state_line),
        ("control", Simulator::exec_two_state_ctrl),
    ]
}

fn fixture() -> (Simulator, u32, u32) {
    let mut sim = crate::simulate(
        "module tb; logic [15:0] data; logic [127:0] wide; endmodule",
        0,
    )
    .expect("elaborate dynamic executor signals");
    let data = *sim.signal_name_to_id.get("data").expect("data signal") as u32;
    let wide = *sim.signal_name_to_id.get("wide").expect("wide signal") as u32;
    sim.ts_store_xz(data as usize, 0xabcd, 0);
    sim.ts_regs = vec![5, 4, 0];
    (sim, data, wide)
}

// #334: a completed out-of-line instruction must continue the stream.
// Check observable reads, blocking/NBA writes and save records, in addition
// to the return code, so a fallback cannot hide a broken success dispatch.
#[test]
fn successful_dynamic_instructions_continue_both_executors() {
    for (engine, execute) in executors() {
        for case in 0..6 {
            let (mut sim, data, wide) = fixture();
            let op = match case {
                0 => TsInsn::SigRangeDyn {
                    d: 0,
                    sig: data,
                    i: 1,
                    w: 8,
                    sw: 16,
                    mask: 0xff,
                },
                1 => TsInsn::RangeStoreDyn {
                    sig: data,
                    i: 1,
                    s: 0,
                    w: 8,
                    sw: 16,
                    mask: 0xff,
                },
                2 => TsInsn::RangeStoreNbaDyn {
                    sig: data,
                    i: 1,
                    s: 0,
                    w: 8,
                    sw: 16,
                    mask: 0xff,
                },
                3 => TsInsn::SaveSigW { sig: wide },
                4 => TsInsn::RangeFillXW {
                    sig: wide,
                    hi: 7,
                    lo: 0,
                    v: 1,
                },
                _ => TsInsn::LoopPoll,
            };
            let insns = [op, TsInsn::Const { d: 2, v: 0xfeed }];
            assert!(execute(&mut sim, &insns, 3), "{engine}, case {case}");
            assert!(!sim.ts_xread_bail, "{engine}, case {case}");
            assert_eq!(sim.ts_regs[2], 0xfeed, "stream must reach its tail");
            match case {
                0 => assert_eq!(sim.ts_regs[0], 0xbc),
                1 => assert_eq!(sim.signal_table[data as usize].to_u64(), Some(0xa05d)),
                2 => {
                    assert_eq!(sim.signal_table[data as usize].to_u64(), Some(0xabcd));
                    let pending = sim
                        .nba_fast
                        .iter()
                        .find(|nba| nba.signal_id == data as usize)
                        .expect("pending dynamic NBA");
                    assert_eq!(pending.value.to_u64(), Some(0xa05d));
                }
                3 => {
                    assert_eq!(sim.ts_save_list.len(), 3);
                    assert_eq!(sim.ts_save_list.last().unwrap().2, u64::MAX);
                }
                4 => assert_eq!(
                    Simulator::raw_bits_slice(&sim.signal_table[wide as usize], 0, 8),
                    (0xff, 0xff)
                ),
                _ => {}
            }
        }
    }
}

#[test]
fn dynamic_read_distinguishes_out_of_bounds_from_unknown_data() {
    for (engine, execute) in executors() {
        for unknown in [false, true] {
            let (mut sim, data, _) = fixture();
            if unknown {
                sim.ts_store_xz(data as usize, 0xabcd, 1 << 4);
            } else {
                sim.ts_regs[1] = 16;
            }
            let insns = [
                TsInsn::SigRangeDyn {
                    d: 0,
                    sig: data,
                    i: 1,
                    w: 8,
                    sw: 16,
                    mask: 0xff,
                },
                TsInsn::Const { d: 2, v: 0xfeed },
            ];
            assert!(!execute(&mut sim, &insns, 3), "{engine}, unknown={unknown}");
            assert_eq!(sim.ts_xread_bail, unknown, "{engine}");
            assert_eq!(sim.ts_regs[2], 0, "an aborted stream must skip its tail");
        }
    }
}
