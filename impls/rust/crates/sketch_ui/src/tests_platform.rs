use std::sync::mpsc;

use super::*;
use crate::fake_files::FakeFiles;

#[test]
fn fake_files_completes_each_kind_of_request() {
    let fake = FakeFiles::new(true);
    let host = Host::new(None, Box::new(fake.clone()));
    let files = host.files();
    let (tx, rx) = mpsc::channel();

    let t = tx.clone();
    files.open(Box::new(move |r| {
        t.send(r.map(|r| r.map(|o| o.bytes))).unwrap()
    }));
    assert_eq!(fake.pending_opens(), 1);
    fake.complete_open(None);
    assert_eq!(rx.recv().unwrap(), None, "cancel");

    let t = tx.clone();
    files.open(Box::new(move |r| {
        t.send(r.map(|r| r.map(|o| o.bytes))).unwrap()
    }));
    fake.complete_open(Some(Err("unreadable".into())));
    assert_eq!(rx.recv().unwrap(), Some(Err("unreadable".into())), "error");

    let t = tx.clone();
    files.open(Box::new(move |r| {
        t.send(r.map(|r| r.map(|o| o.bytes))).unwrap()
    }));
    fake.open_file("a.sketch.json", b"{}");
    assert_eq!(rx.recv().unwrap(), Some(Ok(b"{}".to_vec())), "success");

    let (tx, rx) = mpsc::channel();
    let suggested_name = "a.sketch.json".to_owned();
    files.save(
        SaveTarget::Ask { suggested_name },
        b"data".to_vec(),
        Box::new(move |r| tx.send(r.map(|r| r.map(|h| h.name().to_owned()))).unwrap()),
    );
    assert_eq!(
        fake.save_request(),
        (None, Some("a.sketch.json".into()), b"data".to_vec())
    );
    fake.save_as("b.sketch.json");
    assert_eq!(rx.recv().unwrap(), Some(Ok("b.sketch.json".into())));

    files.show_document("b.sketch.json", false);
    assert_eq!(fake.shown(), vec![("b.sketch.json".into(), false)]);
    assert!(files.can_overwrite());
}

#[test]
fn file_handle_keeps_host_data() {
    let h = FileHandle::new("a.sketch.json", 42_u32);
    assert_eq!(h.name(), "a.sketch.json");
    assert_eq!(h.clone().host::<u32>(), Some(&42));
    assert_eq!(h.host::<String>(), None);
}
