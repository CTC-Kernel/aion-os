---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
status: complete
completedAt: '2026-04-06'
inputDocuments:
  - 'prd.md'
  - 'product-brief-aion-os.md'
  - 'product-brief-aion-os-distillate.md'
  - 'research/domain-calcul-reversible-research-2026-04-06.md'
  - 'research/market-aion-os-reversible-computing-research-2026-04-06.md'
  - 'research/technical-aion-os-rvm-rust-feasibility-research-2026-04-06.md'
workflowType: 'architecture'
project_name: 'Rewind (Aion-OS)'
user_name: 'Thibaultllopis'
date: '2026-04-06'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements :**
33 FRs couvrant 8 capacités. L'architecture supporte 3 couches d'exécution :
1. **Compile-time** (FR11-16) : Macro procédurale `#[reversible]` — parsing AST, validation, génération de code forward/backward
2. **Runtime** (FR01-10, FR17-21) : Exécution des portes réversibles, gestion des `QuantumCell`, exécution bidirectionnelle
3. **Memory management** (FR22-28) : Garbage-Free Collector (pile miroir), algorithme de Bennett (graphe de pebbling)

FR29-31 imposent un pattern Strategy via `ExecutionBackend`, découplant le code utilisateur du backend.

**Non-Functional Requirements :**
- **Performance** : >1B portes/sec → data layout SIMD-friendly (SoA)
- **Portabilité** : Nightly + Stable + WASM → feature flags extensifs
- **Testabilité** : Proptest 100K+ → chaque `ReversibleOp` testable indépendamment

**Scale & Complexity :**
- Domaine : SDK Rust avec composant compile-time (proc-macro)
- Complexité : Élevée — 6 sous-systèmes interagissants
- Composants architecturaux : 7 crates workspace Cargo

### Technical Constraints & Dependencies

| Contrainte | Impact Architectural |
|-----------|---------------------|
| Types affines ≠ linéaires | `QuantumCell` via `Drop` + `ManuallyDrop` |
| `std::simd` nightly only | Feature flag `simd` vs `stable-simd` (pulp) |
| Proc-macro = crate séparée | `rewind-dsl` est un proc-macro crate distinct |
| Bennett facteur ε·2^(1/ε) | ε configurable avec garde-fou, budget mémoire |
| Arena allocator | Indices stables dans `typed-arena`/`bumpalo`, pas `Pin<Box<T>>` |
| WASM target | Core `no_std` compatible |

### Cross-Cutting Concerns

| Concern | Composants Impactés | Stratégie |
|---------|---------------------|-----------|
| Performance SIMD | rewind-gates, rewind-core | SoA layout, feature flags, criterion |
| Correctness | Tous | Trait `ReversibleOp` + proptest helper |
| Portabilité | Tous | `no_std` core, feature flags par cible |
| Erreurs utilisateur | rewind-dsl, rewind-core | `syn::Error` + `compile_error!` |
| Abstraction hardware | rewind-core, rewind-gates | Trait `ExecutionBackend` |

---

## Stack Technologique

### Langage & Runtime

| Composant | Choix | Justification |
|-----------|-------|---------------|
| **Langage** | Rust (edition 2024) | Types affines natifs = logique linéaire, zero-cost abstractions, SIMD, macro procédurales |
| **MSRV** | Latest stable (core), nightly (SIMD) | Feature flag pour séparer stable/nightly |
| **Build** | Cargo workspace | Multi-crate, build parallèle, feature flags natifs |
| **Tests** | `proptest` + `cargo test` | Property-based testing pour vérification réversibilité |
| **Benchmarks** | `criterion` | Micro-benchmarks statistiques, rapports HTML |
| **CI/CD** | GitHub Actions | Tests multi-plateforme (Linux, macOS, Windows) + nightly SIMD |
| **Docs** | `rustdoc` + `mdBook` | API docs auto-générées + tutoriels narratifs |
| **Linting** | `clippy` + custom lints dans `#[reversible]` | Qualité code + détection ops irréversibles |

### Dépendances Externes (Minimales)

