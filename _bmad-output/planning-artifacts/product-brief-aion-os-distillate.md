---
title: "Product Brief Distillate: Rewind (Aion-OS)"
type: llm-distillate
source: "product-brief-aion-os.md"
created: "2026-04-06"
purpose: "Token-efficient context for downstream PRD creation — contains ALL overflow detail from domain research, market research, technical feasibility, and brainstorming sessions"
---

# Distillate : Rewind (Aion-OS) — Detail Pack pour PRD

## Recherche de Domaine — Fondations Théoriques

- **Principe de Landauer** : effacer 1 bit = kT·ln(2) ≈ 0.018 eV à température ambiante. Vérifié expérimentalement (particules colloïdales). Confiance 10/10.
- **Porte de Toffoli** (CCNOT, 1980) : universelle pour le calcul réversible classique. `(a,b,c) → (a, b, c⊕(a·b))`. Auto-inverse. Décomposée en 6 CNOT en implémentation quantique (optimal prouvé).
- **Algorithme de Bennett** (1973) : transforme calcul irréversible temps T, espace S en réversible temps O(T^(1+ε)), espace O(S·log(T)). ATTENTION facteur caché ε·2^(1/ε) dans la borne spatiale. Raffinement Levine-Sherman : Θ(T^(1+ε)/S^ε), Θ(S·(1+ln(T/S))).
- **Logique linéaire de Girard** (1987) : chaque hypothèse utilisée exactement une fois. Correspond aux types linéaires. Rust implémente des types AFFINES (au plus une fois), pas linéaires (exactement une fois). Différence critique pour QuantumCell.
- **Ancilla bits** : bits temporaires du calcul réversible. Types : burnable (OFF initial, libre ensuite) et zeroed (OFF initial, doit rester OFF — nécessite uncomputation). Stratégie SQUARE pour réutilisation sélective.
- **Bennett = Gradient Checkpointing** : connexion formelle inédite. L'algorithme de Bennett (1973) et le gradient checkpointing de Chen (2016) sont structurellement le même algorithme. Paper à soumettre.

## Recherche de Marché — Chiffres Clés

