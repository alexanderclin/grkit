//! Browser entry points that assemble a GRKit SD package.
//!
//! The site serves a runtime build as CI publishes it: `build.json` and the files under `sd/`.
//! The page fetches those files and the selected effects, then builds the ZIP locally.
//! `Builder` holds the logic so that native tests cover it. `Package` only adapts it for JS.

mod gamma;

use anyhow::{ensure, Context, Result};
use grkit_core::{
    camera::{self, Camera},
    effects, fit,
    modules::Selection,
    packaging, runtime, Files,
};
use wasm_bindgen::prelude::*;

/// The largest `.cube` file `import_lut` accepts. A 65-point cube is about
/// 10 MB of text; the bound keeps a stray file from exhausting the tab.
const MAX_CUBE_BYTES: usize = 64 * 1024 * 1024;

/// The outcome of fitting one uploaded LUT: its effect ID and the fit's
/// CIEDE2000 errors on the ColorChecker, as the CLI import reports them.
#[derive(Clone, Debug, PartialEq)]
pub struct LutFit {
    pub id: String,
    pub mean_de: f64,
    pub max_de: f64,
    pub standard_mean_de: f64,
}

impl LutFit {
    fn to_json(&self) -> String {
        serde_json::json!({
            "id": self.id,
            "mean_de": self.mean_de,
            "max_de": self.max_de,
            "standard_mean_de": self.standard_mean_de,
        })
        .to_string()
    }
}

/// Collects one runtime build and a set of compiled effects, then assembles the package.
pub struct Builder {
    manifest: Vec<u8>,
    target: String,
    paths: Vec<String>,
    runtime: Files,
    effects: Files,
    plugins: Selection,
    camera: &'static Camera,
    language: packaging::Language,
    guide: Option<Vec<u8>>,
}

impl Builder {
    /// Starts a package from the runtime's `build.json`.
    pub fn new(manifest: &[u8]) -> Result<Self> {
        let parsed = packaging::read_manifest(manifest)?;
        let target = parsed["target"].as_str().unwrap_or_default().to_owned();
        let paths = parsed["files"]
            .as_array()
            .context("Expected a manifest file list")?
            .iter()
            .map(|item| {
                item["path"]
                    .as_str()
                    .map(str::to_owned)
                    .context("Expected a runtime path")
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            manifest: manifest.to_vec(),
            target,
            paths,
            runtime: Files::new(),
            effects: Files::new(),
            plugins: Selection::ALL,
            camera: &camera::GR3,
            language: packaging::Language::English,
            guide: None,
        })
    }

    /// The runtime files to fetch, as paths under the build's `sd/` directory.
    pub fn runtime_paths(&self) -> &[String] {
        &self.paths
    }

    /// Adds one runtime file. `build` checks its size and SHA-256 against the manifest.
    pub fn add_runtime_file(&mut self, path: &str, bytes: &[u8]) -> Result<()> {
        ensure!(
            self.paths.iter().any(|known| known == path),
            "Not a runtime file of this build: {path}"
        );
        ensure!(
            !self.runtime.contains_key(path),
            "Duplicate runtime file: {path}"
        );
        self.runtime.insert(path.to_owned(), bytes.to_vec());
        Ok(())
    }

    /// Adds one compiled effect and returns its ID. The package filename comes from the ID.
    pub fn add_effect(&mut self, blob: &[u8]) -> Result<String> {
        let (effect, _) = effects::decode(blob)?;
        let id = effect["effect_id"]
            .as_str()
            .context("Expected an effect ID")?
            .to_owned();
        let name = effects::filename(&id)?;
        ensure!(
            !self.effects.contains_key(&name),
            "Duplicate effect ID or filename hash collision: {id}"
        );
        ensure!(
            self.effects.len() < runtime::MAX_COUNT,
            "This target supports at most {} effects",
            runtime::MAX_COUNT
        );
        self.effects.insert(name, blob.to_vec());
        Ok(id)
    }

