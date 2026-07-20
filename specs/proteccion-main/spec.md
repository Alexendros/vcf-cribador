# Protección de rama main + resolución de CI

**Feature:** `proteccion-main`
**Versión spec:** 1.0.0
**Fecha:** 2026-07-20

---

## Objetivo

Proteger la rama `main` con reglas de branch protection, resolver los pipelines CI de los 2 PRs
dependabot pendientes (#8 y #12), mergearlos con éxito, y actualizar la documentación del proyecto
(tasks, roadmap, architecture, README) para reflejar el estado post-merge.

## Requerimientos funcionales

| ID    | Descripción                                                                 |
|-------|-----------------------------------------------------------------------------|
| RF-1  | La rama `main` debe tener branch protection: requiere CI passing, requiere reviews, prohíbe push directo |
| RF-2  | El PR #12 (dependabot/cargo) debe pasar CI completa                         |
| RF-3  | El PR #8 (dependabot/github_actions) debe pasar CI completa                |
| RF-4  | Ambos PRs deben mergearse a `main` tras CI verde                           |
| RF-5  | `dependabot.yml` debe excluir bumps del action `dtolnay/rust-toolchain` que cambien MSRV |
| RF-6  | README debe incluir badges de CI, tareas pendientes y enlaces a docs       |
| RF-7  | `docs/tasks.md` debe listar tareas conocidas y hoja de ruta                |
| RF-8  | `docs/architecture.md` debe reflejar contexto de CI/deployment             |
| RF-9  | Advertencia de doc (unclosed HTML tag en parser.rs) debe corregirse        |

## Criterios de aceptación

1. `make ci` (fmt + clippy + test + doc) pasa en local con código limpio
2. GitHub Actions muestra CI verde para ambos PRs tras rebase
3. `main` tiene branch protection rules activas vía GitHub API
4. `docs/tasks.md` existe con tareas y roadmap
5. README refleja estado actual del proyecto
6. `dependabot.yml` no genera PRs que suban el MSRV