| Crate | Version | Usage | Justification vs. code propre |
|-------|---------|-------|-------------------------------|
| `syn` | 2.x | Parsing AST dans proc-macro | Standard de facto, 0 alternative viable |
| `quote` | 1.x | Génération code dans proc-macro | Compagnon de syn, standard |
| `proc-macro2` | 1.x | Wrapper proc-macro pour tests | Requis par syn/quote |
| `proptest` | 1.x | Property-based testing | Meilleur que quickcheck (stratégies par valeur) |
| `criterion` | 0.5+ | Benchmarking | Standard Rust, rapports statistiques |
| `petgraph` | 0.6+ | Graphe de calcul Bennett (v0.4+) | DAG mature, pas de raison de réimplémenter |
| `bumpalo` | 3.x | Arena allocator | Plus performant que typed-arena, no_std compatible |

**Politique de dépendances** : Minimum absolu. Chaque dépendance doit être justifiée par une impossibilité ou un coût disproportionné de réimplémentation. `cargo audit` en CI obligatoire.

---

## Décisions Architecturales Fondamentales

### ADR-01 : Workspace Multi-Crate vs Crate Monolithique

**Décision** : Workspace Cargo multi-crate

**Contexte** : Le produit a un composant compile-time (proc-macro) et un runtime. En Rust, les proc-macro crates DOIVENT être des crates séparées (`proc-macro = true`). De plus, la modularité permet `cargo add rewind-gates` sans le DSL.

**Conséquences** :
- 7 crates dans le workspace (voir structure ci-dessous)
- Chaque crate est publiable indépendamment sur crates.io
- La crate façade `rewind` re-exporte tout pour l'utilisateur standard

### ADR-02 : Arena Allocator vs Pin<Box<T>> vs Vec

**Décision** : `bumpalo` arena allocator avec indices stables

**Contexte** : La RVM a besoin d'adresses stables pour les registres et la pile d'ancilla. `Pin<Box<T>>` ajoute de la complexité (Unpin, self-referential) pour un gain nul. Un Vec avec indices stables est simple mais fragmente la mémoire.

**Conséquences** :
- Allocation bulk, cache-friendly
- Indices typés (`RegisterId(u32)`, `AncillaId(u32)`) au lieu de pointeurs
- Pas de `Pin`, pas de structures auto-référentielles
- `no_std` compatible via `bumpalo`

### ADR-03 : Match Dispatch vs Computed Goto vs Trait Objects

**Décision** : Match dispatch (Rust `match` dans une boucle)

**Contexte** : Les VMs Rust utilisent typiquement un `match` sur un enum d'opcodes. Sur CPU modernes (Haswell+), le branch predictor rend le match quasi-optimal. Computed goto n'existe pas en safe Rust. Les trait objects (`dyn ReversibleOp`) ajoutent un indirection pointer et empêchent l'inlining.

**Conséquences** :
- Enum `Opcode` avec toutes les instructions réversibles
- Boucle `loop { match program[pc] { ... } }` — simple, maintenable, performant
- Les opcodes sont des données, pas des objets — sérialisables pour le format RBF futur

### ADR-04 : Data Layout SoA (Structure of Arrays) vs AoS (Array of Structures)

**Décision** : SoA pour les registres de bits (SIMD-friendly)

**Contexte** : Les portes Toffoli opèrent sur des vecteurs de bits. SIMD travaille sur des vecteurs contigus en mémoire. Un layout AoS (chaque registre est un struct avec des bits) cause du scatter/gather SIMD. Un layout SoA (un Vec<u64> par "plan de bits") permet le SIMD natif.

**Conséquences** :
- `BitPlane` : `Vec<u64>` (ou `Vec<Simd<u64, N>>` en mode SIMD)
- Toffoli opère sur 3 `BitPlane` en parallèle
- Excellent cache locality pour les opérations bitwise massives
- Moins intuitif pour le code utilisateur — la crate `rewind-gates` abstrait le layout

### ADR-05 : Linéarité via Drop+Panic vs Link-Time Error vs Crate linear_type

**Décision** : `Drop` + `panic!` + `ManuallyDrop` pour v0.1, évaluation `linear_type` crate en v0.2

**Contexte** : Rust n'a pas de types linéaires natifs. Le pattern `Drop` + panic est documenté, compris, et suffisant pour un prototype. La crate `linear_type` offre une abstraction mais ajoute une dépendance. Le link-time error est trop cryptique.

**Conséquences** :
- `QuantumCell<T>` implémente `Drop` qui panique si non consommé
- `consume(self) -> T` utilise `ManuallyDrop` pour bypass le `Drop`
- `mem::forget` contourne le mécanisme — interdit dans `#[reversible]` par la macro
- Migration vers `linear_type` ou futur trait `Leave` possible sans casser l'API