- **Green Data Centers** : $48.26B (2025) → $155.75B (2030), CAGR 26.4%
- **AI Inference** : $106B (2025) → $255B (2030). 80-90% du compute IA = inférence. 165-326 TWh/an d'ici 2028.
- **Green IT Services** : $32.53B (2025) → $94.65B (2031), CAGR 19.5%
- **Rust ecosystem** : 2.27M devs (709K primary), 193K+ crates, 161B+ downloads. 45.5% orgs en usage non-trivial (+17.6%/an). 83% admiration (9e année #1 Stack Overflow).
- **Vaire Computing** : seul acteur commercial RC. $4.5M seed (2024). Chip "Ice River" 50% récupération énergie (2025). Michael Frank (ex-Sandia) a rejoint. Chip AI inference prévu 2027. Production 2027. Potentiel 4000× à horizon 10-15 ans.
- **Conférence RC 2026** : 9-10 juillet 2026. 18ème édition. Deadline de soumission à vérifier.
- **E-CoRe** : projet EU HORIZON pour structurer la communauté RC en Europe. Cibles : ML, blockchains, drones.
- **Coûts PJM** : $2.2B → $14.7B en une enchère de capacité. Vacance data centers US : record 1.6%.

## Recherche de Marché — Segments Clients Détaillés

- **Devs Rust (primaire)** : Pain points = compile times (#1, 27.9%), debugging (en déclin, enquête Rust Foundation fév 2026), complexité (41.6%). Onboarding 3-6 mois. Adoption via HN/Reddit (43.5%), stars GitHub, recommandation pairs. "Grab and go" = critère crate adoption.
- **Chercheurs RC (amorçage)** : Janus (1986, formalisé 2007), RevKit (C++, partiellement maintenu). Manquent d'outils modernes. Cycle adoption : mois-années (paper-driven). Reproductibilité et licence OSS exigées.
- **Sécurité/DevSecOps (secondaire)** : Forensic replay, fuzzing réversible, audit RGPD. Partenariats : Sentry, DataDog, CrowdStrike.
- **Ingénieurs IA/ML (futur)** : Reasoning models (o1) = 10-100× plus de compute. Quantization = palliatif incrémental. Modèles MoE = 3-5× savings. Coût inférence reasoning pourrait dépasser coût salarial employés augmentés.
- **Opérateurs DC (long terme)** : EU EED reporting obligatoire ≥500kW. Data Centre Package Q1 2026. Neutralité carbone 2030. 85% rejettent fournisseurs IT pour raisons ESG. Cycle achat 6-18 mois.

## Recherche de Marché — Concurrence

- **rr (Mozilla)** : record-replay debugger pour Linux C/C++. 1.2-5× slowdown. Pas de vérification compile-time. Pas de Rust natif. Pas de réversibilité computationnelle.
- **UndoDB / LiveRecorder (Perforce)** : commercial. 2-5× slowdown. Enterprise-focused.
- **Janus** : r-Turing complet. Pas de heap, pas de SIMD, pas de types linéaires, niche académique pure.
- **RevKit** : C++, synthèse de circuits réversibles. Focus hardware, pas exécution logicielle.
- **Qiskit/Cirq** : portes Toffoli communes mais paradigme quantique, pas classique réversible. 70% des devs quantiques utilisent OSS → vivier de recrutement.
- **Wasmtime/Wasmer** : modèles à suivre pour adoption d'un runtime spécialisé en Rust (pas concurrents directs).
- **NVIDIA Dynamo** : 30× boost throughput inférence. Complémentaire, pas substitut.
- **CodeCarbon/Kepler** : mesurent la consommation, ne la résolvent pas.

## Faisabilité Technique — 6 Défis Validés

- **Défi 1 (QuantumCell linéaire)** : Pattern UseOnce<T> + Drop + ManuallyDrop. Limitation : mem::forget contournable (mitigation : aion_block! l'interdit). Crate `linear_type` existe. Proposition future Rust : trait Leave (Niko Matsakis). Faisabilité : OUI, phase v0.1.
- **Défi 2 (Toffoli SIMD)** : XOR + AND = purement bitwise = SIMD trivial. std::simd (nightly) ou pulp/wide (stable). AVX-512 = 512 portes/instruction. Faisabilité : OUI, phase v0.1-v0.2.
- **Défi 3 (aion_block! macro)** : syn + quote + compile_error! + proc-macro-error. Pipeline : tokens → AST → validation → code forward + backward. Opérations autorisées : +=, ^=, swap. Rejetées : =, mem::forget, I/O. Overhead compile : 2.5-5.5%. Faisabilité : OUI pour sous-ensemble, incrémental v0.3→v1.0.
- **Défi 4 (Garbage-Free Collector)** : Vec<BitVector> comme pile miroir LIFO. Checkpoint/uncompute/verify_garbage_free. Stratégie SQUARE pour optimisation. Budget mémoire configurable. Faisabilité : OUI, basique trivial.
- **Défi 5 (Algorithme Bennett)** : Modélisation pebbling game sur DAG (crate petgraph). Le plus complexe. Implémentation naïve faisable, optimisation = recherche. Faisabilité : OUI (naïf), phase v0.4.
- **Défi 6 (Pin/Intégrité mémoire)** : Arena allocator + indices > Pin<Box<T>>. Simplification architecturale. typed-arena ou bumpalo. Faisabilité : OUI, basse complexité.

## Faisabilité Technique — Stack Recommandé

- Types linéaires : QuantumCell<T> custom (code propre)
- Portes : Rust natif + std::simd (nightly) ou pulp (stable)
- DSL : syn, quote, proc-macro2 (toutes matures)
- GC : Vec<BitVector> (code propre)
- Bennett : petgraph (mature)
- Allocation : typed-arena ou bumpalo (mature)
- VM : match dispatch (quasi-optimal sur CPU modernes, pas besoin de computed goto)
- Tests : proptest (property-based, propriété clé : ∀x: undo(execute(x)) == x)
- Benchmarks : criterion

## Brainstorming — Idées Rejetées (pour ne pas les re-proposer)

- **#85 Prouver P≠NP via RC** : probablement faux, trop spéculatif pour le brief
- **#50 Licence "Reversible Commons"** : trop restrictif, contredit la stratégie Apache 2.0/MIT
- **#58 NFT de calculs réversibles** : distracteur, pas aligné avec la communauté cible
- **#98 Ne pas lancer de produit (spec seulement)** : trop passif, la communauté attend du code

## Brainstorming — Idées Retenues Non Incluses dans le Brief (pour la PRD)

- **#12 Playground Web WASM** : prototype navigateur pour onboarding <2 min
- **#39 Aion Challenge hebdomadaire** : gamification communautaire
- **#41 Bot GitHub PR reviews** : adoption passive, l'outil vient à vous
- **#42 "Entropy Wars" jeu éducatif** : viralité + pédagogie
- **#45 FPGA Backend** : hardware RC sans attendre Vaire
- **#62 Code hybride réversible/conventionnel** : #[entropy_allowed] pour adoption graduelle
- **#63 "Carbon offset" pour le code** : compensation entropique
- **#65 Sonification entropie** : feedback multi-sensoriel
- **#74 Aion Academy** : cours 10 leçons + exercices proptest interactifs
- **#88 Reversibility Index** : nouvelle métrique ISO potentielle

## Contraintes & Questions Ouvertes

- **Naming "Rewind"** : à valider — vérifier disponibilité crate crates.io + domaine
- **rr différenciation** : le pitch doit être très clair sur pourquoi Rewind ≠ rr. Clé : compile-time verification + multi-usage (pas que debugging)
- **Deadline RC 2026** : vérifier deadline de soumission (conférence 9-10 juillet)
- **Ressources** : le brief assume un développeur solo ou petite équipe — budget/temps non spécifié par l'utilisateur
- **Types affines vs linéaires** : le pattern Drop+panic est un workaround, pas une solution parfaite. mem::forget reste une faille théorique. À documenter honnêtement.
- **Overhead simulation CPU** : le RC simulé sur CPU conventionnel SERA plus lent que le code natif. Il faut communiquer clairement que le gain est en correctness/debugging, pas en performance (sur CPU conventionnel)

## Réglementaire Favorable

- IP : Toffoli (1980), Bennett (1973), Landauer (1961), Girard (1987) — tout en domaine public
- Licence : Apache 2.0 / MIT (standard Rust)
- EU EED : Data Centre Package Q1 2026, neutralité carbone 2030
- ISO/IEC 21031:2024 (SCI) : score carbone logiciel — Rewind peut revendiquer un score minimal
- Export controls : pas applicables au logiciel pur
- RGPD : le RC peut prouver la destruction d'information (use case #56)
