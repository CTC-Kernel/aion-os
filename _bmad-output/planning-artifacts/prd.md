---
stepsCompleted: ['step-01-init', 'step-02-discovery', 'step-02b-vision', 'step-02c-executive-summary', 'step-03-success', 'step-04-journeys', 'step-05-domain', 'step-06-innovation', 'step-07-project-type', 'step-08-scoping', 'step-09-functional', 'step-10-nonfunctional', 'step-11-polish', 'step-12-complete']
classification:
  projectType: 'developer_tool'
  domain: 'scientific'
  complexity: 'high'
  projectContext: 'greenfield'
inputDocuments:
  - 'product-brief-aion-os.md'
  - 'product-brief-aion-os-distillate.md'
  - 'research/domain-calcul-reversible-research-2026-04-06.md'
  - 'research/market-aion-os-reversible-computing-research-2026-04-06.md'
  - 'research/technical-aion-os-rvm-rust-feasibility-research-2026-04-06.md'
  - 'brainstorming/brainstorming-session-2026-04-06-aion-os.md'
documentCounts:
  briefs: 2
  research: 3
  brainstorming: 1
  projectDocs: 0
workflowType: 'prd'
---

# Product Requirements Document - Rewind (Aion-OS)

**Author:** Thibaultllopis
**Date:** 2026-04-06

---

## Executive Summary

**Rewind** est un SDK open-source en Rust qui implémente le premier runtime de calcul nativement réversible. Chaque opération exécutée dans Rewind possède un inverse structurel vérifié au compile-time — permettant l'exécution bidirectionnelle (forward/backward), le rollback sans coût, et le debugging temporel natif sans enregistrement de traces. Le produit cible en priorité les 2,27 millions de développeurs Rust, pour qui le debugging est un pain point en déclin mesurable (enquête Rust 2025), avant d'étendre vers les communautés calcul réversible, sécurité/DevSecOps, et ML.

Le problème de fond : l'informatique conventionnelle détruit de l'information à chaque opération. Chaque `x = y` écrase un état, chaque garbage collection efface des intermédiaires, chaque bit effacé dissipe un minimum de kT·ln(2) de chaleur (principe de Landauer, vérifié expérimentalement). Les debuggers time-travel existants (rr, UndoDB) contournent le problème par du record-replay coûteux (1.2-5× slowdown, overhead mémoire) sans rien prouver sur la correctness du code. Rewind résout le problème à la racine : le calcul est structurellement réversible, vérifié par le compilateur, exécutable dans les deux sens sans overhead d'enregistrement.

Le timing est dicté par la convergence de quatre forces : la crise énergétique de l'IA (1 000 TWh projetés d'ici 2030), l'arrivée du premier chip réversible commercial (Vaire Computing, 2025), les réglementations européennes d'efficacité énergétique (EU EED Data Centre Package Q1 2026), et la maturité de l'écosystème Rust. Aucun SDK open-source de calcul nativement réversible n'existe — Rewind occupe ce vide stratégique.

### Ce Qui Rend Rewind Spécial

L'avantage structurel de Rewind repose sur une coïncidence fondamentale : le système de types affines de Rust (ownership, move semantics) est une implémentation native de la logique linéaire de Girard — exactement la logique qui fonde le calcul réversible. Aucun autre langage mainstream n'offre cet alignement entre son système de types et les exigences de la réversibilité computationnelle. Cet avantage est architectural et non-réplicable sans refonte du langage hôte.

L'API "Sans Théorie" (`forward()`, `backward()`, `checkpoint()`, `restore()`, `#[reversible]`) cache délibérément la complexité (portes de Toffoli, algorithme de Bennett, ancilla bits) — un développeur utilise Rewind sans jamais apprendre la thermodynamique, comme Docker fonctionne sans comprendre les cgroups.

L'architecture modulaire (crates composables : `rewind-gates`, `rewind-cell`, `rewind-gc`, `rewind-dsl`) et le trait `ExecutionBackend` découplent le logiciel du hardware, permettant l'exécution sur CPU conventionnel aujourd'hui et sur chips réversibles (Vaire) demain.

## Classification Projet

