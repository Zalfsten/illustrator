use constraint_runtime::{GeometryRole, Point2};
use sketch_export::Format;
use sketch_session::{Command, GeometrySpec, SegmentParams};

use super::*;
use crate::fake_files::FakeFiles;

/// Create a segment `name` from (0, 0) to (x, 0).
fn segment(name: &str, x: f64) -> Command {
    Command::Create {
        name: Some(name.into()),
        role: GeometryRole::Result,
        look: Default::default(),
        geometry: GeometrySpec::Segment(SegmentParams {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(x, 0.0),
        }),
    }
}

/// A session holding `seg1` and its file state, on `fake`.
fn drawing(fake: &FakeFiles) -> (Session, FileState) {
    let mut session = Session::new(Document::new());
    session.apply(segment("seg1", 1.0)).unwrap();
    let mut files = FileState::new(&session);
    files.refresh(&session, fake);
    (session, files)
}

/// `drawing` opened from the file `name`.
fn opened(fake: &FakeFiles, name: &str) -> (Session, FileState) {
    let (mut session, mut files) = drawing(fake);
    let bytes = session.save().unwrap();
    files.command(FileCommand::Open, &mut session, fake);
    fake.open_file(name, &bytes);
    let step = files.poll(&mut session, fake);
    assert_eq!(step.effect, Some(Effect::Replaced));
    (session, files)
}

fn edit(session: &mut Session, files: &mut FileState, fake: &FakeFiles, cmd: Command) {
    session.apply(cmd).unwrap();
    files.refresh(session, fake);
}

fn names(session: &Session) -> Vec<String> {
    let doc = session.document();
    doc.geometries()
        .values()
        .map(|g| doc.geometry_name(g.id()).unwrap().to_owned())
        .collect()
}

// --- Unsaved changes compare with the saved bytes ---

#[test]
fn fresh_session_is_not_unsaved() {
    let fake = FakeFiles::new(true);
    let (_, files) = drawing(&fake);
    assert!(!files.unsaved());
    assert_eq!(files.file_name(), None, "start without a file");
}

#[test]
fn edit_makes_the_drawing_unsaved_and_undo_clears_it() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    assert!(files.unsaved());
    session.undo().unwrap();
    files.refresh(&session, &fake);
    assert!(!files.unsaved());
}

#[test]
fn different_edit_after_undo_is_unsaved() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = opened(&fake, "a.sketch.json");
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    session.undo().unwrap();
    files.refresh(&session, &fake);
    assert!(!files.unsaved());
    edit(&mut session, &mut files, &fake, segment("seg3", 3.0));
    assert!(files.unsaved());
}

// --- Save and Save As ---

#[test]
fn save_as_changes_the_drawings_file() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = opened(&fake, "a.sketch.json");
    files.command(FileCommand::SaveAs, &mut session, &fake);
    assert_eq!(fake.save_request().1.as_deref(), Some("a.sketch.json"));
    fake.save_as("b.sketch.json");
    let step = files.poll(&mut session, &fake);
    assert_eq!(step.message, Some(Ok("Saved b.sketch.json".into())));
    assert_eq!(files.file_name(), Some("b.sketch.json"));

    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    files.command(FileCommand::Save, &mut session, &fake);
    let (overwrite, _, bytes) = fake.save_request();
    assert_eq!(overwrite.as_deref(), Some("b.sketch.json"));
    assert_eq!(bytes, session.save().unwrap());
}

#[test]
fn save_twice_keeps_history() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = opened(&fake, "a.sketch.json");
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    files.command(FileCommand::Save, &mut session, &fake);
    let written = fake.save_as("a.sketch.json");
    files.poll(&mut session, &fake);
    assert!(!files.unsaved());
    let (loaded, _) = Session::load(&written).unwrap();
    assert_eq!(names(&loaded), ["seg1", "seg2"], "the file holds the edit");
    assert!(session.can_undo(), "undo still reverses the edit");
}

#[test]
fn save_without_a_file_asks_for_untitled() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    files.command(FileCommand::Save, &mut session, &fake);
    let (overwrite, suggested, _) = fake.save_request();
    assert_eq!(overwrite, None);
    assert_eq!(suggested.as_deref(), Some(UNTITLED_FILE));
}

#[test]
fn cancelled_save_as_changes_nothing() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = opened(&fake, "a.sketch.json");
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    files.command(FileCommand::SaveAs, &mut session, &fake);
    fake.complete_save(None);
    assert_eq!(files.poll(&mut session, &fake), Step::default());
    assert_eq!(files.file_name(), Some("a.sketch.json"));
    assert!(files.unsaved());
    assert!(!files.busy());
}

