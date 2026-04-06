---
stepsCompleted: ['step-01-validate-prerequisites', 'step-02-design-epics', 'step-03-create-stories', 'step-04-final-validation']
inputDocuments:
  - 'prd.md'
  - 'architecture.md'
---

# Rewind (Aion-OS) - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for Rewind (Aion-OS), decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

- FR01: Le développeur peut créer une QuantumCell<T> qui encapsule une valeur non-copiable et non-destructible
- FR02: Le système rejette au compile-time toute tentative de Clone ou Copy d'une QuantumCell
- FR03: Le système panique au runtime si une QuantumCell est droppée sans avoir été consommée via consume()
- FR04: Le développeur peut consommer une QuantumCell via consume(self) -> T sans déclencher le Drop
- FR05: Le développeur peut appliquer une porte Pauli-X (NOT) sur une QuantumCell<BitVec>
- FR06: Le développeur peut appliquer une porte CNOT avec un bit de contrôle et un bit cible
- FR07: Le développeur peut appliquer une porte de Toffoli (CCNOT) avec deux bits de contrôle et un bit cible
- FR08: Chaque porte implémente le trait ReversibleOp avec execute() et undo()
- FR09: Pour toute porte g et tout input x : g.undo(g.execute(x)) == x
- FR10: Le développeur peut créer des portes custom en implémentant ReversibleOp
- FR11: Le développeur peut annoter une fonction avec #[reversible] pour activer la vérification de réversibilité
- FR12: Le compilateur rejette toute assignation destructive (x = expr) dans un bloc #[reversible]
- FR13: Le compilateur autorise les opérations réversibles : +=, -=, ^=, swap
- FR14: Le compilateur rejette mem::forget, mem::drop, et les opérations I/O dans un bloc #[reversible]
- FR15: Les messages d'erreur du compilateur pointent vers le token exact problématique avec une suggestion de correction
- FR16: La macro génère automatiquement le code inverse pour chaque bloc #[reversible]
- FR17: Le développeur peut exécuter un ReversibleBlock en mode forward via forward()
- FR18: Le développeur peut exécuter un ReversibleBlock en mode backward via backward()
- FR19: Le développeur peut créer un checkpoint de l'état courant via checkpoint()
- FR20: Le développeur peut restaurer un état précédent via restore(checkpoint_id)
- FR21: L'exécution backward restitue exactement l'état d'entrée original
- FR22: Le système maintient une pile miroir (LIFO) des ancilla bits pour chaque registre
- FR23: Le système peut uncomputer les étapes intermédiaires pour restaurer les registres à |0⟩
- FR24: Le développeur peut vérifier qu'une exécution est garbage-free via verify_garbage_free()
- FR25: Le développeur peut configurer un budget mémoire maximum pour la pile d'ancilla
- FR26: Le système peut transformer automatiquement un calcul irréversible annoté en calcul réversible via Bennett
- FR27: Le développeur peut configurer le paramètre ε pour le trade-off espace/temps de Bennett
- FR28: Le système génère un graphe de calcul (DAG) visualisable pour le chemin de pebbling
- FR29: Le développeur peut implémenter le trait ExecutionBackend pour cibler un hardware spécifique
- FR30: Le système fournit SimulatedCPU comme backend par défaut
- FR31: Le choix du backend est transparent pour le code utilisateur
- FR32: Le système fournit un helper proptest pour vérifier la réversibilité de toute implémentation ReversibleOp
- FR33: Le développeur peut exécuter cargo test pour valider la réversibilité de tous les composants

### NonFunctional Requirements

- NFR01: Portes Toffoli scalaires > 1 milliard ops/sec
- NFR02: Portes Toffoli SIMD AVX2 > 50 milliards ops/sec (v0.2)
- NFR03: Overhead #[reversible] compilation < 5%
- NFR04: Overhead runtime documenté et benchmarké
- NFR05: Propriété undo(execute(x)) == x vérifiée proptest 100K+ inputs
- NFR06: CI GitHub Actions sur chaque PR
- NFR07: Aucun unsafe dans le code utilisateur
- NFR08: Core compile sur toute plateforme Rust stable
- NFR09: SIMD activable par feature flag
- NFR10: Compilation WASM (v0.2+)
- NFR11: Quickstart < 5 minutes
- NFR12: Rustdoc complète avec exemples exécutables
- NFR13: Erreurs #[reversible] explicites avec suggestion de correction
- NFR14: Aucune dépendance avec vulnérabilités (cargo audit)
- NFR15: Supply chain crates auditée, dépendances minimales