| Dimension | Valeur |
|-----------|--------|
| **Type** | Developer tool — SDK/bibliothèque Rust avec macro procédurale et DSL |
| **Domaine** | Scientifique/Computationnel — calcul réversible, thermodynamique de l'information |
| **Complexité** | Élevée — technologie sans précédent, fondements théoriques profonds, multi-segment |
| **Contexte** | Greenfield — nouveau produit, aucun code existant |
| **Licence** | Apache 2.0 / MIT (double licence, standard écosystème Rust) |

---

## Critères de Succès

### Succès Utilisateur

**Le moment "aha"** : Le développeur tape `step backward` pour la première fois et voit l'état remonter dans le temps — pas un replay de trace, un vrai retour d'état computationnel.

| Critère | Métrique | Cible |
|---------|----------|-------|
| Temps découverte → premier usage | `cargo add rewind` → premier `#[reversible]` | < 5 minutes |
| Réversibilité vérifiable | `∀x: undo(execute(x)) == x` par proptest | 100% des portes |
| Compilation réversibilité | `#[reversible]` rejette les ops irréversibles | 0 faux négatifs v0.1 |
| Debugging temporel | Step-backward sur programme non-trivial | Demo publique Q3 2026 |
| Documentation | README + quickstart + 3 exemples | Disponible au lancement |

### Succès Business / Communautaire

| Critère | Métrique | Cible 6 mois | Cible 12 mois |
|---------|----------|-------------|--------------|
| Visibilité | Front page Hacker News | 1 post, 100+ comments | 3+ articles externes |
| Adoption | Installs `cargo add rewind` | 100/mois | 500/mois |
| Communauté | Contributeurs actifs (PRs mergées) | 5 | 20 |
| Écosystème | Crates dépendantes | 2 | 10 |
| Académique | Paper soumis (RC 2026 / NeurIPS) | 1 | 3 citations |
| Partenariats | Vaire Computing | Email envoyé | Collaboration formelle |
| Financement | E-CoRe / Rust Foundation | Soumise | Grant obtenu |

### Succès Technique

| Critère | Métrique | Cible |
|---------|----------|-------|
| Correctness | `undo(execute(x)) == x` | 100% (proptest 100K+ inputs) |
| Perf portes scalaire | Toffoli throughput | > 1B portes/sec |
| Perf portes SIMD | Toffoli AVX2/512 | > 100B portes/sec (v0.2) |
| Overhead compile | `#[reversible]` vs natif | < 5% additionnel |
| Garbage-free | Ancilla résiduels | 0 (assertion) |
| Abstraction hardware | `ExecutionBackend` | Implémenté + `SimulatedCPU` |

### Résultats Mesurables Clés

1. **Validation technique (Q3 2026)** : Prototype v0.1 + demo step-backward publique
2. **Validation communautaire (Q4 2026)** : 100+ stars, 5+ contributeurs, front page HN
3. **Validation académique (Q3 2026)** : Paper "Bennett = Gradient Checkpointing" soumis
4. **Validation marché (Q2 2028)** : Premier POC enterprise ou partenariat Vaire

---

## Périmètre Produit

### MVP — v0.1 "Rewind" (Semaines 1-6)

- `QuantumCell<T>` — type linéaire forcé (Drop + panic + ManuallyDrop)
- Portes réversibles scalaires : Toffoli, CNOT, Pauli-X
- `#[reversible]` attribut — sous-ensemble (+=, ^=, swap), rejet compile-time
- API publique : `forward()`, `backward()`, `checkpoint()`, `restore()`
- Trait `ExecutionBackend` avec `SimulatedCPU`
- Tests proptest pour toutes les portes
- README + quickstart < 5 min + GIF animé forward/backward
- Double licence Apache 2.0 / MIT, publication crates.io

**Hors scope MVP :** SIMD, Bennett automatique, Garbage-Free optimisé, bindings FFI/WASM/Python

### Growth — v0.2-v0.5 (Mois 2-6)

- v0.2 : SIMD (std::simd/pulp) + benchmarks criterion
- v0.3 : `#[reversible]` étendu (boucles, conditionnels réversibles)
- v0.4 : Bennett naïf + Garbage-Free Collector
- v0.5 : Budget mémoire, SQUARE, optimisations Bennett
- Playground Web WASM, Aion Academy, Spec RBF v0.1