    /// Fits one uploaded `.cube` LUT into a custom effect and adds it, the way
    /// `grkit effects import` does on the command line. `input` names the
    /// encoding the LUT expects: `srgb` (the usual case), `linear`, or `vlog`.
    /// `source` is the file name, recorded in the effect's attribution.
    /// The fit works against the Standard base gamma of the runtime's target,
    /// which ships with the packager (see `gamma`); other targets are refused.
    pub fn import_lut(
        &mut self,
        cube: &[u8],
        source: &str,
        id: &str,
        name: &str,
        label: &str,
        shape: &str,
        input: &str,
    ) -> Result<LutFit> {
        ensure!(
            cube.len() <= MAX_CUBE_BYTES,
            "LUT file is too large: {} bytes (limit {MAX_CUBE_BYTES})",
            cube.len()
        );
        ensure!(
            gamma::lut_target_supported(&self.target),
            "Custom LUTs are not available for the {} runtime yet",
            if self.target.is_empty() { "unknown" } else { &self.target }
        );
        ensure!(effects::valid_id(id), "Invalid effect ID: {id}");
        let request = fit::Import {
            effect_id: id,
            name,
            label,
            shape,
            source,
            input: input.parse()?,
            exposure: 0.0,
            options: fit::Options::default(),
        };
        let (fitted, text) = fit::import(cube, &gamma::STANDARD_BASE_GAMMA, &request)?;
        let definition = effects::parse_json(text.as_bytes())?;
        let blob = effects::compile_definition(&definition)?;
        let added = self.add_effect(&blob)?;
        ensure!(added == id, "Imported effect ID mismatch: {added} != {id}");
        Ok(LutFit {
            id: added,
            mean_de: fitted.mean_de,
            max_de: fitted.max_de,
            standard_mean_de: fitted.standard_mean_de,
        })
    }

    /// Removes one effect that `add_effect` or `import_lut` added.
    pub fn remove_effect(&mut self, id: &str) -> Result<()> {
        let name = effects::filename(id)?;
        ensure!(
            self.effects.remove(&name).is_some(),
            "No such effect in this package: {id}"
        );
        Ok(())
    }

    /// Leaves one plugin out of the package: `image-control`, `framing`, `guides`,
    /// `date-stamp`, or `grain`. The installer changes no firmware for it.
    /// Without effects, the installer leaves out Image Control by itself.
    pub fn omit_plugin(&mut self, name: &str) -> Result<()> {
        let plugins = self.plugins.without(name)?;
        ensure!(plugins != self.plugins, "Plugin omitted twice: {name}");
        self.plugins = plugins;
        Ok(())
    }

    /// Selects the camera: `gr3` (the default), `gr3x`, or `gr4`. The package gets its
    /// ModeSet file. `build` rejects a camera that does not run this runtime.
    pub fn set_camera(&mut self, id: &str) -> Result<()> {
        self.camera = camera::find(id)?;
        Ok(())
    }

    /// Selects the language of the installation guide: `en` (the default),
    /// `zh-Hans`, `zh-Hant`, or `ja`.
    pub fn set_language(&mut self, code: &str) -> Result<()> {
        self.language = packaging::Language::parse(code)?;
        Ok(())
    }

    /// Sets the PDF of the installation guide for the camera, in the language of
    /// `set_language`. The site publishes one for each camera and language.
    pub fn set_guide(&mut self, pdf: &[u8]) {
        self.guide = Some(pdf.to_vec());
    }

    /// Verifies the runtime against its manifest and returns the verified package ZIP.
    pub fn build(&self) -> Result<Vec<u8>> {
        let (runtime, manifest) = packaging::load_runtime(&self.manifest, &self.runtime)?;
        packaging::assemble(
            &runtime,
            &manifest,
            &self.effects,
            self.plugins,
            self.camera,
            &packaging::Guide {
                language: self.language,
                pdf: self
                    .guide
                    .as_deref()
                    .context("Set the installation guide first")?,
            },
        )
    }
}

/// Builds a GRKit SD package from a runtime build and compiled effects.
///
/// Each failing call throws an `Error` whose message has the full cause chain.
#[wasm_bindgen]
pub struct Package(Builder);