#[test]
fn save_on_download_only_host_is_a_download() {
    let fake = FakeFiles::new(false);
    let (mut session, mut files) = opened(&fake, "a.sketch.json");
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    files.command(FileCommand::Save, &mut session, &fake);
    let (overwrite, suggested, bytes) = fake.save_request();
    assert_eq!(overwrite, None, "save behaves as save as");
    assert_eq!(suggested.as_deref(), Some("a.sketch.json"));
    assert_eq!(bytes, session.save().unwrap());
    fake.save_as("a.sketch.json");
    let step = files.poll(&mut session, &fake);
    assert_eq!(step.message, Some(Ok("Downloaded a.sketch.json".into())));
    assert!(!files.unsaved());
}

#[test]
fn failed_save_keeps_the_unsaved_state() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = opened(&fake, "a.sketch.json");
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    files.command(FileCommand::SaveAs, &mut session, &fake);
    fake.complete_save(Some(Err("permission denied".into())));
    let step = files.poll(&mut session, &fake);
    assert_eq!(
        step.message,
        Some(Err("Save failed: permission denied".into()))
    );
    assert!(files.unsaved());
    assert_eq!(files.file_name(), Some("a.sketch.json"));
}

// --- Unsaved changes are not discarded silently ---

#[test]
fn new_without_changes_asks_nothing() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    let step = files.command(FileCommand::New, &mut session, &fake);
    assert_eq!(step.effect, Some(Effect::Replaced));
    assert!(!files.asking());
    assert!(names(&session).is_empty());
}

#[test]
fn new_after_editing_discards_on_confirmation() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = opened(&fake, "a.sketch.json");
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    edit(&mut session, &mut files, &fake, segment("seg3", 3.0));
    assert_eq!(
        files.command(FileCommand::New, &mut session, &fake),
        Step::default()
    );
    assert!(files.asking());
    let step = files.answer(Answer::Discard, &mut session, &fake);
    assert_eq!(step.effect, Some(Effect::Replaced));
    assert!(names(&session).is_empty());
    assert!(session.document().constraints().is_empty());
    assert!(!session.can_undo());
    assert_eq!(files.file_name(), None);
    assert!(!files.unsaved());
}

#[test]
fn cancel_before_open_requests_no_file() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    files.command(FileCommand::Open, &mut session, &fake);
    assert_eq!(
        files.answer(Answer::Cancel, &mut session, &fake),
        Step::default()
    );
    assert_eq!(fake.pending_opens(), 0);
    assert_eq!(names(&session), ["seg1", "seg2"]);
    assert!(files.unsaved() && !files.busy());
}

#[test]
fn save_before_closing_closes_after_the_save() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    assert_eq!(
        files.command(FileCommand::Close, &mut session, &fake),
        Step::default()
    );
    files.answer(Answer::Save, &mut session, &fake);
    assert_eq!(fake.save_request().1.as_deref(), Some(UNTITLED_FILE));
    fake.save_as("c.sketch.json");
    let step = files.poll(&mut session, &fake);
    assert_eq!(step.effect, Some(Effect::Close));
    assert!(!files.unsaved());
}

#[test]
fn failed_save_before_open_does_not_open() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    files.command(FileCommand::Open, &mut session, &fake);
    files.answer(Answer::Save, &mut session, &fake);
    fake.complete_save(Some(Err("disk full".into())));
    let step = files.poll(&mut session, &fake);
    assert_eq!(step.effect, None);
    assert_eq!(fake.pending_opens(), 0);
    assert!(files.unsaved());
}

#[test]
fn save_before_open_then_opens() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    files.command(FileCommand::Open, &mut session, &fake);
    files.answer(Answer::Save, &mut session, &fake);
    fake.save_as("c.sketch.json");
    files.poll(&mut session, &fake);
    assert_eq!(fake.pending_opens(), 1);
}

#[test]
fn close_without_changes_closes() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    let step = files.command(FileCommand::Close, &mut session, &fake);
    assert_eq!(step.effect, Some(Effect::Close));
}

// --- File commands wait for the session ---

#[test]
fn file_commands_wait_for_drags_and_requests() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    session.drag_begin("seg1.end").unwrap();
    assert_eq!(
        files.command(FileCommand::New, &mut session, &fake),
        Step::default()
    );
    assert_eq!(names(&session), ["seg1"], "no New during a drag");
    session.drag_cancel();

    files.command(FileCommand::Open, &mut session, &fake);
    assert!(files.busy());
    files.command(FileCommand::Save, &mut session, &fake);
    assert_eq!(
        fake.pending_saves(),
        0,
        "no second request while one is open"
    );

    // An open completing during a drag waits for the drag to end.
    let bytes = {
        let mut other = Session::new(Document::new());
        other.apply(segment("other", 1.0)).unwrap();
        other.save().unwrap()
    };
    fake.open_file("o.sketch.json", &bytes);
    session.drag_begin("seg1.end").unwrap();
    assert_eq!(files.poll(&mut session, &fake), Step::default());
    session.drag_cancel();
    assert_eq!(
        files.poll(&mut session, &fake).effect,
        Some(Effect::Replaced)
    );
    assert_eq!(names(&session), ["other"]);
}