### Vision — v1.0+ (Année 2+)

- RVM complète, intégration Vaire/FPGA, bindings C/Python/WASM
- Framework "Aion ML" (RevNets + gradient checkpointing)
- Reversibility Index, CLI standalone, extension RISC-V "R"

---

## User Journeys

### Journey 1 : Le Développeur Rust Curieux (Segment Primaire)

**Alex**, développeur Rust senior, lit un post Hacker News titré "Rewind: the debugger that runs your code backward". Intrigué, il clique sur le repo GitHub. Le GIF animé dans le README montre un programme qui s'exécute forward, puis backward, avec la pile d'ancilla qui se vide à zéro. Alex pense "pas possible".

Il tape `cargo add rewind` et copie le premier exemple du quickstart. 3 minutes plus tard, il a un programme `#[reversible]` qui additionne deux nombres — et peut remonter l'opération. Il essaie `step backward` sur un calcul plus complexe. L'état du programme remonte instruction par instruction. Alex réalise : ce n'est pas du record-replay comme rr, c'est le calcul LUI-MÊME qui est inversible. Le moment "aha".

Alex partage le repo à son équipe. Il commence à annoter une de ses fonctions critiques avec `#[reversible]`. Le compilateur rejette une assignation destructive — il comprend pourquoi et utilise `^=` à la place. En une semaine, il a un module entier réversible et peut debugger des cas impossibles à reproduire auparavant.

### Journey 2 : La Chercheuse RC (Segment d'Amorçage)

**Dr. Leila**, chercheuse en calcul réversible à l'INRIA, prépare son papier pour RC 2026. Elle utilise Janus depuis 10 ans mais est frustrée par l'absence de SIMD, de types modernes, et d'IDE support. Elle découvre Rewind via la mailing list de la conférence RC.

Elle clone le repo, exécute les exemples de portes Toffoli. Elle vérifie la propriété `∀x: undo(execute(x)) == x` avec proptest — exactement ce qu'elle faisait manuellement. Elle implémente un algorithme de son paper avec `#[reversible]` et obtient un benchmark. Elle cite Rewind dans son papier et propose un PR pour ajouter une porte de Fredkin.

### Journey 3 : L'Ingénieur Sécurité (Segment Secondaire)

**Marco**, ingénieur DevSecOps, enquête sur un incident de production. Le crash s'est produit après des milliers d'opérations, impossible à reproduire. Il intègre Rewind dans le module suspect avec `#[reversible]` + `checkpoint()`. Lors de la prochaine occurrence, il utilise `restore()` puis `backward()` pour remonter exactement à l'état qui a causé le crash — sans logs volumineux, sans modifier le timing.

### Résumé des Capacités Révélées par les Journeys

| Capacité | Journey 1 | Journey 2 | Journey 3 |
|----------|-----------|-----------|-----------|
| `cargo add` + quickstart < 5 min | Principal | Important | Secondaire |
| `#[reversible]` compile-time check | Principal | Principal | Important |
| `forward()` / `backward()` | Principal | Principal | Principal |
| `checkpoint()` / `restore()` | Secondaire | Important | Principal |
| Proptest réversibilité | Secondaire | Principal | Secondaire |
| SIMD performance | Secondaire | Principal | Non requis |
| Portes custom (Fredkin, etc.) | Non requis | Principal | Non requis |

---

## Exigences Spécifiques au Domaine

### Conformité & Réglementaire

- **Propriété intellectuelle** : Toutes les fondations théoriques (Toffoli 1980, Bennett 1973, Landauer 1961, Girard 1987) sont dans le domaine public. Aucun risque brevet identifié pour l'implémentation logicielle.
- **Licence** : Apache 2.0 / MIT obligatoire pour compatibilité écosystème Rust. Vérifier absence de conflit avec les crates dépendantes (proptest, petgraph, syn, quote).
- **Standards énergétiques** : Documenter l'alignement avec ISO/IEC 21031:2024 (SCI) pour le positionnement green computing.
- **RGPD** : Le calcul réversible peut servir de preuve de destruction d'information (use case "secure erase audit"). Documenter cette capacité.

