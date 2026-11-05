//! Opt-in coverage of the real filesystem, including Windows mapped SMB drives.
use photocraft_color::{ColorMode, SampleType};
use photocraft_doc::{Document, Layer, Size};
use photocraft_format::{PcraftWriter, SaveOptions, atomic_write, atomic_write_stream, load_path};

/// Set PHOTOCRAFT_NETWORK_SAVE_DIR to an existing writable folder, then run this ignored test.
/// All writes stay in a uniquely named child folder; existing documents are never replaced.
#[test]
#[ignore = "requires PHOTOCRAFT_NETWORK_SAVE_DIR on a writable network share"]
fn network_saves_create_replace_and_reopen() {
    let parent = std::env::var_os("PHOTOCRAFT_NETWORK_SAVE_DIR").expect("set PHOTOCRAFT_NETWORK_SAVE_DIR");
    let dir = std::path::PathBuf::from(parent).join(format!("photocraft-save-check-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let mut doc = Document::new("network save", Size::new(2, 2), ColorMode::Rgb, SampleType::U8);
    let mut layer = Layer::raster("pixels", doc.pixel_format());
    layer.surface_mut().unwrap().write_pixel(0, 0, &[0.25, 0.5, 0.75, 1.0]);
    doc.layers.push(layer);
    doc.metadata.exif = Some(std::sync::Arc::new(vec![1, 2, 3]));
    let mut writer = PcraftWriter::new();
    for name in ["first", "replacement"] {
        atomic_write(&dir.join("bytes.bin"), name.as_bytes()).unwrap();
        atomic_write_stream(&dir.join("stream.bin"), &mut |w| w.write_all(name.as_bytes())).unwrap();
        assert_eq!(std::fs::read(dir.join("bytes.bin")).unwrap(), name.as_bytes());
        assert_eq!(std::fs::read(dir.join("stream.bin")).unwrap(), name.as_bytes());
        doc.name = name.into();
        writer.save_path(&doc, &dir.join("document.pcraft"), &SaveOptions::default()).unwrap();
        assert_eq!(load_path(&dir.join("document.pcraft")).unwrap(), doc);
        writer.save_dir(&doc, &dir.join("bundle"), &SaveOptions::default()).unwrap();
        assert_eq!(load_path(&dir.join("bundle")).unwrap(), doc);
    }
    std::fs::remove_dir_all(dir).unwrap();
}
