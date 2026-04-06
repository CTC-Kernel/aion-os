---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments: []
workflowType: 'research'
lastStep: 1
research_type: 'technical'
research_topic: 'Faisabilité RVM Aion-OS en Rust — Types Linéaires, SIMD Toffoli, Macro DSL, Uncomputation'
research_goals: 'Valider la faisabilité technique des 6 défis identifiés pour implémenter une Reversible Virtual Machine en Rust : QuantumCell linéaire, portes SIMD, aion_block! macro, Garbage-Free Collector, algorithme de Bennett, Pin<Box<T>>'
user_name: 'Thibaultllopis'
date: '2026-04-06'
web_research_enabled: true
source_verification: true
---

# Research Report: technical

**Date:** 2026-04-06
**Author:** Thibaultllopis
**Research Type:** technical

---

## Research Overview

Analyse de faisabilité technique exhaustive couvrant les 6 défis critiques pour implémenter une Reversible Virtual Machine en Rust : types linéaires (`QuantumCell`), portes SIMD (Toffoli/CNOT/X), macro procédurale DSL (`aion_block!`), Garbage-Free Collector (uncomputation), algorithme de Bennett (compilation réversible), et intégrité mémoire (`Pin`/Arena). Toutes les solutions identifiées sont vérifiées contre l'état de l'art Rust 2025-2026. **Verdict : 6/6 défis faisables, aucun showstopper.**

---

<!-- Content will be appended sequentially through research workflow steps -->

## Confirmation du Périmètre Technique

**Sujet** : Faisabilité RVM Aion-OS en Rust
**Objectifs** : Valider la faisabilité des 6 défis techniques identifiés
**Périmètre confirmé** : 2026-04-06

---

## Analyse de Faisabilité Technique — Les 6 Défis

### Défi 1 : `QuantumCell<T>` — Forcer la Linéarité en Rust

#### Le Problème

Rust implémente des **types affines** (utilisés *au plus* une fois), pas des **types linéaires** (utilisés *exactement* une fois). Or `QuantumCell` exige la linéarité : chaque cellule **doit** être consommée (pas copiée, pas détruite).

#### Solutions Identifiées

**Approche A : `UseOnce<T>` avec `Drop` qui panique**

La technique documentée par Geo-Ant consiste à créer un wrapper qui :
1. Interdit `Clone` et `Copy` (Rust le fait nativement)
2. Implémente `Drop` avec `panic!("QuantumCell dropped without being consumed")`
3. La méthode `consume(self) -> T` utilise `ManuallyDrop` pour éviter le `Drop` implicite

```rust
pub struct QuantumCell<T> {
    value: ManuallyDrop<T>,
    consumed: bool,
}

impl<T> Drop for QuantumCell<T> {
    fn drop(&mut self) {
        if !self.consumed {
            panic!("QuantumCell dropped without being consumed — information lost!");
        }
    }
}
```

