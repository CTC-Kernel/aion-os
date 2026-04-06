use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rewind_core::bitplane::BitPlane;
use rewind_core::traits::ReversibleOp;
use rewind_gates::scalar::{Cnot, CnotState, PauliX, Toffoli, ToffoliState};

fn bench_pauli_x(c: &mut Criterion) {
    let mut group = c.benchmark_group("PauliX");

    for num_words in [1, 16, 256, 1024] {
        group.bench_with_input(
            BenchmarkId::new("scalar", num_words),
            &num_words,
            |b, &n| {
                let input = BitPlane::from_words(vec![0xDEADBEEFu64; n]);
                b.iter(|| {
                    let (output, ancilla) = PauliX.execute(black_box(input.clone()));
                    black_box((output, ancilla))
                });
            },
        );
    }
    group.finish();
}

fn bench_cnot(c: &mut Criterion) {
    let mut group = c.benchmark_group("CNOT");

    for num_words in [1, 16, 256, 1024] {
        group.bench_with_input(
            BenchmarkId::new("scalar", num_words),
            &num_words,
            |b, &n| {
                let state = CnotState {
                    control: BitPlane::from_words(vec![0xCAFEBABEu64; n]),
                    target: BitPlane::from_words(vec![0xDEADBEEFu64; n]),
                };
                b.iter(|| {
                    let (output, ancilla) = Cnot.execute(black_box(state.clone()));
                    black_box((output, ancilla))
                });
            },
        );
    }
    group.finish();
}

fn bench_toffoli(c: &mut Criterion) {
    let mut group = c.benchmark_group("Toffoli");

    for num_words in [1, 16, 256, 1024] {
        group.bench_with_input(
            BenchmarkId::new("scalar", num_words),
            &num_words,
            |b, &n| {
                let state = ToffoliState {
                    control1: BitPlane::from_words(vec![0xCAFEBABEu64; n]),
                    control2: BitPlane::from_words(vec![0x12345678u64; n]),
                    target: BitPlane::from_words(vec![0xDEADBEEFu64; n]),
                };
                b.iter(|| {
                    let (output, ancilla) = Toffoli.execute(black_box(state.clone()));
                    black_box((output, ancilla))
                });
            },
        );
    }
    group.finish();
}

fn bench_toffoli_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("Toffoli_throughput");

    // 1024 words = 65,536 bits per gate invocation
    let n = 1024;
    let state = ToffoliState {
        control1: BitPlane::from_words(vec![0xCAFEBABEu64; n]),
        control2: BitPlane::from_words(vec![0x12345678u64; n]),
        target: BitPlane::from_words(vec![0xDEADBEEFu64; n]),
    };

    group.bench_function("1024_words_execute_undo", |b| {
        b.iter(|| {
            let (output, ancilla) = Toffoli.execute(black_box(state.clone()));
            let restored = Toffoli.undo(black_box(output), ancilla);
            black_box(restored)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_pauli_x,
    bench_cnot,
    bench_toffoli,
    bench_toffoli_throughput,
);
criterion_main!(benches);