### Additional Requirements

- Workspace Cargo multi-crate (7 crates : rewind-core, rewind-gates, rewind-gc, rewind-dsl, rewind-bennett, rewind, rewind-playground)
- Arena allocator bumpalo avec indices typés (RegisterId, AncillaId)
- Match dispatch pour l'exécution VM (enum Opcode)
- SoA data layout (BitPlane) pour SIMD
- Feature flags : simd, stable-simd, bennett, wasm
- no_std compatible core
- Double licence Apache 2.0 / MIT
- Publication crates.io
- README avec GIF animé forward/backward + quickstart

### UX Design Requirements

N/A — SDK sans interface utilisateur graphique.

### FR Coverage Map

| FR | Epic | Description |
|----|------|-------------|
| FR01-04 | Epic 1 | QuantumCell — types linéaires |
| FR08-09 | Epic 1 | Trait ReversibleOp + propriété réversibilité |
| FR29-31 | Epic 1 | Trait ExecutionBackend + SimulatedCPU |
| FR05-07 | Epic 2 | Portes Pauli-X, CNOT, Toffoli |
| FR10 | Epic 2 | Portes custom |
| FR17-21 | Epic 3 | Forward, backward, checkpoint, restore |
| FR32-33 | Epic 3 | Proptest helpers + cargo test |
| FR11-16 | Epic 4 | Macro #[reversible] — validation + codegen |
| FR22-25 | Epic 5 | Garbage-Free Collector — pile miroir, uncompute, budget |
| FR26-28 | Epic 6 | Algorithme de Bennett — DAG, pebbling, executor |
| NFR01-04 | Epic 7 | Performance SIMD + benchmarks |
| NFR05-15 | Transversal | Fiabilité, portabilité, DX, sécurité (intégrés dans chaque epic) |

**Couverture** : 33/33 FRs couverts, 15/15 NFRs adressés.

## Epic List

### Epic 1 : Fondation du Calcul Réversible
Le développeur peut créer et manipuler des types de données réversibles (QuantumCell), définir des opérations réversibles via le trait ReversibleOp, et cibler différents backends d'exécution.
**FRs couverts :** FR01, FR02, FR03, FR04, FR08, FR09, FR29, FR30, FR31

### Epic 2 : Portes Logiques Réversibles
Le développeur peut effectuer des calculs réversibles en utilisant les portes standard (Pauli-X, CNOT, Toffoli) et créer ses propres portes custom.
**FRs couverts :** FR05, FR06, FR07, FR10

### Epic 3 : Exécution Bidirectionnelle & Tests
Le développeur peut exécuter ses programmes en avant et en arrière, créer des checkpoints, restaurer des états, et vérifier automatiquement la réversibilité via proptest.
**FRs couverts :** FR17, FR18, FR19, FR20, FR21, FR32, FR33

### Epic 4 : DSL — Macro #[reversible]
Le développeur peut annoter ses fonctions avec #[reversible] pour obtenir une vérification compile-time de la réversibilité, avec rejet des opérations irréversibles et génération automatique du code inverse.
**FRs couverts :** FR11, FR12, FR13, FR14, FR15, FR16

### Epic 5 : Garbage-Free Collector
Le développeur peut exécuter des calculs sans fuite d'information grâce à l'uncomputation automatique des ancilla bits, avec vérification garbage-free et budget mémoire configurable.
**FRs couverts :** FR22, FR23, FR24, FR25

### Epic 6 : Compilation Réversible (Bennett)
Le développeur peut transformer automatiquement du calcul irréversible en réversible via l'algorithme de Bennett, avec contrôle du trade-off espace/temps et visualisation du graphe de calcul.
**FRs couverts :** FR26, FR27, FR28

### Epic 7 : Optimisations SIMD & Performance
Le développeur bénéficie de performances optimisées grâce aux portes SIMD (AVX2/AVX-512/NEON), avec des benchmarks publiés et reproductibles.
**FRs couverts :** NFR01, NFR02, NFR03, NFR04

---

## Epic 1 : Fondation du Calcul Réversible