### ADR-06 : Feature Flags pour Portabilité

**Décision** : Architecture feature-flag extensive

```toml
[features]
default = []
simd = []           # Active std::simd (nightly)
stable-simd = ["pulp"]  # Active pulp (stable)
bennett = ["petgraph"]  # Active algorithme de Bennett
wasm = []           # Active WASM target
```

**Conséquences** :
- Le core compile partout sans feature flag
- SIMD est opt-in (nightly ou stable via pulp)
- Bennett est opt-in (ajoute petgraph)
- Le Cargo.toml de l'utilisateur détermine les capacités activées

---

## Patterns d'Implémentation & Règles de Cohérence

### Patterns de Nommage

| Contexte | Convention | Exemple |
|----------|-----------|---------|
| Crates | `rewind-{module}` | `rewind-core`, `rewind-gates` |
| Types publics | PascalCase, descriptif | `QuantumCell<T>`, `ReversibleBlock` |
| Traits | PascalCase, verbe/adjectif | `ReversibleOp`, `ExecutionBackend` |
| Fonctions | snake_case, verbe d'action | `forward()`, `backward()`, `checkpoint()` |
| Opcodes | PascalCase dans l'enum | `Opcode::Toffoli`, `Opcode::Checkpoint` |
| Indices typés | PascalCase + newtype | `RegisterId(u32)`, `AncillaId(u32)` |
| Feature flags | kebab-case | `stable-simd`, `bennett` |
| Erreurs | PascalCase + Error suffix | `RewindError`, `EntropyError` |

### Patterns de Structure

**Chaque crate suit :**
```
rewind-{module}/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Re-exports publics + doc module
│   ├── {feature}.rs    # Un fichier par feature principal
│   └── tests/          # Tests d'intégration (si nécessaires)
└── benches/            # Benchmarks criterion (si applicable)
```

### Pattern ReversibleOp — Le Contrat Central

```rust
/// Tout composant de calcul réversible implémente ce trait.
/// La propriété fondamentale : ∀x: undo(execute(x)) == x
pub trait ReversibleOp {
    type State;
    type Ancilla;  // Preuve d'inversion (bits auxiliaires)

    fn execute(&self, state: Self::State) -> (Self::State, Self::Ancilla);
    fn undo(&self, state: Self::State, ancilla: Self::Ancilla) -> Self::State;
}
```

**Règle** : TOUTE implémentation de `ReversibleOp` DOIT avoir un test proptest associé vérifiant la propriété de réversibilité.

### Patterns d'Erreur

```rust
#[derive(Debug, thiserror::Error)]
pub enum RewindError {
    #[error("QuantumCell dropped without being consumed — information lost")]
    InformationLost,
    #[error("Checkpoint {0} not found")]
    CheckpointNotFound(CheckpointId),
    #[error("Ancilla stack not empty after uncomputation — {0} bits remain")]
    GarbageRemaining(usize),
    #[error("Memory budget exceeded: {used} > {limit} bytes")]
    MemoryBudgetExceeded { used: usize, limit: usize },
}
```

### Anti-Patterns Interdits

| Anti-Pattern | Pourquoi | Alternative |
|-------------|----------|------------|
| `Clone` sur `QuantumCell` | Détruit la linéarité | `consume()` + recréer |
| `mem::forget` dans `#[reversible]` | Contourne le Drop | Rejeté par la macro |
| `println!` dans `#[reversible]` | Effet de bord irréversible | Logger hors du bloc réversible |
| `Pin<Box<T>>` pour les registres | Complexité inutile | Arena + indices typés |
| `dyn ReversibleOp` pour dispatch | Empêche inlining | Enum + match dispatch |
| Assignation destructive `x = y` | Perte d'information | `x ^= y`, `x += y`, `swap` |

---

## Structure du Projet & Frontières

### Arbre Complet du Projet

