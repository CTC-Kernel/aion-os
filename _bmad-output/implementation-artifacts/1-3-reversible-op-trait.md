# Story 1.3: Trait ReversibleOp et Types d'État

Status: ready-for-dev

## Story

As a développeur Rewind,
I want un trait ReversibleOp qui définit le contrat de réversibilité et des types d'état typés,
so that toute opération du système est garantie inversible et les indices sont sûrs au compile-time.

## Acceptance Criteria

1. Le trait `ReversibleOp` définit `execute()` et `undo()` avec types associés `State` et `Ancilla`
2. La propriété fondamentale `∀x: undo(execute(x)) == x` est testable via un helper proptest
3. Les types `RegisterId(u32)`, `AncillaId(u32)`, `CheckpointId(u32)` empêchent la confusion d'indices
4. `BitPlane` stocke des bits dans un `Vec<u64>` avec les opérations XOR, AND, NOT
5. `RewindError` couvre les erreurs : InformationLost, CheckpointNotFound, GarbageRemaining, MemoryBudgetExceeded
6. Tous les types sont testés avec proptest et documentés avec rustdoc

## Tasks / Subtasks

- [ ] Task 1: Implémenter ReversibleOp trait (AC: #1)
- [ ] Task 2: Implémenter les types d'état typés (AC: #3)
- [ ] Task 3: Implémenter BitPlane (AC: #4)
- [ ] Task 4: Implémenter RewindError (AC: #5)
- [ ] Task 5: Helper proptest assert_reversible! (AC: #2)
- [ ] Task 6: Mettre à jour les re-exports dans lib.rs
- [ ] Task 7: Tests + validation complète (AC: #6)

## Dev Notes

### Fichiers à modifier
- rewind-core/src/traits.rs — ReversibleOp trait
- rewind-core/src/state.rs — RegisterId, AncillaId, CheckpointId
- rewind-core/src/bitplane.rs — BitPlane
- rewind-core/src/error.rs — RewindError
- rewind-core/src/lib.rs — re-exports

## Dev Agent Record

### Agent Model Used
### Completion Notes List
### File List
### Change Log