**Objectif :** Établir les types fondamentaux, traits, et infrastructure du workspace Cargo qui servent de base à tout le projet Rewind.

### Story 1.1 : Initialisation du Workspace Cargo

As a développeur Rewind,
I want un workspace Cargo multi-crate correctement structuré,
So that je peux développer les composants de manière modulaire et indépendante.

**Acceptance Criteria:**

**Given** un nouveau workspace Cargo
**When** j'exécute `cargo build`
**Then** le workspace compile sans erreur avec les 7 crates : rewind-core, rewind-gates, rewind-gc, rewind-dsl, rewind-bennett, rewind, rewind-playground
**And** chaque crate a son Cargo.toml avec les dépendances inter-crates correctes
**And** les fichiers LICENSE-APACHE, LICENSE-MIT sont à la racine
**And** le README.md contient le pitch "Information is Sacred" et le quickstart placeholder

### Story 1.2 : QuantumCell — Type Linéaire

As a développeur Rust,
I want créer des QuantumCell<T> qui ne peuvent être ni copiées ni détruites,
So that mes données réversibles respectent la logique linéaire (usage exactement une fois).

**Acceptance Criteria:**

**Given** une QuantumCell<T> créée avec `QuantumCell::new(value)`
**When** j'essaie de la `Clone` ou `Copy`
**Then** le compilateur rejette avec une erreur

**Given** une QuantumCell<T>
**When** elle sort du scope sans avoir été consommée via `consume()`
**Then** le programme panique avec "QuantumCell dropped without being consumed — information lost"

**Given** une QuantumCell<T>
**When** j'appelle `consume(self) -> T`
**Then** la valeur est retournée et aucun panic ne se produit

### Story 1.3 : Trait ReversibleOp et Types d'État

As a développeur Rewind,
I want un trait ReversibleOp qui définit le contrat de réversibilité,
So that toute opération du système est garantie inversible.

**Acceptance Criteria:**

**Given** le trait `ReversibleOp` avec `execute()` et `undo()`
**When** un type implémente `ReversibleOp`
**Then** il doit fournir `execute(state) -> (state, ancilla)` et `undo(state, ancilla) -> state`

**Given** les types `State`, `RegisterId(u32)`, `AncillaId(u32)`, `CheckpointId(u32)`
**When** utilisés dans le système
**Then** ils fournissent un typage fort empêchant la confusion d'indices

**Given** le type `BitPlane` (SoA layout pour SIMD futur)
**When** créé avec une taille
**Then** il stocke les bits dans un `Vec<u64>` et supporte les opérations XOR, AND, NOT

### Story 1.4 : ExecutionBackend et SimulatedCPU

As a développeur Rewind,
I want un trait ExecutionBackend avec une implémentation SimulatedCPU par défaut,
So that le code utilisateur est découplé du hardware d'exécution.

**Acceptance Criteria:**

**Given** le trait `ExecutionBackend`
**When** implémenté par `SimulatedCPU`
**Then** il exécute les opcodes réversibles via match dispatch dans une boucle

**Given** un `SimulatedCPU` avec un programme d'opcodes
**When** `execute_forward()` est appelé
**Then** tous les opcodes sont exécutés séquentiellement

**Given** du code utilisateur utilisant l'API publique
**When** le backend change de `SimulatedCPU` à un futur `VaireBackend`
**Then** le code utilisateur ne change pas (même API forward/backward)

### Story 1.5 : Gestion d'Erreurs RewindError

As a développeur Rust,
I want des erreurs typées et explicites quand quelque chose échoue,
So that je comprends immédiatement ce qui ne va pas dans mon calcul réversible.

**Acceptance Criteria:**

**Given** l'enum `RewindError` avec les variantes : InformationLost, CheckpointNotFound, GarbageRemaining, MemoryBudgetExceeded
**When** une erreur se produit
**Then** le message est clair et actionnable (ex: "Ancilla stack not empty after uncomputation — 3 bits remain")

---

## Epic 2 : Portes Logiques Réversibles

**Objectif :** Implémenter les portes de calcul réversible standard et permettre la création de portes custom.

### Story 2.1 : Porte Pauli-X (NOT Réversible)

As a développeur Rust,
I want appliquer une porte NOT réversible sur un BitPlane,
So that je peux inverser tous les bits d'un registre de manière réversible.

**Acceptance Criteria:**