**Limitation critique** : `mem::forget` peut contourner le `Drop` (c'est safe en Rust). Mitigation : la macro `aion_block!` interdit `mem::forget` dans son scope.

**Approche B : Erreur au link-time (prevent_drop pattern)**

Plus agressive : le `Drop` appelle une fonction externe non existante, déplaçant l'erreur au link-time. Avantage : impossible à contourner. Inconvénient : messages d'erreur cryptiques.

**Approche C : Future — Trait `Leave` (proposition Niko Matsakis)**

Niko Matsakis (babysteps) a proposé des "must-move types" qui nécessitent un nouveau trait `Leave` dont `Drop` hériterait. Les types sans `Leave` ne pourraient pas être droppés. **Statut : proposition, pas implémenté en Rust stable.**

**Approche D : Crate `linear_type`**

La crate `linear_type` sur crates.io fournit une implémentation existante de types linéaires en Rust.

#### Verdict de Faisabilité

| Approche | Faisabilité | Sécurité | Ergonomie | Recommandation |
|----------|------------|----------|-----------|---------------|
| A: Drop + panic | ✅ Immédiate | 8/10 (mem::forget) | 7/10 | **Phase 1** — suffisant pour prototype |
| B: Link-time error | ✅ Immédiate | 9/10 | 4/10 | Phase 2 si A insuffisant |
| C: Leave trait | ❌ Pas disponible | 10/10 | 10/10 | Futur — quand Rust l'implémente |
| D: Crate linear_type | ✅ Immédiate | 7/10 | 6/10 | Évaluer en Phase 1 |

**Faisabilité : ✅ CONFIRMÉE** — L'approche A (Drop + panic) couplée à l'interdiction de `mem::forget` dans `aion_block!` est suffisante pour un prototype fonctionnel.

_Sources : [Geo-Ant - Cursed Linear Types](https://geo-ant.github.io/blog/2024/rust-linear-types-use-once/), [Faultlore - The Pain of Linear Types](https://faultlore.com/blah/linear-rust/), [Niko Matsakis - Must Move Types](https://smallcultfollowing.com/babysteps/blog/2023/03/16/must-move-types/), [Crate linear_type](https://docs.rs/linear_type), [Rust Internals - must_use and linear types](https://internals.rust-lang.org/t/must-use-and-linear-types/12736)_

---

### Défi 2 : Portes de Toffoli SIMD-Optimisées

#### Le Problème

Les portes Toffoli (CCNOT), CNOT et Pauli-X opèrent sur des bits individuels. Pour une RVM performante, il faut paralléliser ces opérations sur des vecteurs de bits via SIMD.

#### Implémentation des Portes en Rust Pur

**Pauli-X (NOT réversible)** : `out = !in` → trivial, un XOR avec masque de uns
**CNOT** : `(a, b) → (a, a ^ b)` → un XOR
**Toffoli** : `(a, b, c) → (a, b, c ^ (a & b))` → un AND + un XOR

Ces 3 opérations sont **purement bitwise** (XOR, AND, NOT) → naturellement SIMD-parallélisables.

#### Approches SIMD en Rust

**Option 1 : `std::simd` (nightly)**

```rust
use std::simd::u64x4; // 256 bits = 4 × 64 bits

fn toffoli_simd(a: u64x4, b: u64x4, c: u64x4) -> (u64x4, u64x4, u64x4) {
    (a, b, c ^ (a & b))
}
```

Portable, compile vers les meilleures instructions SIMD sur chaque cible (SSE, AVX2, AVX-512, NEON).

**Option 2 : `core::arch` (stable)**

Accès direct aux intrinsics SSE/AVX via `_mm256_xor_si256`, `_mm256_and_si256`. Plus verbeux mais stable.

**Option 3 : Crate `pulp` ou `wide` (stable)**

Abstraction SIMD stable avec multiversioning automatique.

#### Performance Attendue

- Avec AVX-512 : **512 portes Toffoli par instruction** (512 bits en parallèle)
- Avec AVX2 : **256 portes par instruction**
- Avec SSE2 : **128 portes par instruction**
- Sans SIMD : 64 portes par instruction (un u64 natif)

**Attention** : Le gain réel dépend du pattern d'accès mémoire et du throughput du cache. Les opérations bitwise sont compute-bound, donc le SIMD offre un gain quasi-linéaire.

#### Verdict de Faisabilité

| Approche | Faisabilité | Performance | Portabilité | Recommandation |
|----------|------------|-------------|-------------|---------------|
| `std::simd` (nightly) | ✅ | Optimale | Maximale | **Phase 2** — portable |
| `core::arch` (stable) | ✅ | Optimale | Par arch | Phase 2 alternative |
| `pulp` / `wide` (stable) | ✅ | Bonne | Bonne | Phase 1 si stable requis |
| Scalaire (u64 natif) | ✅ | Baseline | Maximale | **Phase 1** — prototype |

**Faisabilité : ✅ CONFIRMÉE** — Les portes Toffoli sont des opérations bitwise pures, parfaitement adaptées au SIMD. L'implémentation est triviale.

_Sources : [Monadera - Faster Rust with SIMD](https://monadera.com/blog/faster-rust-with-simd/), [Nine Rules for SIMD in Rust](https://towardsdatascience.com/nine-rules-for-simd-acceleration-of-your-rust-code-part-1-c16fe639ce21/), [Rust std::simd docs](https://doc.rust-lang.org/std/simd/index.html)_

---

### Défi 3 : Macro Procédurale `aion_block!`

#### Le Problème

La macro doit :
1. Parser du code Rust séquentiel classique
2. Rejeter au compile-time toute opération irréversible (ex: `x = y` destructif)
3. Générer automatiquement le chemin d'inversion (backtracking)
4. Produire des erreurs de compilation claires

#### Architecture de la Macro

**Crates nécessaires** : `proc-macro2`, `syn` (parsing AST), `quote` (génération de code)

**Pipeline** :
```
Code utilisateur → syn::parse → AST → Validation réversibilité → 
    → Génération code forward → Génération code backward → quote! → TokenStream
```

**Opérations autorisées** (réversibles) :
- `x += expr` → inverse : `x -= expr`
- `x ^= expr` → inverse : `x ^= expr` (auto-inverse)
- `swap(x, y)` → inverse : `swap(x, y)` (auto-inverse)
- `if cond { ... } fi assert` → conditionnels réversibles (pattern Janus)
- Appels de fonctions `ReversibleOp`

**Opérations rejetées** (irréversibles) :
- `x = expr` (assignation destructive — écrase l'ancienne valeur)
- `mem::forget`, `mem::drop` (destruction d'information)
- `println!`, I/O (effets de bord non réversibles)

#### Mécanismes de Rejet Compile-Time

**`compile_error!`** : Émission d'erreurs de compilation personnalisées directement dans le flux de tokens
**`syn::Error`** : Erreurs pointant précisément vers le token problématique dans le code source de l'utilisateur
**`proc-macro-error` crate** : Permet d'accumuler plusieurs erreurs avant d'abandonner (emit_error!)

**Exemple de rejet** :
```rust
aion_block! {
    let mut x = QuantumCell::new(5);
    x = QuantumCell::new(10); // ❌ COMPILE ERROR: "Destructive assignment 
                                //    in aion_block! — use x += or x ^= instead"
}
```

#### Génération du Chemin Inverse

Pour chaque bloc `aion_block!`, la macro génère une structure `AionBlock` avec :
- `fn forward(&mut self)` → exécution normale
- `fn backward(&mut self)` → exécution inverse (instructions en ordre inverse, opérations inversées)

#### Verdict de Faisabilité

| Aspect | Faisabilité | Complexité | Recommandation |
|--------|------------|-----------|---------------|
| Parsing AST avec syn | ✅ | Moyenne | Mature, bien documenté |
| Rejet compile-time | ✅ | Moyenne | `compile_error!` + `syn::Error` |
| Génération code inverse | ✅ | Élevée | Commencer par sous-ensemble (+=, ^=, swap) |
| Couverture complète | ⚠️ | Très élevée | Approche incrémentale v0.3→v1.0 |
| Messages d'erreur clairs | ✅ | Moyenne | `proc-macro-error` + `trybuild` pour tests |

**Faisabilité : ✅ CONFIRMÉE** pour un sous-ensemble d'opérations. La couverture complète (boucles, conditionnels réversibles, appels de fonctions) est un travail de long terme.

_Sources : [Rust compile_error!](https://doc.rust-lang.org/std/macro.compile_error.html), [Turbo.fish - Proc Macro Error Handling](https://blog.turbo.fish/proc-macro-error-handling/), [syn crate](https://docs.rs/syn), [proc-macro-error crate](https://docs.rs/proc-macro-error)_

---

### Défi 4 : Garbage-Free Collector par Uncomputation

#### Le Problème

Le calcul réversible génère des ancilla bits temporaires. Au lieu de les "libérer" (ce qui détruirait de l'information), il faut les "dé-calculer" (uncompute) pour les ramener à leur état initial |0⟩.

#### Architecture : Pile Miroir d'Ancilla

```
Forward execution:        Backward (uncomputation):
┌──────────────┐          ┌──────────────┐
│ Step 1: a→b  │ push(a)  │ Step 3: c→b  │ pop(b)→verify
│ Step 2: b→c  │ push(b)  │ Step 2: b→a  │ pop(a)→verify
│ Step 3: c→d  │ push(c)  │ Step 1: reset │ 
└──────────────┘          └──────────────┘
   Ancilla Stack:            Ancilla Stack:
   [a, b, c]                 [] (vide = garbage-free)
```

#### Implémentation en Rust

```rust
pub struct AncillaStack<T> {
    stack: Vec<T>,          // Pile LIFO des états intermédiaires
    capacity: usize,        // Budget mémoire configurable
}

pub struct GarbageFreeCollector {
    ancilla_stacks: HashMap<RegisterId, AncillaStack<BitVector>>,
}

impl GarbageFreeCollector {
    /// Enregistre un état avant modification (forward)
    fn checkpoint(&mut self, reg: RegisterId, state: BitVector) { ... }
    
    /// Vérifie et restaure un état (backward/uncompute)
    fn uncompute(&mut self, reg: RegisterId, expected: BitVector) -> Result<(), EntropyError> { ... }
    
    /// Vérifie que toutes les ancilla sont à |0⟩
    fn verify_garbage_free(&self) -> bool { ... }
}
```

#### Stratégies d'Optimisation

- **Uncomputation sélective** (SQUARE) : Ne dé-calculer que ce qui est nécessaire, pas tout
- **Réutilisation d'ancilla** : Les bits à |0⟩ après uncomputation sont réutilisables
- **Budget mémoire** : Limiter la taille de la pile miroir, forcer l'uncomputation quand le budget est atteint

#### Verdict de Faisabilité

| Aspect | Faisabilité | Complexité |
|--------|------------|-----------|
| Pile miroir basique (Vec) | ✅ Triviale | Basse |
| Checkpointing d'états | ✅ | Basse |
| Vérification garbage-free | ✅ | Basse |
| Uncomputation automatique | ✅ | Moyenne |
| Optimisation SQUARE | ✅ | Élevée |
| Budget mémoire configurable | ✅ | Basse |

**Faisabilité : ✅ CONFIRMÉE** — L'implémentation basique est triviale en Rust (Vec comme pile LIFO). L'optimisation est un travail d'itération.

---

### Défi 5 : Algorithme de Bennett — Compilation Automatique

#### Le Problème

L'algorithme de Bennett transforme un calcul irréversible de temps T et espace S en calcul réversible avec :
- **Temps** : O(T^(1+ε))
- **Espace** : O(S · log(T))

Le facteur caché ε·2^(1/ε) rend le choix de ε critique.

#### Architecture d'Implémentation

L'algorithme de Bennett se modélise comme un **pebbling game** sur un graphe de calcul :

1. **Forward** : Exécuter le calcul, sauvegarder les checkpoints
2. **Copy** : Copier le résultat final
3. **Backward** : Uncompute en utilisant les checkpoints pour restaurer les états intermédiaires

**En Rust** : Cela se traduit par un interpréteur de graphe de calcul avec :
- Un `ComputationGraph` (DAG) des opérations
- Un `PebblingStrategy` qui détermine quand checkpointer
- Un `ReversibleExecutor` qui exécute forward/backward

#### Complexité d'Implémentation

| Composant | Complexité | Dépendances |
|-----------|-----------|-------------|
| ComputationGraph (DAG) | Moyenne | `petgraph` crate |
| PebblingStrategy (choix de ε) | Élevée | Algorithme récursif de Bennett |
| ReversibleExecutor | Élevée | Défi 4 (GarbageFreeCollector) |
| Optimisation Levine-Sherman | Très élevée | Recherche mathématique |

#### Verdict de Faisabilité

**Faisabilité : ✅ CONFIRMÉE** mais c'est le défi le plus complexe. L'implémentation naïve (ε fixe, pas d'optimisation) est faisable en Phase 4. L'optimisation (choix adaptatif de ε, bornes Levine-Sherman) est un travail de recherche.

---

### Défi 6 : `Pin<Box<T>>` pour l'Intégrité Topologique

#### Le Problème

Les données dans la RVM ne doivent pas être déplacées en mémoire (relocation) car cela pourrait corrompre les références internes du graphe de calcul réversible.

#### Analyse de `Pin<Box<T>>`

**Ce que Pin garantit** :
- L'adresse mémoire du `T` sous-jacent **ne changera pas** tant que le `Pin` existe
- Impossible d'obtenir un `&mut T` qui permettrait `mem::replace` ou `mem::swap`
- Sécurisé contre les moves accidentels

**Ce que Pin ne garantit PAS** :
- Pin ne concerne que les types `!Unpin` (qui ne se déclarent pas déplaçables)
- Les types qui implémentent `Unpin` (la plupart des types Rust) peuvent être extraits de `Pin`

**Pour `QuantumCell`** :
```rust
impl<T> !Unpin for QuantumCell<T> {} // Force Pin à être effectif

// Allocation pinned sur le heap
let cell: Pin<Box<QuantumCell<u64>>> = Box::pin(QuantumCell::new(42));
```

#### Pertinence Réelle pour Aion-OS

**Question** : `Pin<Box<T>>` est-il réellement nécessaire pour Aion-OS ?

- **Oui si** : La RVM utilise des structures auto-référentielles (graphe de calcul avec pointeurs internes)
- **Non si** : La RVM utilise des indices (comme les indexes dans un Vec/Arena) au lieu de pointeurs

**Recommandation** : Utiliser un **arena allocator** (`typed-arena`, `bumpalo`) avec des indices plutôt que `Pin<Box<T>>`. Plus simple, plus performant, et suffisant pour garantir la stabilité des adresses.

#### Verdict de Faisabilité

| Approche | Faisabilité | Performance | Complexité | Recommandation |
|----------|------------|-------------|-----------|---------------|
| `Pin<Box<T>>` | ✅ | Allocation heap par cellule | Moyenne | Pour structures auto-référentielles |
| Arena + indices | ✅ | Allocation bulk, cache-friendly | Basse | **Recommandé** pour la RVM |
| `Vec` + indices stables | ✅ | Simple | Basse | Phase 1 prototype |

**Faisabilité : ✅ CONFIRMÉE** — `Pin<Box<T>>` fonctionne mais n'est probablement pas nécessaire. Un arena allocator avec indices est plus adapté.

_Sources : [Rust std::pin docs](https://doc.rust-lang.org/std/pin/index.html), [Cloudflare - Pin and Unpin](https://blog.cloudflare.com/pin-and-unpin-in-rust/), [Sling Academy - Self-Referential with Pin](https://www.slingacademy.com/article/creating-self-referential-structures-in-rust-with-box-and-pin/)_

---

## Architecture RVM Recommandée

### Stack Technologique

| Couche | Technologie | Crate | Stabilité |
|--------|------------|-------|-----------|
| Types linéaires | `QuantumCell<T>` custom | — (code propre) | Prototype |
| Portes logiques | Rust natif + SIMD | `std::simd` (nightly) ou `pulp` (stable) | Mature |
| DSL | `aion_block!` proc macro | `syn`, `quote`, `proc-macro2` | Mature |
| Garbage-Free GC | Pile miroir `Vec<BitVector>` | — (code propre) | Prototype |
| Algorithme Bennett | Graphe de calcul + pebbling | `petgraph` | Mature |
| Allocation mémoire | Arena allocator | `typed-arena` ou `bumpalo` | Mature |
| VM Execution | Stack-based opcode dispatch | — (code propre) | Prototype |
| Tests | Property-based testing | `proptest` | Mature |
| Benchmarks | Critère de performance | `criterion` | Mature |

### Architecture VM

```
                    ┌─────────────────────────────┐
                    │      aion_block! (DSL)       │
                    │   syn → validate → quote     │
                    └──────────────┬──────────────┘
                                   │ compile-time
                    ┌──────────────▼──────────────┐
                    │   Bytecode réversible        │
                    │   (forward_ops, backward_ops)│
                    └──────────────┬──────────────┘
                                   │ runtime
                    ┌──────────────▼──────────────┐
                    │   RVM Execution Engine        │
                    │   ┌───────┐  ┌────────────┐ │
                    │   │ Stack │  │ Registers  │ │
                    │   │(arena)│  │(QuantumCell)│ │
                    │   └───┬───┘  └─────┬──────┘ │
                    │       │            │         │
                    │   ┌───▼────────────▼──────┐ │
                    │   │   Gate Execution Unit  │ │
                    │   │   (Toffoli SIMD)       │ │
                    │   └───────────┬────────────┘ │
                    │               │              │
                    │   ┌───────────▼────────────┐ │
                    │   │ Garbage-Free Collector  │ │
                    │   │ (ancilla mirror stack)  │ │
                    │   └────────────────────────┘ │
                    └──────────────────────────────┘
```

_Sources : [Rust-hosted-langs - VM Design](https://rust-hosted-langs.github.io/book/chapter-interp-vm-design.html), [Toffoli Original Paper](https://cqi.inf.usi.ch/qic/80_Toffoli.pdf)_

---

## Tableau de Faisabilité Récapitulatif

| # | Défi | Faisabilité | Complexité | Phase | Risque Résiduel |
|---|------|------------|-----------|-------|----------------|
| 1 | QuantumCell linéaire | ✅ **OUI** | Moyenne | v0.1 | `mem::forget` contournable (mitigation : macro) |
| 2 | Portes Toffoli SIMD | ✅ **OUI** | Basse | v0.1-v0.2 | Performance dépend du benchmark réel |
| 3 | `aion_block!` macro | ✅ **OUI** (sous-ensemble) | Élevée | v0.3 | Couverture complète = long terme |
| 4 | Garbage-Free Collector | ✅ **OUI** | Moyenne | v0.4-v0.5 | Explosion mémoire si mal calibré |
| 5 | Algorithme de Bennett | ✅ **OUI** (naïf) | Très élevée | v0.4 | Optimisation = recherche ouverte |
| 6 | Pin/Intégrité mémoire | ✅ **OUI** | Basse | v0.1 | Arena > Pin pour ce cas d'usage |

### Verdict Global

**Les 6 défis sont techniquement faisables en Rust.** Aucun n'est un showstopper. La difficulté est graduée : les défis 1, 2, 6 sont immédiats, le défi 3 est incrémental, et les défis 4, 5 sont les plus complexes mais abordables par phases.

---

## Patterns d'Intégration et Interopérabilité

### API et Interfaces d'Aion-OS

Aion-OS doit exposer plusieurs interfaces pour maximiser l'adoption :

| Interface | Cible | Technologie | Priorité |
|-----------|-------|------------|----------|
| **Crate Rust native** | Développeurs Rust | `cargo add aion-os` — API Rust idiomatique | Critique |
| **C FFI** | Intégration systèmes | `#[repr(C)]` + `cbindgen` pour headers auto | Moyenne |
| **WebAssembly** | Navigateur / edge | `wasm-bindgen` + `wasm-pack` | Moyenne |
| **CLI** | Utilisation standalone | `clap` pour arguments | Haute |
| **Python bindings** | Communauté ML/IA | `PyO3` / `maturin` | Basse (futur) |

**Pattern recommandé — Architecture en couches** :
```
┌─────────────────────────────────┐
│  Python (PyO3)  │  WASM  │  C  │  ← Bindings
├─────────────────────────────────┤
│     Public Rust API (safe)      │  ← Interface stable
├─────────────────────────────────┤
│     Core Engine (unsafe min)    │  ← Implémentation
└─────────────────────────────────┘
```

Le core engine est une crate Rust pure. Les bindings (C, WASM, Python) sont des crates séparées qui wrappent l'API publique. Cela permet une évolution indépendante de chaque couche.

_Sources : [The Big Book of Rust Interop - Layered Design](https://nrc.github.io/big-book-ffi/patterns/layered.html), [Rust FFI - Nomicon](https://doc.rust-lang.org/nomicon/ffi.html), [Rust-Wasm Book](https://rustwasm.github.io/book/reference/js-ffi.html)_

### Intégration avec l'Écosystème Existant

| Écosystème | Intégration | Comment |
|------------|------------|---------|
| **Qiskit/Cirq** | Export/Import circuits | Format OpenQASM ou JSON pour échanger des circuits réversibles |
| **Janus** | Compatibilité programmes | Parser de syntaxe Janus → bytecode Aion-OS |
| **Vaire chips (futur)** | Backend d'exécution | Trait `ExecutionBackend` abstrait le hardware |
| **RISC-V** | Compilation native | Extension ISA réversible (futur) |

---

## Patterns Architecturaux pour la RVM

### Design Pattern Principal : Trait-Based Dispatch

L'architecture de la RVM suit le pattern **trait-based dispatch** idiomatique en Rust :

```rust
/// Trait central — toute opération de la RVM doit être réversible
pub trait ReversibleOp {
    type Input;
    type Output;
    type Ancilla; // Preuve d'inversion (bits auxiliaires)
    
    fn execute(&self, input: Self::Input) -> (Self::Output, Self::Ancilla);
    fn undo(&self, output: Self::Output, ancilla: Self::Ancilla) -> Self::Input;
}
```

**Propriété fondamentale à vérifier** : `∀ x: undo(execute(x)) == x`

### Dispatch d'Opcodes

L'exécution de la RVM utilise un **match dispatch** dans une boucle :

```rust
loop {
    match program[pc] {
        Op::PauliX(reg)        => registers[reg] ^= mask,
        Op::CNOT(ctrl, target) => registers[target] ^= registers[ctrl],
        Op::Toffoli(c1, c2, t) => registers[t] ^= registers[c1] & registers[c2],
        Op::Checkpoint(reg)    => gc.checkpoint(reg, registers[reg].clone()),
        Op::Uncompute(reg)     => gc.uncompute(reg)?,
        Op::Halt               => break,
    }
    pc += 1;
}
```

Sur les CPU modernes (Haswell+), le prédicteur de branchement réduit l'avantage du computed goto, rendant le `match` Rust quasi-optimal.

_Sources : [VM Dispatch Experiments in Rust](https://pliniker.github.io/post/dispatchers/), [Writing Interpreters in Rust](https://rust-hosted-langs.github.io/book/chapter-interp-vm-design.html), [Rust Design Patterns - Interpreter](https://rust-unofficial.github.io/patterns/patterns/behavioural/interpreter.html)_

---

## Stratégie de Test et Vérification

### Property-Based Testing avec Proptest

Le calcul réversible se prête parfaitement au property-based testing :

**Propriété #1 — Réversibilité** : `∀ gate, ∀ input: gate.undo(gate.execute(input)) == input`
**Propriété #2 — Garbage-free** : `∀ program: gc.verify_garbage_free() == true après exécution complète`
**Propriété #3 — Déterminisme** : `∀ program, ∀ input: execute(input) est toujours identique`

```rust
proptest! {
    #[test]
    fn toffoli_is_reversible(a in any::<u64>(), b in any::<u64>(), c in any::<u64>()) {
        let (ra, rb, rc) = toffoli_execute(a, b, c);
        let (ua, ub, uc) = toffoli_undo(ra, rb, rc);
        prop_assert_eq!((a, b, c), (ua, ub, uc));
    }
}
```

**Proptest** est recommandé sur QuickCheck car il permet de définir des stratégies de génération par valeur (pas par type), plus flexibles pour les vecteurs de bits de taille variable.

### Stratégie de Benchmark avec Criterion

| Benchmark | Métrique | Cible |
|-----------|----------|-------|
| Toffoli gate throughput | ops/sec | >1B gates/sec (scalaire), >100B (SIMD AVX-512) |
| `aion_block!` compilation | temps compile | <5% overhead vs code natif |
| Uncomputation overhead | mémoire + temps | <2× vs exécution forward seule |
| Bennett compilation | ratio temps réversible/irréversible | <10× pour ε=0.5 |

_Sources : [Proptest crate](https://github.com/proptest-rs/proptest), [Criterion.rs](https://github.com/bheisler/criterion.rs), [Property-Based Testing in Rust](https://www.lpalmieri.com/posts/an-introduction-to-property-based-testing-in-rust/)_

---

## Synthèse Technique — Résumé Exécutif

### Verdict de Faisabilité Globale : ✅ CONFIRMÉ

**Aion-OS est techniquement réalisable en Rust.** Les 6 défis identifiés ont chacun au moins une solution viable, avec des crates matures pour les composants clés.

### Découvertes Techniques Clés

| # | Découverte | Impact |
|---|-----------|--------|
| 1 | Le pattern `UseOnce<T>` + `Drop` + `ManuallyDrop` résout la linéarité à 80% | `QuantumCell` est implémentable dès la v0.1 |
| 2 | Les portes Toffoli = XOR + AND = SIMD trivial | Performance potentiellement 8-512× vs scalaire |
| 3 | `syn` + `quote` + `compile_error!` forment un toolkit DSL mature | `aion_block!` faisable par sous-ensemble incrémental |
| 4 | `Vec` comme pile LIFO suffit pour le Garbage-Free Collector | Implémentation basique en <100 lignes |
| 5 | Bennett = pebbling game sur DAG → `petgraph` | Le composant le plus complexe, mais bibliothèques matures |
| 6 | Arena allocator > `Pin<Box<T>>` pour la RVM | Simplification architecturale majeure |
| 7 | `match` dispatch quasi-optimal sur CPU modernes | Pas besoin de computed goto |
| 8 | `proptest` vérifie la réversibilité automatiquement | `∀ x: undo(execute(x)) == x` testable par fuzzing |

### Plan d'Implémentation Technique

| Phase | Livrables | Crates | Estimation |
|-------|----------|--------|-----------|
| **v0.1** | `QuantumCell`, Toffoli/CNOT/X scalaire, tests proptest | `proptest` | 2-4 semaines |
| **v0.2** | SIMD portes, benchmarks criterion | `std::simd`/`pulp`, `criterion` | 2-3 semaines |
| **v0.3** | `aion_block!` (+=, ^=, swap) | `syn`, `quote`, `proc-macro2` | 4-6 semaines |
| **v0.4** | Garbage-Free Collector + Bennett naïf | `petgraph` | 4-8 semaines |
| **v0.5** | Optimisation Bennett + budget mémoire | — | 4-8 semaines |
| **v1.0** | RVM complète + FFI C + CLI | `clap`, `cbindgen` | 8-12 semaines |

### Risques Techniques Résiduels

| Risque | Sévérité | Probabilité | Mitigation |
|--------|----------|-------------|-----------|
| `mem::forget` contourne la linéarité | Moyenne | Basse | `aion_block!` l'interdit, code review |
| Overhead SIMD pas mesurable (memory-bound) | Basse | Moyenne | Benchmark dès v0.2, accepter si marginal |
| `aion_block!` trop restrictif pour être utile | Moyenne | Moyenne | Commencer large, restreindre progressivement |
| Bennett explosion mémoire | Élevée | Moyenne | Budget configurable + SQUARE dès v0.5 |
| Complexité dissuade les contributeurs | Moyenne | Moyenne | Documentation exemplaire + onboarding guide |

---

**Date de complétion** : 2026-04-06
**Confiance globale** : Élevée — 6/6 défis faisables, stack technologique mature, architecture claire

_Ce document valide la faisabilité technique complète d'Aion-OS en Rust et fournit un plan d'implémentation phased avec estimation de complexité._
