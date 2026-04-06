# Story 1.2: QuantumCell — Type Linéaire

Status: ready-for-dev

## Story

As a développeur Rust,
I want créer des QuantumCell<T> qui ne peuvent être ni copiées ni détruites implicitement,
so that mes données réversibles respectent la logique linéaire (usage exactement une fois) et que toute perte d'information est détectée.

## Acceptance Criteria

1. `QuantumCell::new(value)` crée une cellule encapsulant la valeur
2. `QuantumCell` ne peut pas être `Clone` ni `Copy` — le compilateur rejette toute tentative
3. Si une `QuantumCell` sort du scope sans avoir été consommée via `consume()`, le programme panique avec le message "QuantumCell dropped without being consumed — information lost"
4. `consume(self) -> T` retourne la valeur intérieure sans déclencher le panic du Drop
5. `QuantumCell` implémente `Debug` (pour le debugging) mais PAS `Clone`, `Copy`, `Default`, ou `PartialEq`
6. L'API est ergonomique : `borrow()` retourne `&T` pour inspection sans consommation, `borrow_mut()` retourne `&mut T` pour modification en place
7. Les tests proptest vérifient la propriété fondamentale : créer une cellule, la consommer, pas de panic
8. Un test vérifie que dropper sans consume() panique bien
9. La documentation rustdoc est complète avec exemples exécutables

## Tasks / Subtasks