**Given** un `BitPlane` contenant des bits
**When** `pauli_x(bitplane)` est appelé
**Then** tous les bits sont inversés

**Given** un BitPlane après application de pauli_x
**When** pauli_x est appliqué une seconde fois
**Then** le BitPlane revient exactement à son état original (auto-inverse)

**Given** proptest avec 100K+ inputs aléatoires
**When** la propriété `pauli_x(pauli_x(x)) == x` est testée
**Then** 100% des tests passent

### Story 2.2 : Porte CNOT (Controlled-NOT)

As a développeur Rust,
I want appliquer une porte CNOT avec contrôle et cible,
So that je peux effectuer des XOR conditionnels réversibles.

**Acceptance Criteria:**

**Given** un BitPlane `control` et un BitPlane `target`
**When** `cnot(control, target)` est appelé
**Then** target devient `target XOR control`, control reste inchangé

**Given** proptest avec 100K+ inputs
**When** la propriété `cnot_undo(cnot(ctrl, tgt)) == (ctrl, tgt)` est testée
**Then** 100% des tests passent

### Story 2.3 : Porte de Toffoli (CCNOT)

As a développeur Rust,
I want appliquer une porte de Toffoli universelle,
So that je peux effectuer n'importe quel calcul réversible classique.

**Acceptance Criteria:**

**Given** deux BitPlanes de contrôle `c1`, `c2` et un BitPlane cible `target`
**When** `toffoli(c1, c2, target)` est appelé
**Then** target devient `target XOR (c1 AND c2)`, c1 et c2 restent inchangés

**Given** proptest avec 100K+ inputs
**When** la propriété `toffoli_undo(toffoli(c1, c2, t)) == (c1, c2, t)` est testée
**Then** 100% des tests passent

**Given** le benchmark criterion
**When** toffoli scalaire est benchmarké
**Then** le throughput est > 1 milliard d'opérations/seconde

### Story 2.4 : Portes Custom via ReversibleOp

As a développeur Rust,
I want créer mes propres portes réversibles en implémentant ReversibleOp,
So that je peux étendre Rewind avec des opérations spécifiques à mon domaine.

**Acceptance Criteria:**

**Given** un type custom implémentant `ReversibleOp`
**When** `execute()` et `undo()` sont fournis
**Then** la porte est utilisable dans le moteur d'exécution

**Given** le helper proptest `assert_reversible!(custom_gate)`
**When** exécuté avec 100K+ inputs
**Then** la propriété `undo(execute(x)) == x` est vérifiée automatiquement

---

## Epic 3 : Exécution Bidirectionnelle & Tests

**Objectif :** Permettre l'exécution de programmes réversibles en avant et en arrière, avec checkpoints et restauration d'état.

### Story 3.1 : Exécution Forward

As a développeur Rust,
I want exécuter un bloc réversible en mode forward,
So that mon calcul produit un résultat tout en préservant la possibilité de remonter.

**Acceptance Criteria:**

**Given** un `ReversibleBlock` contenant une séquence d'opcodes
**When** `forward(block)` est appelé
**Then** chaque opcode est exécuté séquentiellement et l'état final est retourné

**Given** un ReversibleBlock avec des portes Toffoli
**When** forward est exécuté
**Then** le résultat est mathématiquement correct (vérifié par cas de test connus)

### Story 3.2 : Exécution Backward

As a développeur Rust,
I want exécuter un bloc réversible en mode backward,
So that je peux remonter l'exécution à l'état d'entrée original.

**Acceptance Criteria:**

**Given** un ReversibleBlock après exécution forward
**When** `backward(block)` est appelé
**Then** l'état revient exactement à l'input original

**Given** proptest avec 100K+ programmes aléatoires
**When** la propriété `backward(forward(block, input)) == input` est testée
**Then** 100% des tests passent

### Story 3.3 : Checkpoint et Restore

As a développeur Rust,
I want créer des points de sauvegarde et y revenir,
So that je peux explorer différents chemins d'exécution sans tout recalculer.

**Acceptance Criteria:**

**Given** un état d'exécution en cours
**When** `checkpoint(state)` est appelé
**Then** un `CheckpointId` est retourné et l'état est sauvegardé

**Given** un CheckpointId valide
**When** `restore(checkpoint_id)` est appelé
**Then** l'état est restauré exactement à ce qu'il était au moment du checkpoint

