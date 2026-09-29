# Tokn V0.1 - Next Session Checklist

Derniere mise a jour : 2026-09-29 19:01 +02:00

## But

Demarrer directement le developpement V0.1 sans refaire l'enquete Experiment 001.

## Avant de coder

Lire dans cet ordre :
1. ../INDEX.md
2. ../../STATUS.md
3. ../../ROADMAP.md
4. ../experiments/001-POSTMORTEM.md
5. V0.1-IMPLEMENTATION-PLAN.md
6. V0.1-TEST-MATRIX.md

Verifier :
- workspace : E:\Tokn\V0-CodexTkn-Consume\tool
- branche/worktree attendu si Git est ajoute plus tard ;
- aucun run experimental actif ;
- aucune trace source n'est modifiee.

## Premiere action de code

Commencer par **P6 - Policy placement and evidence**.

P0-P5 sont deja implementes et valides. Ne pas les refaire sauf regression.

Ne pas commencer par :
- Experiment 002 ;
- RAG ;
- embeddings ;
- GUI ;
- policy enforcement ;
- optimisation multi-agent.

## P6 sequence

1. Identifier le workspace Codex reel utilise au debut du run.
2. Definir ou un policy_hint peut etre place sans ambiguite.
3. Preserver/restaurer chaque AGENTS.md touche independamment.
4. Capturer si la policy apparait dans les instructions/session evidence.
5. Capturer si le fichier de policy est explicitement lu.
6. Separer policy_hint, policy_observed et policy_enforced.
7. Prober un hook runtime supporte pour enforcement des caps ; sinon documenter hard enforcement = unavailable.

## P6 gate

Avant P7 :
- cargo fmt --check ;
- clippy -D warnings ;
- cargo test workspace ;
- aucune policy soft ne doit etre declaree enforcee ;
- policy_hint / policy_observed / policy_enforced doivent etre distincts ;
- P0-P5 golden replay doit rester PASS.

## Ensuite

P7 experiment validity reducer.

Golden facts a conserver :
- 4 sessions ;
- 80 usage records ;
- logical total 5,311,758 ;
- uncached 226,025 ;
- terminal INCOMPLETE_USAGE_LIMIT ;
- diagnostic PARTIAL ;
- cap policy Experiment 001 = FAIL, jamais PASS ;
- workspace resolver = B07-C_WORKING\PROJECT ;
- RunGroup JSON = 1 parent + 3 sous-agents, source_path exacts ;
- START dry-run + cleanup = PASS ;
- release build courant = PASS.

## Regle

Aucun nouveau run Astra n'est necessaire avant la fin de P0-P9.
Tout V0.1 doit d'abord etre valide sur preuves existantes et fixtures.
