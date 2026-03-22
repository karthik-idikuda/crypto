//! Benchmarks for the NXVM virtual machine

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nxvm::{
    Vm, Opcode,
    stack::u64_to_value,
    vm::instr,
};

fn bench_simple_halt(c: &mut Criterion) {
    c.bench_function("VM: Halt", |b| {
        b.iter(|| {
            let program = vec![instr(Opcode::Halt, None)];
            let mut vm = Vm::new(program, 100_000);
            black_box(vm.execute().unwrap())
        })
    });
}

fn bench_arithmetic_chain(c: &mut Criterion) {
    c.bench_function("VM: 100 additions", |b| {
        b.iter(|| {
            let mut program = Vec::with_capacity(202);
            program.push(instr(Opcode::Push, Some(u64_to_value(0).to_vec())));
            for _ in 0..100 {
                program.push(instr(Opcode::Push, Some(u64_to_value(1).to_vec())));
                program.push(instr(Opcode::Add, None));
            }
            program.push(instr(Opcode::Halt, None));
            let mut vm = Vm::new(program, 1_000_000);
            black_box(vm.execute().unwrap())
        })
    });
}

fn bench_push_pop(c: &mut Criterion) {
    c.bench_function("VM: 100 push/pop cycles", |b| {
        b.iter(|| {
            let mut program = Vec::with_capacity(201);
            for _ in 0..100 {
                program.push(instr(Opcode::Push, Some(u64_to_value(42).to_vec())));
                program.push(instr(Opcode::Pop, None));
            }
            program.push(instr(Opcode::Halt, None));
            let mut vm = Vm::new(program, 1_000_000);
            black_box(vm.execute().unwrap())
        })
    });
}

fn bench_storage_ops(c: &mut Criterion) {
    c.bench_function("VM: 50 SStore/SLoad cycles", |b| {
        b.iter(|| {
            let mut program = Vec::new();
            for i in 0u64..50 {
                // SStore: push value, push slot, sstore
                program.push(instr(Opcode::Push, Some(u64_to_value(i * 100).to_vec())));
                program.push(instr(Opcode::Push, Some(u64_to_value(i).to_vec())));
                program.push(instr(Opcode::SStore, None));
            }
            for i in 0u64..50 {
                // SLoad: push slot, sload, pop
                program.push(instr(Opcode::Push, Some(u64_to_value(i).to_vec())));
                program.push(instr(Opcode::SLoad, None));
                program.push(instr(Opcode::Pop, None));
            }
            program.push(instr(Opcode::Halt, None));
            let mut vm = Vm::new(program, 10_000_000);
            black_box(vm.execute().unwrap())
        })
    });
}

fn bench_comparison_ops(c: &mut Criterion) {
    c.bench_function("VM: 100 comparisons", |b| {
        b.iter(|| {
            let mut program = Vec::new();
            for i in 0u64..100 {
                program.push(instr(Opcode::Push, Some(u64_to_value(i).to_vec())));
                program.push(instr(Opcode::Push, Some(u64_to_value(i + 1).to_vec())));
                program.push(instr(Opcode::Lt, None));
                program.push(instr(Opcode::Pop, None));
            }
            program.push(instr(Opcode::Halt, None));
            let mut vm = Vm::new(program, 1_000_000);
            black_box(vm.execute().unwrap())
        })
    });
}

fn bench_dup_swap(c: &mut Criterion) {
    c.bench_function("VM: 100 dup+swap", |b| {
        b.iter(|| {
            let mut program = vec![
                instr(Opcode::Push, Some(u64_to_value(1).to_vec())),
                instr(Opcode::Push, Some(u64_to_value(2).to_vec())),
            ];
            for _ in 0..100 {
                program.push(instr(Opcode::Dup, None));
                program.push(instr(Opcode::Swap, None));
                program.push(instr(Opcode::Pop, None));
            }
            program.push(instr(Opcode::Halt, None));
            let mut vm = Vm::new(program, 1_000_000);
            black_box(vm.execute().unwrap())
        })
    });
}

fn bench_mixed_workload(c: &mut Criterion) {
    c.bench_function("VM: mixed workload (arith+store+compare)", |b| {
        b.iter(|| {
            let mut program = Vec::new();
            // Compute and store
            for i in 0u64..20 {
                program.push(instr(Opcode::Push, Some(u64_to_value(i).to_vec())));
                program.push(instr(Opcode::Push, Some(u64_to_value(i * 2).to_vec())));
                program.push(instr(Opcode::Add, None));
                program.push(instr(Opcode::Push, Some(u64_to_value(i).to_vec())));
                program.push(instr(Opcode::SStore, None));
            }
            // Load and compare
            for i in 0u64..20 {
                program.push(instr(Opcode::Push, Some(u64_to_value(i).to_vec())));
                program.push(instr(Opcode::SLoad, None));
                program.push(instr(Opcode::Push, Some(u64_to_value(0).to_vec())));
                program.push(instr(Opcode::Gt, None));
                program.push(instr(Opcode::Pop, None));
            }
            program.push(instr(Opcode::Halt, None));
            let mut vm = Vm::new(program, 10_000_000);
            black_box(vm.execute().unwrap())
        })
    });
}

criterion_group!(
    benches,
    bench_simple_halt,
    bench_arithmetic_chain,
    bench_push_pop,
    bench_storage_ops,
    bench_comparison_ops,
    bench_dup_swap,
    bench_mixed_workload,
);
criterion_main!(benches);
