//! Default Pipelines (`pulsar.default_pipelines`): the render pipelines the
//! engine ships, as `.rendergraph` assets in the plugin's folder
//! (`plugins/pulsar.default_pipelines/`).
//!
//! Helio registers no pipeline and has no default: it knows the frame (the
//! slots, draw phases and lighting model its presets are made of) and turns a
//! preset into a graph asset (`helio_presets::graphs::graph_asset`). This
//! plugin decides which pipelines ship and serves each as a file:
//!
//! | File | Preset |
//! |---|---|
//! | `Deferred.rendergraph` | the deferred model; TSR when the project enables it, FXAA otherwise |
//! | `DeferredFxaa.rendergraph` | the deferred model with FXAA |
//! | `Forward.rendergraph` | the forward model, no G-buffer |
//! | `<label>.rendergraph` | each pipeline a registered lighting model brings (the lights' HLFS) |
//!
//! **Made, not authored.** A kind's passes join a preset where its feature
//! placed them (the frame's slots, draw phases and lighting-model points), so
//! each file is every enabled plugin's passes in their place: a project that
//! disables a kind gets files without its nodes, and a plugin this repository
//! has never heard of gets its nodes into the shipped pipelines by placing
//! them. A hand-authored file could drop an absent kind's nodes but never
//! pick up a new one's, and would be a second copy of the frame's structure,
//! which the spine's presets own. The folder is made again when the plugin set
//! changes (`pulsar_plugins::generation`), and a changed file is reported
//! through the folder's capability, so the engine registers it again.
//! Anti-aliasing is not baked in: `Deferred` names the anti-aliasing node,
//! which follows the project's TSR setting when the graph builds.
//!
//! **Wired once the pins are known.** With the pins the engine's probe found
//! (`pipelines::current_node_registry`), a file is the readable form: every
//! pass wired to what supplies its inputs, laid out in lanes and sections, and
//! the pins it was wired from recorded in it (`GraphAsset::pins`), so it
//! validates and builds where nothing has probed (a packaged game). Before the
//! probe has run it is the chain form (every pass on the execution chain), which
//! builds the same pipeline; the render graph editor waits for the probe
//! before it reads a file, so it always shows the wired form. The swap from one
//! form to the other rebuilds no renderer: the engine rebuilds only when what a
//! graph builds changes (`Validation::build_key`).
//!
//! The files are read-only, like everything in a plugin folder: a project
//! makes its own pipeline by saving a copy into its assets (the render graph
//! editor's "Save a copy to the project") and picking that in World Settings.
//!
//! **Old ids.** Levels saved before the pipelines were assets name Helio's
//! old built-in ids (`helio.deferred`, `helio.fxaa`, the lighting models'
//! `helio.hlfs`, ...); the plugin's [`PipelineIdMigration`] rewrites them to
//! the files' paths when a level loads.

pulsar_plugins::installed!("pulsar-plugin.toml", build);

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use engine_backend::subsystems::render::helio_renderer::pipelines::{
    self, PipelineIdMigration, RENDER_GRAPH_EXTENSION,
};
use engine_fs::{FolderSource, FsEntry, FsMetadata, ManifestEntry, MemoryFolder, PluginFolder};
use helio_graph_asset::NodeRegistry;
use helio_presets::Preset;
use parking_lot::Mutex;

/// The plugin's id, its folder's name.
pub const PLUGIN_ID: &str = "pulsar.default_pipelines";

/// One pipeline the plugin ships: its file's stem, what it is made from and
/// the old built-in ids it replaces.
pub struct Shipped {
    pub stem: String,
    pub preset: Preset,
    pub description: String,
    pub replaces: Vec<String>,
}

/// The pipelines the plugin ships for the nodes `registry` holds: Helio's
/// presets, then each pipeline a registered lighting model brings.
pub fn shipped(registry: &NodeRegistry) -> Vec<Shipped> {
    let mut list = vec![
        Shipped {
            stem: "Deferred".into(),
            preset: Preset::Deferred,
            description: "Full deferred pipeline: shadows, GI, sky, fog, post-processing; TSR when the project enables it, FXAA otherwise.".into(),
            // `helio.deferred` was the engine's default; `helio.deferred_graph`
            // was the same nodes as data.
            replaces: vec!["helio.deferred".into(), "helio.deferred_graph".into()],
        },
        Shipped {
            stem: "DeferredFxaa".into(),
            preset: Preset::DeferredFxaa,
            description: "Deferred lighting with FXAA instead of temporal resolve.".into(),
            replaces: vec!["helio.fxaa".into()],
        },
        Shipped {
            stem: "Forward".into(),
            preset: Preset::Forward,
            description: "Forward-lit opaque geometry, no G-buffer.".into(),
            replaces: vec!["helio.forward_opaque".into(), "helio.forward_only".into()],
        },
    ];
    for model in helio_frame_api::lighting::registered_models(registry) {
        for pipeline in &model.pipelines {
            list.push(Shipped {
                stem: file_stem(pipeline.label),
                preset: Preset::Model(*pipeline),
                description: pipeline.description.to_string(),
                replaces: vec![pipeline.id.to_string()],
            });
        }
    }
    list
}

