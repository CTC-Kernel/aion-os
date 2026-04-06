//! Serde serialization tests for reversible programs.

#[cfg(feature = "serde")]
mod serde_tests {
    use rewind::prelude::*;

    #[test]
    fn serialize_deserialize_op_json() {
        let op = Op::Toffoli {
            c1: 0,
            c2: 1,
            target: 2,
        };
        let json = serde_json::to_string(&op).unwrap();
        let deserialized: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(op, deserialized);
    }

    #[test]
    fn serialize_deserialize_program_json() {
        let program = ProgramBuilder::new()
            .not(0)
            .cnot(0, 1)
            .toffoli(0, 1, 2)
            .fredkin(0, 1, 2)
            .build();

        let json = serde_json::to_string_pretty(&program).unwrap();
        let deserialized: ReversibleProgram = serde_json::from_str(&json).unwrap();
        assert_eq!(program, deserialized);
    }

    #[test]
    fn serialize_deserialize_bitplane_json() {
        let bp = BitPlane::from(0xDEADBEEFu64);
        let json = serde_json::to_string(&bp).unwrap();
        let deserialized: BitPlane = serde_json::from_str(&json).unwrap();
        assert_eq!(bp, deserialized);
    }

    #[test]
    fn serialized_program_is_executable() {
        let program = ProgramBuilder::new()
            .not(0)
            .cnot(0, 1)
            .toffoli(0, 1, 2)
            .build();

        let json = serde_json::to_string(&program).unwrap();
        let loaded: ReversibleProgram = serde_json::from_str(&json).unwrap();

        let mut rt = ReversibleRuntime::new(vec![
            BitPlane::from(0xAAu64),
            BitPlane::from(0xBBu64),
            BitPlane::from(0xCCu64),
        ]);
        let original = rt.registers().to_vec();

        rt.run_forward(&loaded);
        rt.rewind_all().unwrap();
        assert_eq!(rt.registers().to_vec(), original);
    }
}