### Contraintes Techniques

- **Types affines ≠ linéaires** : Rust ne garantit pas la linéarité stricte. `mem::forget` contourne le `Drop`. Mitigation : interdiction dans le scope `#[reversible]`, documentation explicite de la limitation.
- **Overhead simulation** : Le RC simulé sur CPU conventionnel SERA plus lent que le code natif. Communiquer clairement : le gain est en correctness/debugging, pas en performance (sur CPU conventionnel).
- **Facteur caché Bennett** : Le facteur ε·2^(1/ε) dans la borne spatiale diverge quand ε → 0. Imposer un ε minimum configurable avec des défauts raisonnables.
- **Nightly Rust** : `std::simd` requiert nightly. Alternative stable : crate `pulp`. Supporter les deux via feature flags.

### Exigences d'Intégration

| Système | Type d'Intégration | Phase |
|---------|-------------------|-------|
| **crates.io** | Publication crate | MVP |
| **GitHub Actions** | CI/CD tests proptest | MVP |
| **criterion** | Benchmarks automatisés | v0.2 |
| **Vaire Computing SDK** | Trait `ExecutionBackend` | v1.0+ |
| **WASM** (wasm-bindgen) | Playground navigateur | v0.2-v0.3 |
| **PyO3** | Bindings Python pour ML | v1.0+ |

### Mitigation des Risques Domaine

| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|------------|
| `mem::forget` contourne la linéarité | Basse | Moyenne | Interdit dans `#[reversible]`, lint custom, documentation |
| Overhead Bennett explosion mémoire | Moyenne | Élevée | Budget configurable, SQUARE, ε minimum |
| Confusion avec rr/UndoDB | Élevée | Moyenne | Différenciation explicite dans README et docs |
| Jargon RC repousse les devs | Élevée | Élevée | API "Sans Théorie", métaphores (Git, Docker) |

---

## Innovation & Patterns Nouveaux

### Zones d'Innovation Détectées

1. **Calcul nativement réversible en logiciel** : Aucun SDK n'existe. Premier du genre.
2. **Types affines comme fondation du RC** : Connexion Rust ownership ↔ logique linéaire de Girard jamais exploitée dans un produit.
3. **Bennett = Gradient Checkpointing** : Connexion formelle inédite entre deux algorithmes de 1973 et 2016.
4. **Debugging par réversibilité native** : Pas du record-replay, mais du calcul structurellement inversible — nouveau paradigme de debugging.

### Contexte Compétitif

- **Aucun concurrent direct** : Le segment "SDK calcul réversible en Rust" est vide.
- **Concurrents indirects** : rr (record-replay, C/C++ only), UndoDB (commercial), Janus (académique, années 80), RevKit (C++, circuits).
- **Modèles à suivre** : Wasmtime (runtime spécialisé Rust, CNCF sandbox), LLVM (IR universelle).

### Approche de Validation

| Innovation | Validation | Critère de succès |
|-----------|-----------|------------------|
| Réversibilité native | Proptest : `∀x: undo(execute(x)) == x` | 100% sur 100K+ inputs |
| Types affines pour RC | Prototype `QuantumCell<T>` fonctionnel | Compile-time rejection des ops irréversibles |
| Bennett = Gradient Ckpt | Paper académique + implémentation | Soumission RC 2026 ou NeurIPS |
| Step-backward debugging | Demo publique sur programme non-trivial | Vidéo/GIF partageable |

### Fallbacks si l'Innovation Échoue

| Innovation | Risque d'échec | Fallback |
|-----------|---------------|----------|
| `#[reversible]` trop restrictif | Moyen | Mode `#[reversible(checked)]` — vérification runtime au lieu de compile-time |
| SIMD pas de gain mesurable | Faible | Rester en scalaire, le gain est en correctness pas en performance |
| Bennett trop complexe | Moyen | Reporter à v0.5+, focus sur les portes manuelles en v0.1 |
| Step-backward pas assez utile | Faible | Pivoter vers transactions réversibles / fuzzing réversible |

---

## Developer Tool — Exigences Spécifiques

