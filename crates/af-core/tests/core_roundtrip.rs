use af_core::newline::{detect_newline, NewlineStyle};
use af_core::writer::{write_new_file, write_with_options, EditMode, WriteOptions};
use af_core::reader::read_text_file;
use std::fs;

#[test]
fn write_new_file_defaults_to_utf8_lf() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("new.txt");
    write_new_file(&path, "a\r\nb\r\n").unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(bytes, b"a\nb\n");
    let read = read_text_file(&path, None).unwrap();
    assert_eq!(read.meta.encoding, "utf-8");
    assert!(!read.meta.bom);
    assert_eq!(read.meta.eol, "lf");
}

#[test]
fn preserve_existing_crlf() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("crlf.txt");
    fs::write(&path, b"line1\r\nline2\r\n").unwrap();
    let opts = WriteOptions { mode: EditMode::Preserve, ..Default::default() };
    write_with_options(&path, "line1\nline2\nline3\n", &opts).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(bytes, b"line1\r\nline2\r\nline3\r\n");
    let read = read_text_file(&path, None).unwrap();
    assert_eq!(read.meta.eol, "crlf");
}

#[test]
fn normalize_converts_to_utf8_lf() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("n.txt");
    fs::write(&path, b"line1\r\n").unwrap();
    let opts = WriteOptions { mode: EditMode::Normalize, ..Default::default() };
    write_with_options(&path, "a\r\nb\r\n", &opts).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(bytes, b"a\nb\n");
    let read = read_text_file(&path, None).unwrap();
    assert_eq!(detect_newline(&read.content), NewlineStyle::Lf);
}
