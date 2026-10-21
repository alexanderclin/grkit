use grkit_core::{effects, packaging, runtime, Files};
use grkit_wasm::Builder;
use serde_json::{json, Value};

fn runtime_files() -> Files {
    let mut binary = vec![0; 52];
    binary[..6].copy_from_slice(b"\x7fELF\x01\x01");
    binary[16..20].copy_from_slice(b"\x02\x00\x28\x00");
    runtime::runtime_files(&binary)
}

fn manifest(files: &Files) -> Vec<u8> {
    let value = json!({"profile": "production", "target": "gr3-2.10", "startup_activation": "Off",
        "files": files.iter().map(|(path, bytes)| json!({"path": path, "size": bytes.len(),
            "sha256": packaging::sha256(bytes)})).collect::<Vec<_>>()});
    packaging::json_bytes(&value).unwrap()
}

fn effect(id: &str) -> Vec<u8> {
    let curve: Vec<_> = (0..256).map(|i| i * 64).collect();
    let value = json!({"effect_id": id, "name": "Test", "flags": 0,
        "matrix": [8192, 0, 0, 0, 8192, 0, 0, 0, 8192],
        "curves": [&curve, &curve, &curve]});
    effects::encode(&value, &vec![0; 6400]).unwrap()
}

/// A stand-in for a guide PDF, which the site fetches.
const PDF: &[u8] = b"%PDF-1.7\n% GRKit test guide\n";

fn builder_with_runtime(files: &Files) -> Builder {
    let mut builder = Builder::new(&manifest(files)).unwrap();
    for path in builder.runtime_paths().to_vec() {
        builder.add_runtime_file(&path, &files[&path]).unwrap();
    }
    builder.set_guide(PDF);
    builder
}

fn error(result: anyhow::Result<impl Sized>) -> String {
    format!("{:#}", result.err().expect("expected an error"))
}

#[test]
fn builds_a_verified_package_from_a_published_runtime() {
    let files = runtime_files();
    let mut builder = builder_with_runtime(&files);
    let paths: Vec<_> = files.keys().cloned().collect();
    assert_eq!(builder.runtime_paths(), paths.as_slice());
    assert_eq!(builder.add_effect(&effect("test-a")).unwrap(), "test-a");
    assert_eq!(builder.add_effect(&effect("test-b")).unwrap(), "test-b");

    let package = builder.build().unwrap();
    assert_eq!(package, builder.build().unwrap());
    let verified: Value = packaging::verify_package(&package).unwrap();
    assert_eq!(verified["effect_ids"], json!(["test-a", "test-b"]));
}

#[test]
fn a_package_without_effects_is_valid() {
    let package = builder_with_runtime(&runtime_files()).build().unwrap();
    assert_eq!(
        packaging::verify_package(&package).unwrap()["effect_ids"],
        json!([])
    );
}

#[test]
fn omitted_plugins_are_written_and_checked() {
    let mut builder = builder_with_runtime(&runtime_files());
    builder.omit_plugin("framing").unwrap();
    assert!(error(builder.omit_plugin("framing")).contains("Plugin omitted twice"));
    assert!(error(builder.omit_plugin("hdr")).contains("Unknown plugin"));
    builder.omit_plugin("guides").unwrap();
    let package = builder.build().unwrap();
    assert_eq!(
        packaging::verify_package(&package).unwrap()["plugins"],
        json!(["image-control", "date-stamp", "grain", "diffusion"])
    );
    // Without effects, omitting every other plugin leaves nothing to show.
    builder.omit_plugin("date-stamp").unwrap();
    builder.build().unwrap();
    builder.omit_plugin("diffusion").unwrap();
    builder.omit_plugin("grain").unwrap();
    assert!(error(builder.build()).contains("shows no plugin"));
    builder.add_effect(&effect("test-a")).unwrap();
    builder.build().unwrap();
}

#[test]
fn the_camera_selects_the_modeset_file() {
    let mut builder = builder_with_runtime(&runtime_files());
    assert!(error(builder.set_camera("gr2")).contains("Unknown camera"));
    // GR IV needs its own runtime.
    builder.set_camera("gr4").unwrap();
    assert!(error(builder.build()).contains("GR IV runs the gr4-1.11 runtime"));
    builder.set_camera("gr3x").unwrap();
    let package = builder.build().unwrap();
    let verified = packaging::verify_package(&package).unwrap();
    assert_eq!(verified["camera"], "gr3x");
    builder.set_camera("gr3").unwrap();
    let package = builder.build().unwrap();
    assert_eq!(
        packaging::verify_package(&package).unwrap()["camera"],
        "gr3"
    );
}

#[test]
fn a_package_needs_the_guide_pdf() {
    let files = runtime_files();
    let mut builder = Builder::new(&manifest(&files)).unwrap();
    for path in builder.runtime_paths().to_vec() {
        builder.add_runtime_file(&path, &files[&path]).unwrap();
    }
    assert!(error(builder.build()).contains("Set the installation guide first"));
    builder.set_guide(b"not a PDF");
    assert!(error(builder.build()).contains("not a PDF document"));
    builder.set_guide(PDF);
    assert!(builder.build().is_ok());
}

#[test]
fn the_language_names_the_guide() {
    let mut builder = builder_with_runtime(&runtime_files());
    assert!(error(builder.set_language("fr")).contains("Unknown language"));
    builder.set_language("zh-Hant").unwrap();
    let package = builder.build().unwrap();
    assert_eq!(
        packaging::verify_package(&package).unwrap()["language"],
        "zh-Hant"
    );
    builder.set_language("en").unwrap();
    let package = builder.build().unwrap();
    assert_eq!(
        packaging::verify_package(&package).unwrap()["language"],
        "en"
    );
}

