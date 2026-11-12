# Plugin_DefaultPipelines

The render pipelines the engine ships, as render-graph assets.

`pulsar.default_pipelines` ("Default Pipelines") is a plugin of
[Pulsar-Native](https://github.com/Far-Beyond-Pulsar/Pulsar-Native), the engine and editor that renders with
[Helio](https://github.com/Far-Beyond-Pulsar/Helio). Pulsar-Native vendors this repository as a git submodule
at `plugins/vendor/default_pipelines`.

## What it owns

Helio registers no pipeline and has no default: it knows the frame (its slots, draw phases
and lighting-model points) and turns a preset into a graph asset. This plugin decides which
pipelines ship and serves each as a read-only file in every project's
`plugins/pulsar.default_pipelines/` folder:

| File | What |
|---|---|
| `Deferred.rendergraph` | The deferred pipeline; TSR when the project enables it, FXAA otherwise |
| `DeferredFxaa.rendergraph` | Deferred lighting with FXAA |
| `Forward.rendergraph` | Forward-lit opaque geometry, no G-buffer |
| `<label>.rendergraph` | Each pipeline a registered lighting model brings (the lights' HLFS) |

- **Made for the project's plugins.** Every kind's passes join a pipeline where its feature
  placed them, so a project that disables a kind gets files without its nodes, and a new
  plugin's passes join without a change here. The files are made again when the plugin set
  changes.
- **Readable.** Once the engine has probed the nodes' pins, each file is the wired form: every
  pass wired to what supplies its inputs, laid out in lanes and sections, with the pins it was
  wired from recorded so a game validates it without probing. Open one in the render graph
  editor to see how the pipeline works.
- **Make your own.** The files are read-only. The render graph editor's "Save a copy to the
  project" writes a copy into the project's `render_graphs/`; pick it in World Settings and
  the level runs it.
- **Old levels.** Levels that name Helio's old built-in ids (`helio.deferred`, `helio.fxaa`,
  ...) run the file that replaced them (`PipelineIdMigration`).

| Crate | Path | What |
|---|---|---|
| `default_pipelines_plugin` | `./` (`src/`) | The plugin's root crate: the folder (`engine_fs::FileProvider`) and the old-id migration |

## Building

The crate is a member of the Pulsar-Native workspace (`workspace = true` dependencies, Helio
from Pulsar's submodule), so it builds only inside a Pulsar-Native checkout. CI checks out
Pulsar-Native (the branch of the same name when there is one), puts this commit at
`plugins/vendor/default_pipelines` and runs rustfmt, clippy and the tests.
