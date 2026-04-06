# Story 1.1: Initialisation du Workspace Cargo

Status: review

## Story

As a développeur Rewind,
I want un workspace Cargo multi-crate correctement structuré avec les 7 crates du projet,
so that je peux développer les composants de manière modulaire, indépendante, et publiable sur crates.io.

## Acceptance Criteria

1. Le workspace compile sans erreur avec `cargo build` contenant les 7 crates : rewind-core, rewind-gates, rewind-gc, rewind-dsl, rewind-bennett, rewind (façade), rewind-playground
2. Chaque crate a un Cargo.toml valide avec les dépendances inter-crates correctes
3. Les fichiers LICENSE-APACHE et LICENSE-MIT sont à la racine du workspace
4. Le README.md contient le pitch "Information is Sacred", une description du projet, et un placeholder pour le quickstart
5. La crate `rewind` (façade) re-exporte les modules publics de rewind-core et rewind-gates
6. La crate `rewind-dsl` est configurée comme proc-macro crate (`proc-macro = true`)
7. Le workspace utilise Rust edition 2024 pour toutes les crates
8. `cargo test` passe sur le workspace complet (même si les tests sont vides/triviaux pour l'instant)
9. Le `.gitignore` est configuré pour Rust (target/, Cargo.lock pour libraries)

## Tasks / Subtasks

- [x] Task 1: Créer le workspace root (AC: #1, #7, #9)
  - [x] Créer `Cargo.toml` workspace avec members list des 7 crates
  - [x] Configurer `[workspace.package]` avec edition = "2024", license, repository, description
  - [x] Créer `.gitignore` pour Rust (target/, *.swp, .DS_Store)
  - [x] NE PAS ignorer Cargo.lock (c'est une application/workspace, pas une lib standalone)

- [x] Task 2: Créer les fichiers de licence (AC: #3)
  - [x] Créer `LICENSE-APACHE` avec le texte Apache License 2.0
  - [x] Créer `LICENSE-MIT` avec le texte MIT License, copyright Thibaultllopis 2026

- [x] Task 3: Créer rewind-core (AC: #1, #2)
  - [x] Créer `rewind-core/Cargo.toml` héritant du workspace.package
  - [x] Créer `rewind-core/src/lib.rs` avec doc comment et modules placeholder : `pub mod cell;`, `pub mod traits;`, `pub mod backend;`, `pub mod state;`, `pub mod error;`, `pub mod bitplane;`
  - [x] Créer les fichiers modules vides avec doc comments : cell.rs, traits.rs, backend.rs, state.rs, error.rs, bitplane.rs
  - [x] Vérifier que `cargo build -p rewind-core` compile

- [x] Task 4: Créer rewind-gates (AC: #1, #2)
  - [x] Créer `rewind-gates/Cargo.toml` avec dépendance sur rewind-core, feature flags `simd` et `stable-simd`
  - [x] Créer `rewind-gates/src/lib.rs` avec modules placeholder : `pub mod scalar;`, `#[cfg(feature = "simd")] pub mod simd;`, `#[cfg(feature = "stable-simd")] pub mod stable_simd;`
  - [x] Créer les fichiers modules vides : scalar.rs, simd.rs, stable_simd.rs
  - [x] Vérifier que `cargo build -p rewind-gates` compile

- [x] Task 5: Créer rewind-gc (AC: #1, #2)
  - [x] Créer `rewind-gc/Cargo.toml` avec dépendance sur rewind-core
  - [x] Créer `rewind-gc/src/lib.rs` avec modules placeholder : `pub mod stack;`, `pub mod budget;`
  - [x] Créer les fichiers modules vides : stack.rs, budget.rs
  - [x] Vérifier que `cargo build -p rewind-gc` compile

- [x] Task 6: Créer rewind-dsl (AC: #1, #2, #6)
  - [x] Créer `rewind-dsl/Cargo.toml` avec `[lib] proc-macro = true` et dépendances syn, quote, proc-macro2
  - [x] Créer `rewind-dsl/src/lib.rs` avec un proc-macro placeholder `#[proc_macro_attribute] pub fn reversible(...)` qui retourne le TokenStream non modifié (pass-through)
  - [x] Vérifier que `cargo build -p rewind-dsl` compile

- [x] Task 7: Créer rewind-bennett (AC: #1, #2)
  - [x] Créer `rewind-bennett/Cargo.toml` avec dépendance sur rewind-core, feature flag `bennett` optionnel
  - [x] Créer `rewind-bennett/src/lib.rs` avec modules placeholder : `pub mod graph;`, `pub mod pebbling;`, `pub mod executor;`
  - [x] Créer les fichiers modules vides : graph.rs, pebbling.rs, executor.rs
  - [x] Vérifier que `cargo build -p rewind-bennett` compile

- [x] Task 8: Créer rewind (façade) (AC: #1, #2, #5)
  - [x] Créer `rewind/Cargo.toml` avec dépendances sur rewind-core, rewind-gates, rewind-dsl, feature flags passthrough (simd, stable-simd, bennett)
  - [x] Créer `rewind/src/lib.rs` avec re-exports : `pub use rewind_core::*;`, `pub use rewind_gates::*;`, `pub use rewind_dsl::*;`
  - [x] Vérifier que `cargo build -p rewind` compile

- [x] Task 9: Créer rewind-playground (AC: #1, #2)
  - [x] Créer `rewind-playground/Cargo.toml` avec dépendance sur rewind (façade)
  - [x] Créer `rewind-playground/src/lib.rs` avec doc comment placeholder "WASM playground - future"
  - [x] Vérifier que `cargo build -p rewind-playground` compile

- [x] Task 10: Créer le README.md (AC: #4)
  - [x] Créer `README.md` avec : titre "Rewind", tagline "Information is Sacred", description du projet (SDK calcul nativement réversible en Rust), section "Quick Start" placeholder, badges placeholder (license, CI), lien vers les crates
  - [x] Créer `CONTRIBUTING.md` avec guide de contribution basique

- [x] Task 11: Validation complète du workspace (AC: #1, #8)
  - [x] Exécuter `cargo build` — doit compiler sans erreur ni warning
  - [x] Exécuter `cargo test` — doit passer (tests vides OK)
  - [x] Exécuter `cargo clippy` — doit passer sans warning
  - [x] Vérifier que `cargo doc --no-deps` génère la documentation

## Dev Notes

### Architecture Requirements

**Source** : [architecture.md — ADR-01 Workspace Multi-Crate]

Le workspace suit l'architecture définie dans le document d'architecture :

```
rewind/                              # Workspace root
├── Cargo.toml                       # Workspace manifest
├── LICENSE-APACHE
├── LICENSE-MIT
├── README.md
├── CONTRIBUTING.md
├── .gitignore
├── rewind-core/                     # Types fondamentaux + traits
├── rewind-gates/                    # Portes logiques réversibles
├── rewind-gc/                       # Garbage-Free Collector
├── rewind-dsl/                      # Macro procédurale #[reversible]
├── rewind-bennett/                  # Algorithme de Bennett
├── rewind/                          # Crate façade (re-export tout)
└── rewind-playground/               # WASM playground (futur)
```

### Dépendances Inter-Crates

```
rewind (façade) ──► rewind-core
                ──► rewind-gates ──► rewind-core
                ──► rewind-dsl
rewind-gc ──────────► rewind-core
rewind-bennett ─────► rewind-core
rewind-playground ──► rewind (façade)
```

### Feature Flags Architecture

**Source** : [architecture.md — ADR-06 Feature Flags]

```toml
# rewind-gates/Cargo.toml
[features]
default = []
simd = []              # Active std::simd (nightly)
stable-simd = ["pulp"] # Active pulp (stable)

# rewind/Cargo.toml (façade — passthrough)
[features]
default = []
simd = ["rewind-gates/simd"]
stable-simd = ["rewind-gates/stable-simd"]
bennett = ["dep:rewind-bennett"]
```

### Conventions de Nommage

**Source** : [architecture.md — Patterns de Nommage]

- Crate names : `rewind-{module}` (kebab-case)
- Package names dans Rust : `rewind_{module}` (snake_case, auto-converti par Cargo)
- Tous les modules publics ont un doc comment `//!` en tête de fichier

### Ce Qui N'est PAS dans cette Story

- Aucune implémentation de QuantumCell, ReversibleOp, ou des portes (Stories 1.2-2.3)
- Aucun test fonctionnel (seulement la validation que `cargo test` passe)
- Aucune logique business — uniquement la structure et le scaffolding
- Aucune dépendance externe sauf syn/quote/proc-macro2 pour rewind-dsl

### Versions de Dépendances

- `syn` : "2" (dernière version majeure stable)
- `quote` : "1" (dernière version majeure stable)
- `proc-macro2` : "1" (dernière version majeure stable)
- Rust edition : 2024

### Project Structure Notes

- Ce workspace est créé dans le répertoire racine du projet `/Users/thibaultllopis/aion-os/`
- Les fichiers `src/` et `Cargo.toml` existants à la racine devront être remplacés par la structure workspace
- Le `Cargo.lock` existant peut être supprimé et régénéré par `cargo build`

### References

- [Source: architecture.md — ADR-01 Workspace Multi-Crate]
- [Source: architecture.md — ADR-06 Feature Flags]
- [Source: architecture.md — Patterns de Nommage]
- [Source: architecture.md — Structure du Projet]
- [Source: prd.md — Périmètre MVP v0.1]
- [Source: epics.md — Epic 1, Story 1.1]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

Aucun problème rencontré.

### Completion Notes List

- Workspace Cargo créé avec 7 crates compilant sans erreur
- Ancien code single-crate (src/) supprimé (préservé dans git history)
- Toutes les dépendances inter-crates configurées correctement
- rewind-dsl configuré comme proc-macro avec placeholder pass-through
- Feature flags (simd, stable-simd, bennett) configurés avec passthrough dans la façade
- Licences Apache 2.0 + MIT créées
- README avec pitch "Information is Sacred" et table des crates
- CONTRIBUTING.md avec guide de développement
- cargo build ✅, cargo test ✅, cargo clippy ✅ (0 warnings), cargo doc ✅ (0 warnings)

### File List

- Cargo.toml (workspace root — créé)
- .gitignore (mis à jour)
- LICENSE-MIT (créé)
- LICENSE-APACHE (créé)
- README.md (créé)
- CONTRIBUTING.md (créé)
- rewind-core/Cargo.toml (créé)
- rewind-core/src/lib.rs (créé)
- rewind-core/src/cell.rs (créé)
- rewind-core/src/traits.rs (créé)
- rewind-core/src/backend.rs (créé)
- rewind-core/src/state.rs (créé)
- rewind-core/src/error.rs (créé)
- rewind-core/src/bitplane.rs (créé)
- rewind-gates/Cargo.toml (créé)
- rewind-gates/src/lib.rs (créé)
- rewind-gates/src/scalar.rs (créé)
- rewind-gates/src/simd.rs (créé)
- rewind-gates/src/stable_simd.rs (créé)
- rewind-gc/Cargo.toml (créé)
- rewind-gc/src/lib.rs (créé)
- rewind-gc/src/stack.rs (créé)
- rewind-gc/src/budget.rs (créé)
- rewind-dsl/Cargo.toml (créé)
- rewind-dsl/src/lib.rs (créé)
- rewind-bennett/Cargo.toml (créé)
- rewind-bennett/src/lib.rs (créé)
- rewind-bennett/src/graph.rs (créé)
- rewind-bennett/src/pebbling.rs (créé)
- rewind-bennett/src/executor.rs (créé)
- rewind/Cargo.toml (créé)
- rewind/src/lib.rs (créé)
- rewind-playground/Cargo.toml (créé)
- rewind-playground/src/lib.rs (créé)
- src/ (supprimé — ancien code single-crate)
- Cargo.lock (supprimé et régénéré)

### Change Log

- 2026-04-06: Story 1.1 complète — Workspace Cargo créé avec 7 crates, licences, README, validation complète (build, test, clippy, doc)
