---
title: "Product Brief: Rewind (Aion-OS)"
status: "complete"
created: "2026-04-06"
updated: "2026-04-06"
inputs:
  - "_bmad-output/planning-artifacts/research/domain-calcul-reversible-research-2026-04-06.md"
  - "_bmad-output/planning-artifacts/research/market-aion-os-reversible-computing-research-2026-04-06.md"
  - "_bmad-output/planning-artifacts/research/technical-aion-os-rvm-rust-feasibility-research-2026-04-06.md"
  - "_bmad-output/brainstorming/brainstorming-session-2026-04-06-aion-os.md"
---

# Product Brief : Rewind (Aion-OS)

*"Information is Sacred — elle ne doit jamais être détruite."*

## Résumé Exécutif

**Rewind** est le premier SDK open-source de **calcul nativement réversible** en Rust. Contrairement aux debuggers time-travel existants (rr, UndoDB, WinDbg) qui enregistrent et rejouent des traces d'exécution, Rewind garantit la réversibilité **au niveau du calcul lui-même** — chaque opération est structurellement inversible, vérifiée au compile-time, sans overhead d'enregistrement. Le résultat : un debugging temporel natif, des rollbacks sans coût, et du code prouvablement correct par construction.

Sous le capot, Rewind implémente une Reversible Virtual Machine (RVM) fondée sur les portes de Toffoli, l'algorithme de Bennett, et les types affines de Rust. Le principe fondateur : l'information ne doit jamais être détruite.

Le timing est critique. La crise énergétique de l'IA (1 000 TWh projetés d'ici 2030), les réglementations européennes (EU EED, Data Centre Package Q1 2026), et la première récupération d'énergie sur silicium par Vaire Computing (2025) convergent pour créer une demande structurelle. Pourtant, **aucun SDK open-source natif au calcul réversible n'existe en Rust** — un vide entre la théorie (Janus, RevKit) et le hardware émergent (Vaire). Rewind comble ce vide.

L'écosystème Rust (2,27 millions de développeurs, +17,6%/an) est le vecteur de distribution. La stratégie de lancement cible le **debugging temporel** comme premier cas d'usage — valeur immédiate, adoption par la communauté Rust. Les pivots vers le ML réversible (2028) et le hardware (2030+) ne seront activés qu'après validation de la traction initiale.

## Le Problème

**Les développeurs Rust ne peuvent pas remonter le temps de manière fiable.** Quand un bug se produit après 10 000 itérations, les options sont limitées : logs, `println!` (qui masquent les Heisenbugs), ou re-exécution. Les debuggers time-travel existants (rr, UndoDB) enregistrent des traces — ce qui consomme de la mémoire, ralentit l'exécution, et ne prouve rien sur la correctness du code. L'expérience de debugging est le **pain point en déclin le plus marqué** de l'enquête Rust 2025, avec la Rust Foundation qui lance une enquête dédiée en février 2026.

À plus grande échelle, l'informatique mondiale détruit de l'information à un rythme industriel. Chaque `x = y` écrase l'ancienne valeur de x. Chaque garbage collection efface des états intermédiaires. Cette destruction a un coût physique — le principe de Landauer prouve que chaque bit effacé dissipe un minimum de kT·ln(2) de chaleur. Les data centers, qui consomment déjà 1,5% de l'électricité mondiale, sont piégés par cette physique.

## La Solution

**Rewind** permet d'écrire du code qui s'exécute en avant ET en arrière :

- **`#[reversible]`** — Un attribut Rust qui vérifie au compile-time que la fonction est réversible
- **`forward()` / `backward()`** — Exécution dans les deux sens, nativement
- **`checkpoint()` / `restore()`** — Points de sauvegarde programmables dans l'exécution
- **Step-backward debugging** — Remonter l'exécution instruction par instruction

L'API cache volontairement la théorie. Un développeur n'a jamais besoin de connaître Toffoli, Bennett ou Landauer pour utiliser Rewind — comme Docker n'exige pas de comprendre les cgroups.

**Architecture modulaire :** Collection de crates composables (`rewind-gates`, `rewind-cell`, `rewind-gc`, `rewind-dsl`) plutôt qu'une VM monolithique, respectant la culture modulaire de Rust.

## Ce Qui Rend Rewind Différent

