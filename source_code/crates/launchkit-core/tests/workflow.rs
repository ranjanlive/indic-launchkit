use launchkit_core as core;
use std::{fs, path::Path};
fn fixture(base: &Path) -> std::path::PathBuf {
    let root = base.join("project");
    core::init(&root, "Little Steps", "com.example.steps", "aurora").unwrap();
    let icon = base.join("icon.png");
    image::RgbaImage::from_pixel(128, 128, image::Rgba([101, 83, 216, 255]))
        .save(&icon)
        .unwrap();
    let shot = base.join("screen.png");
    image::RgbaImage::from_pixel(240, 426, image::Rgba([240, 235, 254, 255]))
        .save(&shot)
        .unwrap();
    core::import(&root, &icon, "icon", "").unwrap();
    core::import(&root, &shot, "screenshot", "Make room for progress").unwrap();
    root
}
#[test]
fn export_is_checksummed_and_existing_output_is_preserved() {
    let temp = tempfile::tempdir().unwrap();
    let root = fixture(temp.path());
    let mut p = core::load(&root).unwrap();
    p.tagline = "A calmer way to organize your day".into();
    let out = temp.path().join("release");
    let planned = core::export(&root, &p, &out, true, false).unwrap();
    assert!(!out.exists());
    assert_eq!(planned.artifacts.len(), 7);
    assert!(!temp.path().join(".release.launchkit-lock").exists());
    let actual = core::export(&root, &p, &out, false, false).unwrap();
    assert_eq!(planned.artifacts.len(), actual.artifacts.len());
    for file in &actual.artifacts {
        assert_eq!(core::hash_file(&out.join(&file.file)).unwrap(), file.sha256);
        assert!(file.bytes > 0);
    }
    assert_eq!(
        image::image_dimensions(out.join("assets/screenshots/01.png")).unwrap(),
        (1080, 1920)
    );
    assert_eq!(
        image::image_dimensions(out.join("assets/feature-graphic.png")).unwrap(),
        (1024, 500)
    );
    let receipt = fs::read(out.join("SHA256SUMS.txt")).unwrap();
    assert!(core::export(&root, &p, &out, false, false).is_err());
    assert_eq!(receipt, fs::read(out.join("SHA256SUMS.txt")).unwrap());
    assert!(!temp.path().join(".release.launchkit-lock").exists());
}
#[test]
fn all_templates_preview_without_network_or_external_svg() {
    let temp = tempfile::tempdir().unwrap();
    let root = fixture(temp.path());
    let mut p = core::load(&root).unwrap();
    p.title = "प्रगति".into();
    for template in ["aurora", "paper", "midnight"] {
        p.template = template.into();
        let bytes = core::render::png(&root, &p, "screenshot", 0, true).unwrap();
        assert_eq!(image::load_from_memory(&bytes).unwrap().width(), 480);
    }
}
#[test]
fn project_validation_rejects_escape_unknown_fields_and_missing_assets() {
    let temp = tempfile::tempdir().unwrap();
    let root = fixture(temp.path());
    let mut p = core::load(&root).unwrap();
    p.icon = Some("../icon.png".into());
    assert!(core::save(&root, &p).is_err());
    p.icon = Some("assets/../../icon.png".into());
    assert!(core::save(&root, &p).is_err());
    let mut raw = serde_json::to_value(core::load(&root).unwrap()).unwrap();
    raw["execute"] = "bad".into();
    fs::write(root.join(core::FILE_NAME), raw.to_string()).unwrap();
    assert!(core::load(&root).is_err());
    let empty = temp.path().join("empty");
    let p = core::init(&empty, "Empty", "", "aurora").unwrap();
    assert!(core::export(&empty, &p, &temp.path().join("out"), true, false).is_err());
}
#[test]
fn invalid_caption_and_fake_apk_do_not_change_project() {
    let temp = tempfile::tempdir().unwrap();
    let root = fixture(temp.path());
    let before = fs::read(root.join(core::FILE_NAME)).unwrap();
    assert!(core::import(
        &root,
        &temp.path().join("screen.png"),
        "screenshot",
        &"x".repeat(181)
    )
    .is_err());
    let fake = temp.path().join("fake.apk");
    fs::write(&fake, "not an apk").unwrap();
    assert!(core::import(&root, &fake, "apk", "").is_err());
    assert_eq!(before, fs::read(root.join(core::FILE_NAME)).unwrap());
}
#[test]
fn dry_run_catches_layout_and_missing_apk() {
    let temp = tempfile::tempdir().unwrap();
    let root = fixture(temp.path());
    let mut p = core::load(&root).unwrap();
    let out = temp.path().join("out");
    assert!(core::export(&root, &p, &out, true, true).is_err());
    p.title = "W".repeat(64);
    assert!(core::export(&root, &p, &out, true, false).is_err());
    assert!(!out.exists());
}
#[cfg(unix)]
#[test]
fn symlink_assets_are_rejected() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let root = fixture(temp.path());
    symlink(temp.path().join("icon.png"), root.join("assets/link.png")).unwrap();
    assert!(core::asset(&root, "assets/link.png").is_err());
}