### Architecture Crates

```
rewind (workspace root)
├── rewind-core        # QuantumCell<T>, ReversibleOp trait, ExecutionBackend trait
├── rewind-gates       # Toffoli, CNOT, Pauli-X (scalaire + SIMD)
├── rewind-gc          # Garbage-Free Collector (pile miroir ancilla)
├── rewind-dsl         # Macro procédurale #[reversible], aion_block!
├── rewind-bennett     # Algorithme de Bennett (compilation réversible)
├── rewind             # Crate façade (re-export tout)
└── rewind-playground  # WASM playground (futur)
```

### API Surface Publique (v0.1)

```rust
// Core types
pub struct QuantumCell<T> { ... }  // Type linéaire
pub trait ReversibleOp { ... }      // Trait pour opérations réversibles
pub trait ExecutionBackend { ... }  // Abstraction hardware

// Portes
pub fn pauli_x(cell: &mut QuantumCell<BitVec>) { ... }
pub fn cnot(control: &QuantumCell<BitVec>, target: &mut QuantumCell<BitVec>) { ... }
pub fn toffoli(c1: &QuantumCell<BitVec>, c2: &QuantumCell<BitVec>, 
               target: &mut QuantumCell<BitVec>) { ... }

// API utilisateur
pub fn forward(block: &ReversibleBlock) -> Result<State, RewindError> { ... }
pub fn backward(block: &ReversibleBlock) -> Result<State, RewindError> { ... }
pub fn checkpoint(state: &State) -> CheckpointId { ... }
pub fn restore(id: CheckpointId) -> Result<State, RewindError> { ... }

// Macro
#[reversible]  // Attribut proc-macro
```

### Considérations d'Implémentation

- **Match dispatch** pour l'exécution VM (quasi-optimal sur CPU modernes, pas besoin de computed goto)
- **Arena allocator** (`typed-arena` / `bumpalo`) pour l'allocation mémoire interne (plus performant que `Pin<Box<T>>`)
- **Feature flags** : `simd` (active std::simd nightly), `stable-simd` (active pulp), `bennett` (active la compilation automatique)
- **Minimum Supported Rust Version (MSRV)** : latest stable pour le core, nightly pour SIMD
- **Documentation** : rustdoc complète, exemples exécutables dans chaque module, book mdBook pour tutoriels

### Publication & Distribution

| Canal | Format | Phase |
|-------|--------|-------|
| **crates.io** | `cargo add rewind` | MVP |
| **GitHub** | Source + CI + releases | MVP |
| **docs.rs** | Rustdoc auto-généré | MVP |
| **Web playground** | WASM dans le navigateur | v0.2-v0.3 |
| **mdBook** | Tutoriel "Rewind by Example" | v0.2 |

---

## Développement Phasé & Stratégie MVP

### Philosophie MVP

**Approche : "Prouver le paradigme, pas livrer un produit"**

La v0.1 n'est pas un produit complet — c'est une **preuve que le calcul nativement réversible fonctionne en Rust**. Le minimum pour convaincre : un programme non-trivial qui s'exécute forward ET backward, vérifié par proptest, avec une macro `#[reversible]` qui rejette le code irréversible au compile-time. Si la demo step-backward fonctionne et que le quickstart prend < 5 min, le MVP est un succès.

**Ressources** : Développeur solo ou petite équipe (1-3). Pas de financement initial — OSS pur. Candidature grants (E-CoRe, Rust Foundation) en parallèle.

### Phase 1 : MVP "Rewind" (Semaines 1-6)

| Composant | Priorité | Effort estimé |
|-----------|----------|--------------|
| `QuantumCell<T>` + `ReversibleOp` trait | P0 | 1 semaine |
| Portes Toffoli/CNOT/X scalaires | P0 | 1 semaine |
| `#[reversible]` macro (sous-ensemble) | P0 | 2 semaines |
| API forward/backward/checkpoint/restore | P0 | 1 semaine |
| Tests proptest + CI GitHub Actions | P0 | 3 jours |
| README + quickstart + GIF | P0 | 2 jours |
| Trait `ExecutionBackend` | P1 | 3 jours |
| Publication crates.io | P0 | 1 jour |