| Dimension | Rewind | Alternatives |
|-----------|--------|-------------|
| **Paradigme** | Réversibilité native (0 perte d'info, 0 enregistrement) | rr/UndoDB : record-replay (overhead mémoire + CPU) |
| **Vérification** | Compile-time (`#[reversible]` rejette le code irréversible) | Aucune — les debuggers TT ne vérifient pas la correctness |
| **Langage** | Rust — seul langage mainstream avec types affines natifs | rr (C/C++ seulement), Janus (académique), RevKit (circuits) |
| **Performance** | SIMD (512 portes/instruction AVX-512), 0 overhead d'enregistrement | rr : 1.2-5× slowdown, UndoDB : 2-5× slowdown |
| **Approche** | API "Sans Théorie" — forward/backward/checkpoint/restore | rr : gdb interface, RevKit : théorie requise |
| **Cas d'usage** | Debugging + rollback transactionnel + fuzzing + sécurité | rr : debugging seulement |

**Avantage défendable :** Rust est le seul langage mainstream dont le système de types (ownership, move semantics) est structurellement aligné avec la logique linéaire qui fonde le calcul réversible. Cet avantage est architectural, pas juste implémentationnel.

## Qui Utilise Rewind

**Segment primaire : Développeurs Rust** (2,27M, +17,6%/an)
Ils cherchent un meilleur debugging et sont attirés par les projets techniquement pionniers. Le moment "aha" : la première fois qu'ils tapent `step backward` et voient l'état du programme remonter dans le temps.

**Segment d'amorçage : Chercheurs en calcul réversible**
Communauté de niche mais influente (conférence RC, projet EU E-CoRe). Ils manquent d'outils modernes — Rewind est le pont entre leur théorie et du code exécutable.

**Segment secondaire : Ingénieurs sécurité & DevSecOps**
Le calcul réversible permet le replay forensique exact d'un incident, le fuzzing réversible (mutation + retour), et l'audit RGPD (prouver la destruction d'information). Partenariats naturels : Sentry, DataDog, outils d'observabilité.

**Segments futurs :** Ingénieurs ML (Bennett = gradient checkpointing, RevNets mémoire O(1)), systèmes embarqués/IoT (debug firmware, tolérance aux fautes), opérateurs data center (quand les chips Vaire arrivent en 2027+).

## Critères de Succès

| Métrique | Objectif | Échéance |
|----------|----------|----------|
| Front page Hacker News | 1 post avec 100+ commentaires | Q3 2026 |
| Installs `cargo add rewind` | 500/mois | Q2 2027 |
| Contributeurs actifs GitHub | 20 (avec PRs mergées) | Q4 2027 |
| Crates dépendantes de rewind | 10 | Q4 2027 |
| Paper soumis (RC 2026 ou NeurIPS) | 1 ("Bennett = Gradient Checkpointing") | Q3 2026 |
| Prototype fonctionnel step-backward | Demo publique | Q3 2026 |
| Premier POC enterprise ou partenariat Vaire | 1 | Q2 2028 |

## Périmètre

**v0.1 "Rewind" — ce qui est DEDANS :**
- `QuantumCell<T>` (type linéaire forcé par Drop+panic)
- Portes réversibles : Toffoli, CNOT, Pauli-X (scalaire)
- `#[reversible]` attribut (sous-ensemble : +=, ^=, swap)
- API publique : forward/backward/checkpoint/restore
- Tests property-based (proptest : `∀x: undo(execute(x)) == x`)
- Trait `ExecutionBackend` (abstraction hardware future)
- Double licence Apache 2.0 / MIT

**Ce qui est DEHORS (v0.1) :**
- Optimisations SIMD (v0.2)
- Algorithme de Bennett automatique (v0.4)
- Garbage-Free Collector optimisé (v0.5)
- Intégration hardware Vaire/FPGA (v1.0+)
- Bindings Python/C/WASM (futur)

## Vision

Si Rewind réussit, il devient la **couche d'abstraction de référence** entre le code développeur et le hardware réversible de demain. La spec ouverte **RBF (Reversible Bytecode Format)** et le **Reversibility Index** positionnent Rewind comme standard de facto.

**Feuille de route — priorité séquentielle, pas parallèle :**

1. **2026-2027 — FOCUS UNIQUE : "Rewind"** — Debugger temporel + SDK RC pour Rust. Objectif : traction communautaire mesurable (500 installs/mois, 20 contributeurs). Validation avant tout pivot.
2. **2028-2030 — Si traction validée : "Aion ML"** — Framework RevNet/gradient checkpointing. Activation uniquement si la communauté et/ou un partenariat ML le justifient.
3. **2030+ — Si hardware disponible : "Aion Platform"** — SDK pour chips réversibles. Activation uniquement quand Vaire ou un autre fabricant livre du hardware.

**Monétisation :** Open-source core (Apache 2.0/MIT). Revenus potentiels via consulting, certification "Reversibility Index", et support enterprise (open core). Pas de monétisation prévue en phase 1 — construction communautaire uniquement.

**Partenariats stratégiques :** Vaire Computing (hardware), E-CoRe/EU HORIZON (financement recherche), Rust Foundation (grants), universités (adoption pédagogique — INRIA, ETH, MIT CSAIL).

L'information est sacrée. Rewind est le premier outil qui la traite comme telle.