```
rewind/                              # Workspace root
├── Cargo.toml                       # Workspace manifest
├── LICENSE-APACHE                   # Apache 2.0
├── LICENSE-MIT                      # MIT
├── README.md                        # GIF + quickstart + pitch
├── CONTRIBUTING.md                  # Guide contribution
│
├── rewind-core/                     # Types fondamentaux + traits
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                   # Re-exports: QuantumCell, ReversibleOp, ExecutionBackend
│       ├── cell.rs                  # QuantumCell<T> implémentation
│       ├── traits.rs                # ReversibleOp, ExecutionBackend traits
│       ├── backend.rs               # SimulatedCPU backend par défaut
│       ├── state.rs                 # State, CheckpointId, RegisterId types
│       ├── error.rs                 # RewindError enum
│       └── bitplane.rs              # BitPlane (SoA data layout pour SIMD)
│
├── rewind-gates/                    # Portes logiques réversibles
│   ├── Cargo.toml                   # Features: simd, stable-simd
│   ├── src/
│   │   ├── lib.rs                   # Re-exports: pauli_x, cnot, toffoli
│   │   ├── scalar.rs                # Implémentations scalaires (u64)
│   │   ├── simd.rs                  # Implémentations SIMD (std::simd)
│   │   └── stable_simd.rs           # Implémentations SIMD stable (pulp)
│   └── benches/
│       └── gates_benchmark.rs       # Criterion benchmarks toutes portes
│
├── rewind-gc/                       # Garbage-Free Collector
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                   # GarbageFreeCollector, AncillaStack
│       ├── stack.rs                 # Pile miroir LIFO
│       ├── budget.rs                # Budget mémoire configurable
│       └── square.rs                # Strat��gie SQUARE (v0.5)
│
├── rewind-dsl/                      # Macro procédurale #[reversible]
│   ├── Cargo.toml                   # proc-macro = true
│   └── src/
│       ├── lib.rs                   # #[reversible] entry point
│       ├── parse.rs                 # Parsing AST (syn)
│       ├─�� validate.rs              # Validation réversibilité
│       ├── codegen.rs               # Génération forward + backward (quote)
│       └── errors.rs                # Messages d'erreur compile-time
│
├── rewind-bennett/                  # Algorithme de Bennett (v0.4+)
│   ├── Cargo.toml                   # Depends on petgraph
│   └── src/
│       ├── lib.rs                   # BennettCompiler
│       ├── graph.rs                 # ComputationGraph (DAG)
│       ├── pebbling.rs              # PebblingStrategy
│       └── executor.rs              # ReversibleExecutor
│
├── rewind/                          # Crate façade (re-export tout)
│   ├── Cargo.toml                   # Features: simd, stable-simd, bennett
│   └── src/
│       └── lib.rs                   # pub use rewind_core::*; etc.
│
├── examples/                        # Exemples exécutables
│   ├── hello_rewind.rs              # Quickstart minimal
│   ├── step_backward.rs             # Demo debugging temporel
│   └── toffoli_circuit.rs           # Circuit Toffoli complet
│
└── tests/                           # Tests d'intégration workspace
    ├── reversibility_props.rs       # Proptest: toutes les portes
    └── end_to_end.rs                # Test complet forward/backward
```

### Frontières Architecturales

```
┌─────────────────────────────────────────────────────────────┐
│                     COMPILE-TIME                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  rewind-dsl (#[reversible])                          │   │
│  │  parse.rs → validate.rs → codegen.rs                 │   │
│  │  INPUT: TokenStream  OUTPUT: TokenStream              │   │
│  └──────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                      RUNTIME                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ rewind-core  │  │ rewind-gates │  │ rewind-gc        │  │
│  │ QuantumCell  │◄─│ Toffoli/CNOT │──│ AncillaStack     │  │
│  │ ReversibleOp │  │ scalar/SIMD  │  │ GarbageFreeCollector│ │
│  │ Backend trait│  └──────────────┘  └──────────────────┘  │
│  └──────┬───────┘                                           │
│         │ impl ExecutionBackend                              │
│  ┌──────▼───────┐  ┌──────────────┐                        │
│  │ SimulatedCPU │  │ VaireBackend │ (futur)                │
│  │ (défaut)     │  │ FPGABackend  │ (futur)                │
│  └──────────────┘  └──────────────┘                        │
├─────────────────────────────────────────────────────────────┤
│                    COMPILATION RC                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  rewind-bennett (v0.4+)                              │   │
│  │  ComputationGraph → PebblingStrategy → Executor       │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Flux de Données Principal

```
Code utilisateur
    │
    ▼ compile-time
#[reversible] (rewind-dsl)
    │ parse → validate → codegen
    ▼