/// A file stem from a label: letters and digits kept, every other run made
/// one `_` (`HLFS + FXAA` is `HLFS_FXAA`).
fn file_stem(label: &str) -> String {
    let mut stem = String::new();
    for c in label.chars() {
        if c.is_ascii_alphanumeric() {
            stem.push(c);
        } else if !stem.ends_with('_') {
            stem.push('_');
        }
    }
    stem.trim_matches('_').to_owned()
}

/// The file name of `stem` in the folder.
fn file_name(stem: &str) -> String {
    format!("{stem}.{RENDER_GRAPH_EXTENSION}")
}

/// The project-relative path (the pipeline id a level stores) of the file
/// `stem`.
pub fn asset_path(stem: &str) -> String {
    format!(
        "{}/{PLUGIN_ID}/{}",
        engine_fs::virtual_fs::PLUGINS_FOLDER,
        file_name(stem)
    )
}

/// The node registry the files are made from: the engine's (Helio's nodes and
/// every enabled plugin's), with the probed pins once the probe has run.
fn node_registry() -> NodeRegistry {
    pipelines::current_node_registry()
}

/// Every file of the folder for `registry`, by name.
pub fn files(registry: &NodeRegistry) -> BTreeMap<String, Vec<u8>> {
    shipped(registry)
        .into_iter()
        .map(|shipped| {
            let mut asset = helio_presets::graphs::graph_asset(shipped.preset, registry);
            asset.name = shipped.stem.clone();
            asset.description = shipped.description;
            (
                file_name(&shipped.stem),
                (asset.to_json() + "\n").into_bytes(),
            )
        })
        .collect()
}

/// The plugin's folder: the files for the enabled plugins, made again when
/// the plugin set changes or the probe finds the pins.
struct Pipelines {
    files: MemoryFolder,
    /// The plugin generation the files were made for, and whether they were
    /// wired (made with probed pins).
    made_for: Mutex<Option<(u64, bool)>>,
}

impl Pipelines {
    /// Makes the files again if the plugin set changed since they were made,
    /// or the probe has found the pins since; [`MemoryFolder`] reports each
    /// file that changed.
    fn refresh(&self) {
        let generation = pulsar_plugins::generation();
        let mut made_for = self.made_for.lock();
        if *made_for == Some((generation, true)) {
            return;
        }
        let registry = node_registry();
        let wired = registry.has_generated_pins();
        if *made_for == Some((generation, wired)) {
            return;
        }
        *made_for = Some((generation, wired));
        let fresh = files(&registry);
        let root = Path::new("");
        let stale: Vec<String> = self
            .files
            .list_dir(root, "")
            .map(|entries| entries.into_iter().map(|entry| entry.name).collect())
            .unwrap_or_default();
        drop(made_for);
        for name in stale {
            if !fresh.contains_key(&name) {
                self.files.delete(&name);
            }
        }
        for (name, bytes) in fresh {
            if self.files.read_file(root, &name).ok().as_deref() != Some(bytes.as_slice()) {
                self.files.write(&name, bytes);
            }
        }
    }
}

impl FolderSource for Pipelines {
    fn read_file(&self, project: &Path, path: &str) -> anyhow::Result<Vec<u8>> {
        self.refresh();
        self.files.read_file(project, path)
    }

    fn list_dir(&self, project: &Path, path: &str) -> anyhow::Result<Vec<FsEntry>> {
        self.refresh();
        self.files.list_dir(project, path)
    }

    fn metadata(&self, project: &Path, path: &str) -> Option<FsMetadata> {
        self.refresh();
        self.files.metadata(project, path)
    }

    fn manifest(&self, project: &Path, path: &str) -> Vec<ManifestEntry> {
        self.refresh();
        self.files.manifest(project, path)
    }

    fn project_independent(&self) -> bool {
        true
    }
}

fn open(folder: PluginFolder) -> Arc<dyn FolderSource> {
    Arc::new(Pipelines {
        files: MemoryFolder::new(folder),
        made_for: Mutex::new(None),
    })
}

/// The file an old built-in pipeline id now is, for the enabled plugins.
fn migrate(id: &str) -> Option<String> {
    shipped(&pipelines::current_node_registry())
        .into_iter()
        .find(|shipped| shipped.replaces.iter().any(|old| old == id))
        .map(|shipped| asset_path(&shipped.stem))
}

/// Everything the plugin registers.
pub fn build(p: &mut pulsar_plugins::PluginBuilder) {
    // The folder of shipped pipelines. A packaged game serves the packager's
    // snapshot of it (made for the project's plugins).
    p.add(engine_fs::FileProvider {
        origin: p.origin(),
        open,
        in_game: engine_fs::InGame::Snapshot,
    });
    // Levels that name an old built-in id run the file that replaced it.
    p.add(PipelineIdMigration {
        origin: p.origin(),
        migrate,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems_are_file_safe() {
        assert_eq!(file_stem("HLFS + FXAA"), "HLFS_FXAA");
        assert_eq!(file_stem("HLFS"), "HLFS");
        assert_eq!(
            asset_path("Deferred"),
            "plugins/pulsar.default_pipelines/Deferred.rendergraph"
        );
    }
}