// --- Open ---

#[test]
fn open_saved_drawing_reports_only_the_open() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    let bytes = session.save().unwrap();
    let mut fresh = Session::new(Document::new());
    let mut state = FileState::new(&fresh);
    state.command(FileCommand::Open, &mut fresh, &fake);
    fake.open_file("a.sketch.json", &bytes);
    let step = state.poll(&mut fresh, &fake);
    assert_eq!(step.message, Some(Ok("Opened a.sketch.json".into())));
    assert_eq!(names(&fresh), ["seg1", "seg2"]);
    assert!(!fresh.can_undo());
    assert!(!state.unsaved());
    assert_eq!(state.file_name(), Some("a.sketch.json"));
}

#[test]
fn open_adjusted_file_is_unsaved_and_reported() {
    let fake = FakeFiles::new(true);
    let bytes = br#"{"format": "illustrator-sketch", "version": 2, "scale": 10, "font_size": 10, "preamble": "", "commands": [
        {"op": "create", "kind": "segment", "name": "seg1", "start": [0, 0], "end": [1, 0]},
        {"op": "constrain", "kind": "fix", "name": "fix1", "point": "seg1.start", "target": [0.01, 0]}
    ]}"#;
    let (mut session, mut files) = drawing(&fake);
    files.command(FileCommand::Open, &mut session, &fake);
    fake.open_file("adj.sketch.json", bytes);
    let step = files.poll(&mut session, &fake);
    let Some(Ok(msg)) = step.message else {
        panic!("{step:?}")
    };
    assert!(
        msg.starts_with("Opened adj.sketch.json — adjusted: seg1 (max "),
        "{msg}"
    );
    assert!(files.unsaved(), "adjusted on load");
}

#[test]
fn rejected_file_leaves_the_drawing_open() {
    let fake = FakeFiles::new(true);
    let bytes = br#"{"format": "illustrator-sketch", "version": 2, "scale": 10, "font_size": 10, "preamble": "", "commands": [
        {"op": "create", "kind": "segment", "name": "s1", "start": [0, 0], "end": [1, 0]},
        {"op": "create", "kind": "segment", "name": "s2", "start": [1, 0], "end": [1, 1]},
        {"op": "constrain", "kind": "join", "name": "j1", "a": "s1.end", "b": "s2.start"},
        {"op": "constrain", "kind": "join", "name": "j2", "a": "nope.end", "b": "s2.end"}
    ]}"#;
    let (mut session, mut files) = opened(&fake, "a.sketch.json");
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    files.command(FileCommand::SaveAs, &mut session, &fake);
    fake.save_as("a.sketch.json");
    files.poll(&mut session, &fake);

    files.command(FileCommand::Open, &mut session, &fake);
    fake.open_file("bad.sketch.json", bytes);
    let step = files.poll(&mut session, &fake);
    let Some(Err(msg)) = step.message else {
        panic!("{step:?}")
    };
    assert!(msg.contains("command 3"), "{msg}");
    assert_eq!(step.effect, None);
    assert_eq!(names(&session), ["seg1", "seg2"]);
    assert!(session.can_undo(), "history kept");
    assert_eq!(files.file_name(), Some("a.sketch.json"));
}

#[test]
fn cancelled_open_changes_nothing() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    files.command(FileCommand::Open, &mut session, &fake);
    files.answer(Answer::Discard, &mut session, &fake);
    fake.complete_open(None);
    assert_eq!(files.poll(&mut session, &fake), Step::default());
    assert_eq!(names(&session), ["seg1", "seg2"]);
    assert!(session.can_undo());
    assert!(files.unsaved() && !files.busy());
}

#[test]
fn open_message_lists_adjusted_and_frozen() {
    use sketch_session::{Adjusted, Conflict};
    let report = LoadReport {
        adjusted: vec![
            Adjusted {
                geometries: vec!["seg1".into(), "arc1".into()],
                displacement: 0.003,
            },
            Adjusted {
                geometries: vec!["seg4".into()],
                displacement: 0.001,
            },
        ],
        conflicts: vec![
            Conflict {
                geometries: vec![],
                constraints: vec![],
                residual: 1.0,
            };
            2
        ],
    };
    assert_eq!(
        open_message("a.sketch.json", &report),
        "Opened a.sketch.json — adjusted: seg1, arc1, seg4 (max 0.003) — frozen: 2 components"
    );
}

// --- Title ---

