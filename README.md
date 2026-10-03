# Tokn Observer

[![CI](https://github.com/ELITEAYOTO/Tokn/actions/workflows/ci.yml/badge.svg)](https://github.com/ELITEAYOTO/Tokn/actions/workflows/ci.yml)

Tokn est un moteur **local-first** d'observabilite et d'analyse pour les coding agents.
Il cherche a augmenter le travail utile et la qualite par token sans brider le modele.
La North Star long terme est : **meme ou meilleure qualite pour un cout agent total plus faible**.

Codex/Astra est le premier runtime valide, pas une dependance du domaine Tokn.
Le Core analytique est concu pour rester provider-neutral ; les details runtime/provider appartiennent a des adapters minces.

Version binaire actuelle : **V0.1** (workspace Cargo `0.1.0`).

**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract V1 FROZEN ; Store foundation DONE ; Local MCP transport prototype ACCEPTED ; M4 Historical Analyzer + Context Ledger CORE ACCEPTED / EVIDENCE-GATED EXTENSIONS ; M4.5 Context Identity + Shadow Foundations STARTED / OBSERVATION-ONLY.**

## Ce que Tokn fait aujourd'hui

Tokn peut actuellement :
- detecter et lire des preuves Codex locales sans modifier les originaux ;
- reconstruire runs, agents, usage tokens, outils, qualite et provenance runtime ;
- persister un historique privacy-safe dans SQLite ;
- comparer des operations/resultats seulement quand leur identite est prouvee ;
- suivre SourceStableId, versions BEFORE/AFTER et provenance Git workspace ;
- exposer des vues read-only via CLI et un prototype MCP local stdio.

Tokn reste **observation-first** : aucune economie causale n'est revendiquee sans experience valide.

## Plateformes

| Plateforme | Etat |
| --- | --- |
| Windows 10/11 x64 + MSVC | **Valide aujourd'hui** |
| Linux | Architecture cible, pas encore validee |
| macOS | Architecture cible, pas encore validee |

Le CI officiel tourne sur `windows-latest` et le toolchain courant cible `x86_64-pc-windows-msvc`.
Ne pas interpreter l'architecture multi-runtime comme une promesse multi-OS deja validee.

## Quickstart developpeur

Prerequis :
- Git ;
- Rustup ;
- Visual Studio / Build Tools avec le toolchain C++ MSVC.

Depuis PowerShell :

```powershell
git clone https://github.com/ELITEAYOTO/Tokn.git
cd Tokn
.\scripts\dev-check.ps1
.\scripts\build-release.ps1
.\target\release\tokn-observe.exe doctor
.\target\release\tokn-observe.exe sessions --limit 5
```

`rust-toolchain.toml` selectionne automatiquement Rust 1.97.1 avec rustfmt et Clippy.

## Exemple de sortie

Exemple synthetique de `doctor` :

```text
TOKN OBSERVER 0.1.0

PLATFORM
  OS              windows
  architecture    x86_64

CODEX
  surface         Desktop
  version         codex-cli <version observee>

SESSIONS
  root            <chemin local>
```

Les chemins et versions affiches viennent de la machine locale ; ne pas publier de sorties reelles contenant des informations sensibles.

## Commandes utiles

Quelques surfaces read-only courantes :

```powershell
.\target\release\tokn-observe.exe doctor
.\target\release\tokn-observe.exe sessions --limit 20
.\target\release\tokn-observe.exe context-ledger --help
.\target\release\tokn-observe.exe activity-timeline --help
.\target\release\tokn-observe.exe cross-agent-evidence --help
.\target\release\tokn-observe.exe source-version-history --help
.\target\release\tokn-observe.exe workspace-git-provenance-history --help
```

## Principes non negociables

- `UNKNOWN` reste `UNKNOWN` ;
- `NO_EVIDENCE != PASS` ;
- parent seul != run complet si des sous-agents existent ;
- output workspace != input workspace tant que ce n'est pas prouve ;
- aucune economie causale sans experience valide ;
- aucune reduction de contexte n'est un gain si la qualite baisse ;
- les semantiques tokens OpenAI ne deviennent jamais un contrat universel par defaut ;
- aucune sortie outil brute n'est persistee durablement uniquement pour fabriquer une identite.

## Ce que Tokn ne revendique pas encore

Tokn ne prouve pas encore :
- un pourcentage de tokens economises par une intervention active ;
- une freshness/staleness generale du contexte ;
- une causalite entre une mutation outil et un changement fichier sans preuve directe ;
- un support Linux/macOS valide ;
- un support Claude/OpenCode actif.

Le premier A/B causal d'optimisation reste **Experiment 003**, apres finding reproductible, Opportunity Analyzer et protocole qualite/variance predeclare.

## Developpement

Check rapide avant commit/PR :

```powershell
.\scripts\dev-check.ps1
```

Gate complet local avant PR :

```powershell
.\scripts\dev-check.ps1 -Full
```

Le workflow GitHub reste le gate final. Voir `docs/operations/DEVELOPMENT-WORKFLOW.md`.

## Benchmark

Le protocole baseline descriptif est documente dans `docs/benchmarks/BASELINE-PROTOCOL-V1.md`.
Il sert a mesurer variance, decomposition des couts et surface adressable **avant** toute optimisation active.
Il ne remplace pas Experiment 003 et ne doit pas etre presente comme une preuve de gain Tokn.

## Architecture et documentation

Point d'entree unique : `docs/INDEX.md`.

Pour reprendre le developpement :
1. `STATUS.md` - etat reel ;
2. `ROADMAP.md` - ordre et gates ;
3. `docs/plans/IMPLEMENTATION-PATH.md` - chemin d'execution courant ;
4. `docs/plans/NEXT-SESSION-CHECKLIST.md` - prochaine action concrete ;
5. `docs/design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md` - contraintes architecturales ;
6. `docs/operations/DEVELOPMENT-WORKFLOW.md` - workflow Git/CI local.

Les plans/tests V0.1 restent des references historiques/validation. Ne pas relire tous les documents historiques pour reprendre le travail.

## Securite et confidentialite

Tokn traite des preuves locales potentiellement sensibles. Ne jamais publier de rollout/trace reel.
Les packages sont construits depuis des fichiers suivis par Git + une whitelist explicite, puis rescannes avant publication.
Voir `SECURITY.md`.

## Licence

Tokn est distribue sous double licence **MIT OR Apache-2.0**.
Voir `LICENSE-MIT` et `LICENSE-APACHE`.