#[wasm_bindgen]
impl Package {
    /// Starts a package from the runtime's `build.json` bytes.
    #[wasm_bindgen(constructor)]
    pub fn new(manifest: &[u8]) -> Result<Package, JsError> {
        Builder::new(manifest).map(Package).map_err(js_error)
    }

    /// The runtime files to fetch, as paths under the build's `sd/` directory.
    #[wasm_bindgen(js_name = runtimePaths)]
    pub fn runtime_paths(&self) -> Vec<String> {
        self.0.runtime_paths().to_vec()
    }

    /// Adds one runtime file. `build()` checks its size and SHA-256.
    #[wasm_bindgen(js_name = addRuntimeFile)]
    pub fn add_runtime_file(&mut self, path: &str, bytes: &[u8]) -> Result<(), JsError> {
        self.0.add_runtime_file(path, bytes).map_err(js_error)
    }

    /// Adds one compiled `.GRF` effect and returns its effect ID.
    #[wasm_bindgen(js_name = addEffect)]
    pub fn add_effect(&mut self, blob: &[u8]) -> Result<String, JsError> {
        self.0.add_effect(blob).map_err(js_error)
    }

    /// Fits one uploaded `.cube` LUT into a custom effect and adds it.
    /// Returns a JSON object: the effect `id` and the fit's `mean_de`,
    /// `max_de`, and `standard_mean_de` color errors.
    #[wasm_bindgen(js_name = importLut)]
    pub fn import_lut(
        &mut self,
        cube: &[u8],
        source: &str,
        id: &str,
        name: &str,
        label: &str,
        shape: &str,
        input: &str,
    ) -> Result<String, JsError> {
        self.0
            .import_lut(cube, source, id, name, label, shape, input)
            .map(|fit| fit.to_json())
            .map_err(js_error)
    }

    /// Removes one effect that `addEffect` or `importLut` added.
    #[wasm_bindgen(js_name = removeEffect)]
    pub fn remove_effect(&mut self, id: &str) -> Result<(), JsError> {
        self.0.remove_effect(id).map_err(js_error)
    }

    /// Leaves one plugin out of the package: `image-control`, `framing`, `guides`,
    /// `date-stamp`, or `grain`.
    #[wasm_bindgen(js_name = omitPlugin)]
    pub fn omit_plugin(&mut self, name: &str) -> Result<(), JsError> {
        self.0.omit_plugin(name).map_err(js_error)
    }

    /// Selects the camera: `gr3` (the default), `gr3x`, or `gr4`.
    #[wasm_bindgen(js_name = setCamera)]
    pub fn set_camera(&mut self, id: &str) -> Result<(), JsError> {
        self.0.set_camera(id).map_err(js_error)
    }

    /// Selects the language of the installation guide: `en`, `zh-Hans`, `zh-Hant`, or `ja`.
    #[wasm_bindgen(js_name = setLanguage)]
    pub fn set_language(&mut self, code: &str) -> Result<(), JsError> {
        self.0.set_language(code).map_err(js_error)
    }

    /// Sets the PDF of the installation guide for the camera, in the selected language.
    #[wasm_bindgen(js_name = setGuide)]
    pub fn set_guide(&mut self, pdf: &[u8]) {
        self.0.set_guide(pdf);
    }

    /// Returns the verified package as ZIP bytes.
    pub fn build(&self) -> Result<Vec<u8>, JsError> {
        self.0.build().map_err(js_error)
    }
}

/// Returns the ZIP that turns SD scripts off on a camera: `gr3`, `gr3x`, or `gr4`.
#[wasm_bindgen(js_name = scriptsOffPackage)]
pub fn scripts_off_package(camera: &str) -> Result<Vec<u8>, JsError> {
    grkit_core::camera::find(camera)
        .and_then(grkit_core::packaging::assemble_scripts_off)
        .map_err(js_error)
}

fn js_error(error: anyhow::Error) -> JsError {
    JsError::new(&format!("{error:#}"))
}