- [ ] Task 1: Implémenter QuantumCell<T> (AC: #1, #2, #4, #5)
  - [ ] Dans `rewind-core/src/cell.rs`, définir `pub struct QuantumCell<T>` avec champs `value: ManuallyDrop<T>` et `consumed: bool`
  - [ ] Implémenter `QuantumCell::new(value: T) -> Self` qui wrap la valeur dans ManuallyDrop
  - [ ] Implémenter `consume(self) -> T` qui utilise `ManuallyDrop::into_inner` et met consumed=true via un pattern unsafe minimal
  - [ ] NE PAS dériver ni implémenter Clone, Copy, Default, PartialEq
  - [ ] Implémenter Debug manuellement (afficher "QuantumCell(consumed: {bool})" sans exposer la valeur)

- [ ] Task 2: Implémenter le Drop qui panique (AC: #3)
  - [ ] Implémenter `Drop for QuantumCell<T>` qui vérifie `self.consumed`
  - [ ] Si consumed == false : `panic!("QuantumCell dropped without being consumed — information lost")`
  - [ ] Si consumed == true : drop proprement la valeur via `ManuallyDrop::drop`
  - [ ] ATTENTION : le pattern consume(self) doit empêcher le Drop de se déclencher — utiliser `std::mem::forget` sur self APRÈS extraction de la valeur, OU utiliser un flag consumed

- [ ] Task 3: Implémenter borrow et borrow_mut (AC: #6)
  - [ ] Implémenter `borrow(&self) -> &T` qui retourne une référence à la valeur intérieure
  - [ ] Implémenter `borrow_mut(&mut self) -> &mut T` qui retourne une référence mutable
  - [ ] Ces méthodes ne consomment PAS la cellule — elles permettent l'inspection/modification

- [ ] Task 4: Mettre à jour les re-exports (AC: pas spécifique)
  - [ ] Dans `rewind-core/src/lib.rs`, ajouter `pub use cell::QuantumCell;`
  - [ ] Vérifier que `rewind::QuantumCell` est accessible via la façade

- [ ] Task 5: Écrire les tests unitaires (AC: #7, #8)
  - [ ] Test `new_and_consume` : créer une QuantumCell, la consommer, vérifier la valeur retournée
  - [ ] Test `drop_without_consume_panics` : utiliser `#[should_panic(expected = "information lost")]` pour vérifier le panic
  - [ ] Test `borrow_does_not_consume` : emprunter la valeur, puis consommer — pas de panic
  - [ ] Test `borrow_mut_modifies_value` : modifier via borrow_mut, consommer, vérifier la nouvelle valeur
  - [ ] Test proptest : pour 10000+ valeurs u64 aléatoires, `QuantumCell::new(x).consume() == x`

- [ ] Task 6: Écrire la documentation rustdoc (AC: #9)
  - [ ] Doc comment module `//!` en tête de cell.rs avec explication du concept
  - [ ] Doc comment sur la struct avec exemple `# Examples`
  - [ ] Doc comments sur chaque méthode publique (new, consume, borrow, borrow_mut)
  - [ ] Vérifier que `cargo doc --no-deps` compile sans warnings
  - [ ] Vérifier que les exemples rustdoc sont exécutables (pas de `ignore`)

- [ ] Task 7: Validation complète
  - [ ] `cargo build -p rewind-core` compile sans erreur
  - [ ] `cargo test -p rewind-core` — tous les tests passent
  - [ ] `cargo clippy -p rewind-core` — 0 warnings
  - [ ] `cargo test` (workspace complet) — pas de régression

## Dev Notes

### Architecture Requirements

**Source** : [architecture.md — ADR-05 Linéarité via Drop+Panic]

Le pattern choisi est `Drop` + `panic!` + `ManuallyDrop` :
- `ManuallyDrop<T>` empêche le drop automatique de la valeur intérieure
- Un flag `consumed: bool` track si la cellule a été consommée
- `Drop` vérifie le flag et panique si non consommé
- `consume(self)` extrait la valeur et empêche le Drop de paniquer

### Pattern d'Implémentation Détaillé

```rust
use std::mem::ManuallyDrop;

pub struct QuantumCell<T> {
    value: ManuallyDrop<T>,
    consumed: bool,
}

impl<T> QuantumCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: ManuallyDrop::new(value),
            consumed: false,
        }
    }

    pub fn consume(mut self) -> T {
        self.consumed = true;
        // SAFETY: consumed is set to true, so Drop won't try to use the value.
        // We take ownership here and ManuallyDrop ensures no double-free.
        unsafe { ManuallyDrop::take(&mut self.value) }
    }

    pub fn borrow(&self) -> &T {
        &self.value
    }

    pub fn borrow_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Drop for QuantumCell<T> {
    fn drop(&mut self) {
        if !self.consumed {
            panic!("QuantumCell dropped without being consumed — information lost");
        }
        // If consumed, value was already taken via ManuallyDrop::take — nothing to drop
    }
}
```

### Limitation Connue

`std::mem::forget` peut contourner le Drop et donc la vérification de linéarité. C'est safe en Rust (mem::forget n'est pas unsafe). La mitigation viendra en Epic 4 quand `#[reversible]` interdira mem::forget dans son scope.

### Dépendances

- Aucune dépendance externe nécessaire
- `proptest` sera ajouté comme dev-dependency dans `rewind-core/Cargo.toml`

### Fichiers à Modifier/Créer

| Fichier | Action |
|---------|--------|
| `rewind-core/src/cell.rs` | Remplacer le placeholder par l'implémentation complète |
| `rewind-core/src/lib.rs` | Ajouter `pub use cell::QuantumCell;` |
| `rewind-core/Cargo.toml` | Ajouter `proptest` en dev-dependency |

### Learnings de Story 1.1

- Le workspace compile et passe clippy/test/doc sans warnings
- Les modules dans rewind-core sont des placeholders (doc comments seulement)
- La façade `rewind` re-exporte tout de rewind-core via `pub use rewind_core::*;`
- Edition 2024, pas de dépendances externes dans rewind-core encore

### Testing Standards

**Source** : [architecture.md — Pattern ReversibleOp]

- TOUT type dans rewind-core doit avoir des tests unitaires
- Proptest pour les propriétés fondamentales (ici : consume retourne la même valeur)
- `#[should_panic]` pour vérifier les panics attendus
- Les tests sont dans le même fichier (`#[cfg(test)] mod tests`)

### References

- [Source: architecture.md — ADR-05 Linéarité via Drop+Panic]
- [Source: architecture.md — Patterns d'Erreur]
- [Source: prd.md — FR01, FR02, FR03, FR04]
- [Source: epics.md — Epic 1, Story 1.2]
- [Source: technical-research — Défi 1 QuantumCell linéaire]
- [Source: 1-1-workspace-cargo-init.md — Story précédente]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log