**Given** un CheckpointId invalide
**When** `restore(invalid_id)` est appelé
**Then** `RewindError::CheckpointNotFound` est retourné

### Story 3.4 : Framework de Test Proptest

As a développeur Rust,
I want un helper proptest qui vérifie automatiquement la réversibilité,
So that chaque composant ReversibleOp est testé avec rigueur.

**Acceptance Criteria:**

**Given** la macro `assert_reversible!(gate, strategy)`
**When** exécutée dans un test
**Then** elle génère 100K+ inputs aléatoires et vérifie `undo(execute(x)) == x` pour chacun

**Given** `cargo test` exécuté à la racine du workspace
**When** tous les tests s'exécutent
**Then** toutes les portes et tous les composants passent les tests de réversibilité

### Story 3.5 : README, Quickstart & Exemples

As a développeur Rust découvrant Rewind,
I want un README avec GIF, quickstart, et exemples fonctionnels,
So that je comprends et utilise Rewind en moins de 5 minutes.

**Acceptance Criteria:**

**Given** le README.md
**When** un développeur le lit
**Then** il contient : GIF animé forward/backward, pitch "Information is Sacred", quickstart `cargo add rewind`, et 3 exemples

**Given** les exemples dans `examples/`
**When** `cargo run --example hello_rewind` est exécuté
**Then** le programme compile et s'exécute correctement, démontrant la réversibilité

**Given** la crate publiée sur crates.io
**When** un développeur fait `cargo add rewind`
**Then** la crate s'installe et compile sans erreur

---

## Epic 4 : DSL — Macro #[reversible]

**Objectif :** Fournir une vérification compile-time de la réversibilité via une macro procédurale.

### Story 4.1 : Infrastructure Proc-Macro

As a développeur Rewind,
I want la crate rewind-dsl avec l'infrastructure de macro procédurale,
So that #[reversible] peut parser et transformer le code Rust.

**Acceptance Criteria:**

**Given** la crate `rewind-dsl` avec `proc-macro = true`
**When** un utilisateur annote une fonction avec `#[reversible]`
**Then** la macro reçoit le TokenStream et peut le parser avec syn

**Given** le pipeline parse → validate → codegen
**When** le code est compilé
**Then** l'overhead de compilation est < 5% (vérifié par benchmark)

### Story 4.2 : Validation — Rejet des Opérations Irréversibles

As a développeur Rust,
I want que #[reversible] rejette les opérations irréversibles au compile-time,
So that je suis guidé vers du code correct par le compilateur.

**Acceptance Criteria:**

**Given** une assignation destructive `x = expr` dans un bloc #[reversible]
**When** le code est compilé
**Then** le compilateur émet une erreur pointant vers le token exact avec suggestion "use ^= or += instead"

**Given** `mem::forget` ou `mem::drop` dans un bloc #[reversible]
**When** le code est compilé
**Then** le compilateur émet une erreur "forbidden in reversible context"

**Given** des opérations `+=`, `-=`, `^=`, `swap` dans un bloc #[reversible]
**When** le code est compilé
**Then** aucune erreur — ces opérations sont autorisées

### Story 4.3 : Codegen — Génération du Code Inverse

As a développeur Rust,
I want que #[reversible] génère automatiquement le code inverse,
So que backward() fonctionne sans que j'écrive le code inverse moi-même.

**Acceptance Criteria:**

**Given** un bloc #[reversible] contenant `x += 5; y ^= x;`
**When** le code est compilé
**Then** la macro génère un ReversibleBlock avec forward_ops et backward_ops (inverse : `y ^= x; x -= 5;`)

**Given** proptest sur le code généré
**When** forward puis backward sont exécutés
**Then** l'état revient à l'original

---

## Epic 5 : Garbage-Free Collector

**Objectif :** Garantir que le calcul réversible ne laisse aucun bit résiduel après exécution.

### Story 5.1 : Pile Miroir d'Ancilla

As a développeur Rewind,
I want une pile LIFO qui stocke les ancilla bits pendant l'exécution forward,
So que le backward peut les consommer pour restaurer l'état.

**Acceptance Criteria:**

**Given** une exécution forward avec des opérations Toffoli
**When** chaque opération pousse ses ancilla sur la pile
**Then** la pile contient tous les états intermédiaires dans l'ordre LIFO