**Journeys supportées** : Journey 1 (dev curieux — quickstart + `#[reversible]` + step backward)

### Phase 2 : Growth (Mois 2-6)

| Composant | Priorité | Phase |
|-----------|----------|-------|
| SIMD portes (std::simd/pulp) | P1 | v0.2 |
| Benchmarks criterion publiés | P1 | v0.2 |
| `#[reversible]` étendu (boucles, conditionnels) | P1 | v0.3 |
| Playground WASM | P2 | v0.3 |
| Bennett naïf | P1 | v0.4 |
| Garbage-Free Collector | P1 | v0.4 |
| Aion Academy (3 leçons) | P2 | v0.3 |
| Spec RBF v0.1 | P2 | v0.5 |

**Journeys supportées** : Journey 1 complète + Journey 2 (chercheuse RC)

### Phase 3 : Expansion (Année 2+)

| Composant | Priorité | Déclencheur |
|-----------|----------|------------|
| Intégration Vaire/FPGA | P1 | Hardware disponible |
| Bindings Python (PyO3) | P2 | Demande communauté ML |
| Framework Aion ML (RevNets) | P2 | Paper validé + traction |
| Reversibility Index | P3 | Adoption communautaire |
| CLI standalone | P2 | Adoption > 1000/mois |

### Stratégie de Mitigation des Risques

| Risque | Phase impactée | Mitigation | Trigger contingence |
|--------|---------------|------------|-------------------|
| Adoption lente (RC trop exotique) | Phase 1 | Pitch "debugger", pas "RC" | < 50 stars après 3 mois → pivoter messaging |
| Vaire SDK propriétaire | Phase 3 | Publier Aion-OS avant, standard RBF | Vaire annonce SDK → accélérer collaboration |
| Burnout mainteneur solo | Phase 1-2 | Candidater grants, recruter contributeurs | Pas de contributor après 6 mois → réduire scope |
| `#[reversible]` trop restrictif | Phase 1 | Mode `checked` (runtime) en fallback | > 50% des users demandent relaxation |
| Bennett explosion mémoire | Phase 2 | Budget configurable, SQUARE | Benchmarks montrent > 10× mémoire → documenter limites |

---

## Exigences Fonctionnelles

### Capacité 1 : Types Linéaires et Cellules Réversibles

- **FR01** : Le développeur peut créer une `QuantumCell<T>` qui encapsule une valeur non-copiable et non-destructible
- **FR02** : Le système rejette au compile-time toute tentative de `Clone` ou `Copy` d'une `QuantumCell`
- **FR03** : Le système panique au runtime si une `QuantumCell` est droppée sans avoir été consommée via `consume()`
- **FR04** : Le développeur peut consommer une `QuantumCell` via `consume(self) -> T` sans déclencher le Drop

### Capacité 2 : Portes Logiques Réversibles

- **FR05** : Le développeur peut appliquer une porte Pauli-X (NOT) sur une `QuantumCell<BitVec>`
- **FR06** : Le développeur peut appliquer une porte CNOT avec un bit de contrôle et un bit cible
- **FR07** : Le développeur peut appliquer une porte de Toffoli (CCNOT) avec deux bits de contrôle et un bit cible
- **FR08** : Chaque porte implémente le trait `ReversibleOp` avec `execute()` et `undo()`
- **FR09** : Pour toute porte g et tout input x : `g.undo(g.execute(x)) == x` (propriété de réversibilité)
- **FR10** : Le développeur peut créer des portes custom en implémentant `ReversibleOp`

### Capacité 3 : Macro `#[reversible]`

- **FR11** : Le développeur peut annoter une fonction avec `#[reversible]` pour activer la vérification de réversibilité
- **FR12** : Le compilateur rejette toute assignation destructive (`x = expr`) dans un bloc `#[reversible]`
- **FR13** : Le compilateur autorise les opérations réversibles : `+=`, `-=`, `^=`, `swap`
- **FR14** : Le compilateur rejette `mem::forget`, `mem::drop`, et les opérations I/O dans un bloc `#[reversible]`
- **FR15** : Les messages d'erreur du compilateur pointent vers le token exact problématique avec une suggestion de correction
- **FR16** : La macro génère automatiquement le code inverse pour chaque bloc `#[reversible]`