#[test]
fn missing_or_changed_runtime_files_are_rejected() {
    let files = runtime_files();
    let mut missing = Builder::new(&manifest(&files)).unwrap();
    missing
        .add_runtime_file("SCRIPT/STARTUP.TTL", &files["SCRIPT/STARTUP.TTL"])
        .unwrap();
    assert!(error(missing.build()).contains("Runtime directory differs from its manifest"));

    let mut changed = Builder::new(&manifest(&files)).unwrap();
    for path in changed.runtime_paths().to_vec() {
        let bytes = if path == "SCRIPT/STARTUP.TTL" {
            b"changed".to_vec()
        } else {
            files[&path].clone()
        };
        changed.add_runtime_file(&path, &bytes).unwrap();
    }
    assert!(error(changed.build()).contains("Runtime hash or size mismatch: SCRIPT/STARTUP.TTL"));
}

#[test]
fn unknown_and_duplicate_runtime_files_are_rejected() {
    let files = runtime_files();
    let mut builder = Builder::new(&manifest(&files)).unwrap();
    assert!(error(builder.add_runtime_file("SCRIPT/EXTRA.SH", b"x"))
        .contains("Not a runtime file of this build"));
    let path = "SCRIPT/STARTUP.TTL";
    builder.add_runtime_file(path, &files[path]).unwrap();
    assert!(error(builder.add_runtime_file(path, &files[path])).contains("Duplicate runtime file"));
}

#[test]
fn non_production_manifests_are_rejected() {
    let files = runtime_files();
    let mut value: Value = serde_json::from_slice(&manifest(&files)).unwrap();
    value["profile"] = json!("diagnostic");
    let mut builder = Builder::new(&packaging::json_bytes(&value).unwrap()).unwrap();
    for path in builder.runtime_paths().to_vec() {
        builder.add_runtime_file(&path, &files[&path]).unwrap();
    }
    assert!(error(builder.build()).contains("Expected a production build for a supported target"));
    assert!(error(Builder::new(b"{}")).contains("Expected a manifest file list"));
}

#[test]
fn duplicate_invalid_and_excess_effects_are_rejected() {
    let mut builder = builder_with_runtime(&runtime_files());
    builder.add_effect(&effect("test-a")).unwrap();
    assert!(error(builder.add_effect(&effect("test-a"))).contains("Duplicate effect ID"));
    assert!(error(builder.add_effect(b"not an effect")).contains("GRFX"));

    let mut full = builder_with_runtime(&runtime_files());
    for index in 0..runtime::MAX_COUNT {
        full.add_effect(&effect(&format!("test-{index}"))).unwrap();
    }
    assert!(error(full.add_effect(&effect("test-extra"))).contains("at most 20 effects"));
    assert!(full.build().is_ok());
}

/// The smallest valid `.cube`: the identity transform at two points per axis.
const IDENTITY_CUBE: &[u8] = b"TITLE \"Identity\"\nLUT_3D_SIZE 2\n\
    0 0 0\n1 0 0\n0 1 0\n1 1 0\n0 0 1\n1 0 1\n0 1 1\n1 1 1\n";

#[test]
fn imports_a_custom_lut_and_builds_it_into_the_package() {
    let files = runtime_files();
    let mut builder = builder_with_runtime(&files);
    let fit = builder
        .import_lut(IDENTITY_CUBE, "identity.cube", "my-lut", "My LUT", "MY", "film", "srgb")
        .unwrap();
    assert_eq!(fit.id, "my-lut");
    assert!(fit.mean_de.is_finite() && fit.max_de >= fit.mean_de);
    assert!(builder
        .import_lut(IDENTITY_CUBE, "identity.cube", "my-lut", "My LUT", "MY", "film", "srgb")
        .is_err());

    let package = builder.build().unwrap();
    let verified: Value = packaging::verify_package(&package).unwrap();
    assert_eq!(verified["effect_ids"], json!(["my-lut"]));

    builder.remove_effect("my-lut").unwrap();
    assert!(builder.remove_effect("my-lut").is_err());
    let package = builder.build().unwrap();
    let verified: Value = packaging::verify_package(&package).unwrap();
    assert_eq!(verified["effect_ids"], json!([]));
}

#[test]
fn import_lut_rejects_bad_input() {
    let files = runtime_files();
    let mut builder = builder_with_runtime(&files);
    assert!(builder
        .import_lut(b"not a cube", "bad.cube", "bad-lut", "Bad", "BAD", "film", "srgb")
        .is_err());
    assert!(builder
        .import_lut(IDENTITY_CUBE, "identity.cube", "Bad ID!", "Bad", "BAD", "film", "srgb")
        .is_err());
    assert!(builder
        .import_lut(IDENTITY_CUBE, "identity.cube", "ok-lut", "Ok", "OK", "film", "hdr")
        .is_err());
}

#[test]
fn import_lut_rejects_an_unsupported_target() {
    let files = runtime_files();
    let value = json!({"profile": "production", "target": "gr9-9.99", "startup_activation": "Off",
        "files": files.iter().map(|(path, bytes)| json!({"path": path, "size": bytes.len(),
            "sha256": packaging::sha256(bytes)})).collect::<Vec<_>>()});
    let mut builder = Builder::new(&packaging::json_bytes(&value).unwrap()).unwrap();
    let error = builder
        .import_lut(IDENTITY_CUBE, "identity.cube", "my-lut", "My LUT", "MY", "film", "srgb")
        .unwrap_err();
    assert!(format!("{error:#}").contains("gr9-9.99"), "{error:#}");
}