**Given** une exécution backward
**When** chaque opération pop ses ancilla de la pile
**Then** l'état est correctement restauré et la pile est vide à la fin

### Story 5.2 : Vérification Garbage-Free

As a développeur Rust,
I want vérifier qu'aucun ancilla bit ne reste après exécution,
So que je prouve que mon calcul ne fuit aucune information.

**Acceptance Criteria:**

**Given** un calcul complet (forward + backward)
**When** `verify_garbage_free()` est appelé
**Then** il retourne `true` si la pile est vide, `Err(GarbageRemaining(n))` sinon

### Story 5.3 : Budget Mémoire Configurable

As a développeur Rust,
I want limiter la mémoire utilisée par la pile d'ancilla,
So que mon calcul ne consomme pas plus de mémoire que prévu.

**Acceptance Criteria:**

**Given** un budget mémoire de N bytes configuré
**When** la pile d'ancilla atteint le budget
**Then** `RewindError::MemoryBudgetExceeded` est retourné avec les détails

---

## Epic 6 : Compilation Réversible (Bennett)

**Objectif :** Permettre la transformation automatique de calcul irréversible en réversible.

### Story 6.1 : Graphe de Calcul (ComputationGraph)

As a développeur Rewind,
I want représenter un calcul comme un DAG,
So que l'algorithme de Bennett peut planifier l'exécution réversible.

**Acceptance Criteria:**

**Given** une séquence d'opérations
**When** `ComputationGraph::from_ops(ops)` est appelé
**Then** un DAG est construit avec les nœuds (opérations) et les arêtes (dépendances)

### Story 6.2 : Stratégie de Pebbling

As a développeur Rewind,
I want configurer le paramètre ε du trade-off espace/temps,
So que je contrôle le compromis entre mémoire utilisée et temps d'exécution.

**Acceptance Criteria:**

**Given** un ComputationGraph et un ε configurable
**When** `PebblingStrategy::plan(graph, epsilon)` est appelé
**Then** un plan de checkpoints est généré respectant les bornes O(T^(1+ε)) temps, O(S·log(T)) espace

**Given** ε < 0.1 (garde-fou)
**When** le développeur configure ε = 0.01
**Then** un warning est émis expliquant que le facteur caché ε·2^(1/ε) diverge

### Story 6.3 : Exécuteur Réversible Bennett

As a développeur Rust,
I want transformer automatiquement un calcul irréversible en réversible,
So que je n'ai pas à réécrire mon code manuellement.

**Acceptance Criteria:**

**Given** un calcul irréversible annoté pour Bennett
**When** `BennettCompiler::compile(computation)` est appelé
**Then** un ReversibleBlock est généré avec les étapes forward, copy, backward de Bennett

---

## Epic 7 : Optimisations SIMD & Performance

**Objectif :** Maximiser les performances des portes réversibles via SIMD.

### Story 7.1 : Portes SIMD Nightly (std::simd)

As a développeur Rust nightly,
I want des portes Toffoli/CNOT/X optimisées SIMD,
So que mes calculs réversibles sont 8-512× plus rapides.

**Acceptance Criteria:**

**Given** le feature flag `simd` activé (nightly)
**When** les portes sont exécutées sur des BitPlanes
**Then** les opérations utilisent `std::simd` (u64xN) pour paralléliser

**Given** le benchmark criterion
**When** toffoli SIMD AVX2 est benchmarké
**Then** le throughput est > 50 milliards d'opérations/seconde

### Story 7.2 : Portes SIMD Stable (pulp)

As a développeur Rust stable,
I want des portes SIMD sans nightly,
So que je bénéficie de performances optimisées sur Rust stable.

**Acceptance Criteria:**

**Given** le feature flag `stable-simd` activé
**When** les portes sont exécutées
**Then** les opérations utilisent la crate `pulp` pour le SIMD portable

### Story 7.3 : Suite de Benchmarks Publiée

As a développeur évaluant Rewind,
I want des benchmarks reproductibles et publiés,
So que je peux évaluer les performances avant d'adopter.

**Acceptance Criteria:**

**Given** la suite de benchmarks criterion dans `rewind-gates/benches/`
**When** `cargo bench` est exécuté
**Then** les résultats couvrent : scalaire, SIMD nightly, SIMD stable, pour chaque porte
**And** les résultats sont publiés dans le README avec les chiffres clés