### Capacité 4 : Exécution Bidirectionnelle

- **FR17** : Le développeur peut exécuter un `ReversibleBlock` en mode forward via `forward()`
- **FR18** : Le développeur peut exécuter un `ReversibleBlock` en mode backward via `backward()`
- **FR19** : Le développeur peut créer un checkpoint de l'état courant via `checkpoint()`
- **FR20** : Le développeur peut restaurer un état précédent via `restore(checkpoint_id)`
- **FR21** : L'exécution backward restitue exactement l'état d'entrée original

### Capacité 5 : Garbage-Free Collector (v0.4+)

- **FR22** : Le système maintient une pile miroir (LIFO) des ancilla bits pour chaque registre
- **FR23** : Le système peut uncomputer (dé-calculer) les étapes intermédiaires pour restaurer les registres à |0⟩
- **FR24** : Le développeur peut vérifier qu'une exécution est garbage-free via `verify_garbage_free()`
- **FR25** : Le développeur peut configurer un budget mémoire maximum pour la pile d'ancilla

### Capacité 6 : Algorithme de Bennett (v0.4+)

- **FR26** : Le système peut transformer automatiquement un calcul irréversible annoté en calcul réversible via Bennett
- **FR27** : Le développeur peut configurer le paramètre ε pour le trade-off espace/temps de Bennett
- **FR28** : Le système génère un graphe de calcul (DAG) visualisable pour le chemin de pebbling

### Capacité 7 : Abstraction Hardware

- **FR29** : Le développeur peut implémenter le trait `ExecutionBackend` pour cibler un hardware spécifique
- **FR30** : Le système fournit `SimulatedCPU` comme backend par défaut
- **FR31** : Le choix du backend est transparent pour le code utilisateur (même API forward/backward)

### Capacité 8 : Tests et Validation

- **FR32** : Le système fournit un helper proptest pour vérifier la réversibilité de toute implémentation `ReversibleOp`
- **FR33** : Le développeur peut exécuter `cargo test` pour valider la réversibilité de tous les composants

---

## Exigences Non-Fonctionnelles

### Performance

- **NFR01** : Les portes Toffoli scalaires exécutent > 1 milliard d'opérations/seconde sur un CPU moderne (benchmarké via criterion)
- **NFR02** : Les portes Toffoli SIMD (AVX2) exécutent > 50 milliards d'opérations/seconde (v0.2)
- **NFR03** : L'overhead de `#[reversible]` à la compilation est < 5% du temps de compilation total
- **NFR04** : L'overhead runtime de l'exécution réversible vs code natif est documenté et benchmarké pour chaque release

### Fiabilité

- **NFR05** : La propriété `∀x: undo(execute(x)) == x` est vérifiée par proptest avec 100 000+ inputs aléatoires pour chaque porte
- **NFR06** : La CI GitHub Actions exécute la suite de tests complète sur chaque PR
- **NFR07** : Aucun `unsafe` dans le code utilisateur — tout le `unsafe` est confiné au core (`ManuallyDrop` dans `QuantumCell`)

### Portabilité

- **NFR08** : Le core (sans SIMD) compile sur toute plateforme supportée par Rust stable
- **NFR09** : Le SIMD est activable par feature flag (`simd` pour nightly, `stable-simd` pour pulp)
- **NFR10** : La crate compile vers WASM pour le playground navigateur (v0.2+)

### Utilisabilité Développeur

- **NFR11** : Le quickstart (README → premier programme `#[reversible]` fonctionnel) prend < 5 minutes
- **NFR12** : Chaque module public a une documentation rustdoc complète avec exemples exécutables
- **NFR13** : Les erreurs de `#[reversible]` produisent des messages explicites pointant vers le token problématique avec une suggestion de correction

### Sécurité

- **NFR14** : Aucune dépendance avec des vulnérabilités connues (vérifié par `cargo audit` en CI)
- **NFR15** : La supply chain crates est auditée — dépendances minimales, toutes de confiance (syn, quote, proptest, petgraph = crates matures)
