# Plugin_DefaultPipelines

`pulsar.default_pipelines`: the render pipelines the engine ships, as `.rendergraph` assets in
the plugin's folder (`plugins/pulsar.default_pipelines/`), made from Helio's presets for the
plugins a project enables.
Pulsar-Native vendors this repository at `plugins/vendor/default_pipelines`; it builds only
inside a Pulsar-Native checkout (see `README.md`).

## Required for every agent

Before reading, designing, editing, reviewing or briefing any change here,
load the **`helio-architecture`** skill. Its single source is Helio's
[`.agents/skills/helio-architecture/SKILL.md`](https://github.com/Far-Beyond-Pulsar/Helio/blob/main/.agents/skills/helio-architecture/SKILL.md); in a Pulsar-Native
checkout it is at `crates/renderer/helio/.agents/skills/helio-architecture/SKILL.md`.
That applies to bug fixes, perf work, tests and small changes too.

Follow the skill's steps in order and end with its report section (Design,
AAA comparison, Deleted, Verified). Anything you brief (a subagent, another
session or another agent) must load the skill too. The skill is the single
source of these rules: this file only points to it, so change the skill
rather than copying rules here.

## Docs

| File | What |
|---|---|
| Pulsar-Native `.agents/RENDER_GRAPH.md` | Render pipelines, `.rendergraph` assets, the shipped pipelines, the render graph editor |
| [Helio `docs/architecture/plugin-migration.md`](https://github.com/Far-Beyond-Pulsar/Helio/blob/main/docs/architecture/plugin-migration.md) | Every object kind is a plugin; "As built, p23" for this plugin |
| Pulsar-Native `AGENTS.md`, `.agents/PLUGIN_SYSTEM.md`, `.agents/CRATES.md` | Plugin descriptors, the installed set, submodule pins |

## Key commands (from the Pulsar-Native root)

```
cargo check -p default_pipelines_plugin --tests
cargo test -p default_pipelines_plugin
cargo test -p installed_plugins --test default_pipelines   the files as the editor and a game read them (GPU)
cargo test -p one_owner          every concept has one owner (scans this plugin)
```