ReversibleBlock { forward_ops: Vec<Opcode>, backward_ops: Vec<Opcode> }
    │
    ▼ runtime
ExecutionEngine (rewind-core)
    │
    ├──▶ forward(): exécute forward_ops séquentiellement
    │     │ pour chaque op: gate.execute(state) → (new_state, ancilla)
    │     │ push ancilla sur AncillaStack (rewind-gc)
    │     ▼
    │   Output State
    │
    └──▶ backward(): exécute backward_ops séquentiellement
          │ pour chaque op: pop ancilla, gate.undo(state, ancilla)
          ▼
        Original Input State (= vérifiable par proptest)
```

---

## Validation Architecturale

### Cohérence des Décisions

| ADR | Compatible avec | Vérifié |
|-----|----------------|---------|
| ADR-01 (workspace) | ADR-06 (feature flags) — chaque crate a ses propres features | ✅ |
| ADR-02 (arena) | ADR-04 (SoA) — BitPlane alloué dans l'arena | ✅ |
| ADR-03 (match dispatch) | ADR-04 (SoA) — opcodes opèrent sur BitPlanes | ✅ |
| ADR-05 (Drop+panic) | ADR-01 (workspace) — QuantumCell dans rewind-core | ✅ |
| ADR-06 (feature flags) | ADR-03 (match) — opcodes SIMD conditionnels | ✅ |

### Couverture des Exigences

| Exigence | Composant Architectural | Couvert |
|----------|------------------------|---------|
| FR01-04 (QuantumCell) | rewind-core/cell.rs | ✅ |
| FR05-10 (Portes) | rewind-gates/ | ✅ |
| FR11-16 (#[reversible]) | rewind-dsl/ | ✅ |
| FR17-21 (Forward/Backward) | rewind-core/backend.rs | ✅ |
| FR22-25 (GC) | rewind-gc/ | ✅ |
| FR26-28 (Bennett) | rewind-bennett/ | ✅ |
| FR29-31 (Backend) | rewind-core/traits.rs | ✅ |
| FR32-33 (Tests) | tests/ + proptest | ✅ |
| NFR01-04 (Performance) | rewind-gates/simd.rs + criterion | ✅ |
| NFR05-07 (Fiabilité) | proptest + CI + unsafe minimal | ✅ |
| NFR08-10 (Portabilité) | feature flags + no_std | ✅ |
| NFR11-13 (DX) | rustdoc + mdBook + syn::Error | ✅ |
| NFR14-15 (Sécurité) | cargo audit + dépendances minimales | ✅ |

### Analyse des Gaps

| Gap Potentiel | Sévérité | Mitigation |
|--------------|----------|------------|
| Pas de benchmarks WASM | Basse | Ajouter en v0.3 quand playground est construit |
| Pas de format de sérialisation RBF défini | Moyenne | Spec RBF en v0.5, utiliser serde + bincode en interne |
| Pas de mécanisme de migration entre versions | Basse | Semver strict, pas de persistence d'état entre sessions |
| Pas de logging/tracing dans le runtime | Basse | Ajouter `tracing` feature flag optionnel |

### Checklist de Complétude

- [x] Stack technologique défini avec justifications
- [x] 6 ADRs documentés (workspace, arena, dispatch, SoA, linéarité, features)
- [x] Patterns de nommage, structure, erreur, et anti-patterns
- [x] Arbre complet du projet avec tous les fichiers
- [x] Frontières architecturales (compile-time / runtime / compilation RC)
- [x] Flux de données principal documenté
- [x] Couverture FR/NFR vérifiée (33/33 FRs, 15/15 NFRs)
- [x] Cohérence inter-ADR validée
- [x] Gaps identifiés avec mitigations

### Évaluation de Maturité

**L'architecture est prête pour l'implémentation.** Tous les composants sont définis, les fronti��res sont claires, les décisions sont justifiées, et les 33 FRs + 15 NFRs sont couverts. La séquence d'implémentation recommandée :

1. `rewind-core` (QuantumCell, traits, state, error, BitPlane)
2. `rewind-gates` (scalaire d'abord, SIMD ensuite)
3. `rewind-dsl` (#[reversible] sous-ensemble)
4. `rewind` (crate façade + exemples)
5. `rewind-gc` (v0.4)
6. `rewind-bennett` (v0.4)

Chaque étape est testable indépendamment avec proptest.