#[test]
fn title_marks_unsaved_changes() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = opened(&fake, "a.sketch.json");
    assert_eq!(fake.shown().last(), Some(&("a.sketch.json".into(), false)));
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    assert_eq!(fake.shown().last(), Some(&("a.sketch.json*".into(), true)));
    session.undo().unwrap();
    files.refresh(&session, &fake);
    assert_eq!(fake.shown().last(), Some(&("a.sketch.json".into(), false)));
    let calls = fake.shown().len();
    files.refresh(&session, &fake);
    assert_eq!(
        fake.shown().len(),
        calls,
        "unchanged title is not shown again"
    );
}

#[test]
fn untitled_title() {
    let fake = FakeFiles::new(true);
    let (_, files) = drawing(&fake);
    assert_eq!(fake.shown(), vec![("untitled".into(), false)]);
    assert_eq!(files.title(), "untitled");
}

// --- Export (drawing-export: Export command, Export leaves the drawing unchanged) ---

#[test]
fn export_suggests_the_drawings_name_with_svg() {
    let fake = FakeFiles::new(true);
    let (_, files) = opened(&fake, "triangle.sketch.json");
    assert_eq!(files.export_name("svg"), "triangle.svg");
    assert_eq!(files.export_name("pdf"), "triangle.pdf");
    let (_, files) = opened(&fake, "plain.json");
    assert_eq!(files.export_name("svg"), "plain.svg");
    let (_, files) = opened(&fake, "noext");
    assert_eq!(files.export_name("svg"), "noext.svg");
    let (_, files) = drawing(&fake);
    assert_eq!(files.export_name("svg"), "untitled.svg");
    assert_eq!(files.export_name("pdf"), "untitled.pdf");
}

#[test]
fn export_asks_for_a_target_and_keeps_the_drawings_file() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = opened(&fake, "triangle.sketch.json");
    edit(&mut session, &mut files, &fake, segment("seg2", 2.0));
    let title = files.title();
    assert!(files.export(b"<svg/>".to_vec(), Format::Svg, &fake));
    assert!(files.busy());
    let (overwrite, suggested, bytes) = fake.save_request();
    assert_eq!(overwrite, None, "always asks");
    assert_eq!(suggested.as_deref(), Some("triangle.svg"));
    assert_eq!(bytes, b"<svg/>");
    fake.save_as("figure.svg");
    let step = files.poll(&mut session, &fake);
    assert_eq!(step, Step { effect: None, message: Some(Ok("Exported figure.svg".into())) });
    assert_eq!(files.file_name(), Some("triangle.sketch.json"));
    assert!(files.unsaved(), "still unsaved");
    assert_eq!(files.title(), title);
    files.refresh(&session, &fake);
    assert!(files.unsaved(), "baseline unchanged");
    assert!(!files.busy());
}

#[test]
fn cancelled_export_writes_nothing() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    assert!(files.export(b"x".to_vec(), Format::Svg, &fake));
    fake.complete_save(None);
    assert_eq!(files.poll(&mut session, &fake), Step::default());
    assert_eq!(files.file_name(), None);
    assert!(!files.busy());
}

#[test]
fn failed_export_is_shown() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    files.export(b"x".to_vec(), Format::Svg, &fake);
    fake.complete_save(Some(Err("disk full".into())));
    let step = files.poll(&mut session, &fake);
    assert_eq!(step.message, Some(Err("Export failed: disk full".into())));
    assert!(!files.unsaved());
}

#[test]
fn export_on_download_only_host_is_a_download() {
    let fake = FakeFiles::new(false);
    let (mut session, mut files) = opened(&fake, "a.sketch.json");
    files.export(b"x".to_vec(), Format::Svg, &fake);
    assert_eq!(fake.save_request().1.as_deref(), Some("a.svg"));
    fake.save_as("a.svg");
    let step = files.poll(&mut session, &fake);
    assert_eq!(step.message, Some(Ok("Downloaded a.svg".into())));
    assert_eq!(files.file_name(), Some("a.sketch.json"));
}

#[test]
fn no_export_while_busy() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = drawing(&fake);
    files.command(FileCommand::SaveAs, &mut session, &fake);
    assert!(!files.export(b"x".to_vec(), Format::Svg, &fake));
    assert_eq!(fake.pending_saves(), 1, "only the save");
}

#[test]
fn pdf_export_suggests_pdf() {
    let fake = FakeFiles::new(true);
    let (mut session, mut files) = opened(&fake, "triangle.sketch.json");
    assert!(files.export(b"%PDF-".to_vec(), Format::Pdf, &fake));
    assert_eq!(fake.save_request().1.as_deref(), Some("triangle.pdf"));
    fake.save_as("triangle.pdf");
    let step = files.poll(&mut session, &fake);
    assert_eq!(step.message, Some(Ok("Exported triangle.pdf".into())));
    assert_eq!(files.file_name(), Some("triangle.sketch.json"));
}
