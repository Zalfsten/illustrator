# file-commands Specification

## Purpose

Defines how the app creates, opens and saves drawings as sketch files through the host boundary, which file a drawing belongs to, and how the app protects unsaved changes from being lost.

## Requirements

### Requirement: A drawing belongs to at most one file

The app SHALL keep, besides its session, the file the drawing belongs to, or none. Opening a file and saving to a file SHALL make it the drawing's file. New and the drawing shown at start SHALL have no file. The file SHALL be named to the user by its name as the host reports it.

#### Scenario: Save As changes the drawing's file

- **WHEN** a drawing opened from `a.sketch.json` is saved with Save As to `b.sketch.json`
- **THEN** the drawing's file is `b.sketch.json`
- **AND** a later Save writes to `b.sketch.json`

#### Scenario: Start without a file

- **WHEN** the app starts
- **THEN** its drawing has no file

### Requirement: New starts an empty drawing

New SHALL replace the drawing by an empty one with default document settings (document-settings), in a new session with an empty history, and without a file.

#### Scenario: New after editing

- **WHEN** the user confirms New on a drawing with three geometries
- **THEN** the drawing has no geometries and no constraints
- **AND** undo is not available

### Requirement: Open loads a sketch file into a new session

Open SHALL ask the host for a file and load its bytes as a sketch file (sketch-file). On success the loaded drawing SHALL replace the current one in a new session with an empty history, and the file SHALL become the drawing's file. If the user cancels the host's file request, nothing SHALL change.

#### Scenario: Open a saved drawing

- **WHEN** the user opens a sketch file that holds a segment `seg1`
- **THEN** the drawing shows `seg1` with the values from the file
- **AND** undo is not available

#### Scenario: Cancelled open

- **WHEN** the user starts Open and cancels the host's file request
- **THEN** the drawing, its history and its file are unchanged

### Requirement: A rejected file leaves the drawing open

If loading fails, the current drawing, its history and its file SHALL stay unchanged, and the app SHALL show the load error. When the error belongs to one command of the file, the message SHALL name that command's position (counting from 0).

#### Scenario: Corrupt command in a file

- **WHEN** the user opens a file whose command at position 3 has an unresolvable address
- **THEN** the drawing before Open is still shown with its history
- **AND** the app shows an error that names position 3

### Requirement: Load report is shown

After a successful Open, the app SHALL show the load report (sketch-file): the components whose values were adjusted, with their geometry names and largest displacement, and the components left frozen. A load without adjustments or frozen components SHALL report only that the file was opened.

#### Scenario: Opened file with an adjusted component

- **WHEN** a file loads and one component holding `seg1` and `arc1` is adjusted
- **THEN** the app shows that `seg1` and `arc1` were adjusted, with the largest displacement

### Requirement: Save writes the drawing's file

Save SHALL write the drawing as a sketch file (sketch-file) to the drawing's file through the host. A drawing without a file SHALL be saved as with Save As. Save SHALL NOT change the drawing or its history.

#### Scenario: Save twice

- **WHEN** a drawing opened from a file is edited and saved
- **THEN** the file holds the edited drawing
- **AND** undo still reverses the edit

#### Scenario: Save without a file

- **WHEN** the user saves a drawing that has no file
- **THEN** the host asks for a target as for Save As

### Requirement: Save As asks for a target

Save As SHALL ask the host for a target, suggesting the drawing's file name, or `untitled.sketch.json` when it has none, and write the drawing there. If the user cancels the host's request, nothing SHALL change.

#### Scenario: Cancelled Save As

- **WHEN** the user starts Save As and cancels the host's request
- **THEN** the drawing's file and its unsaved-changes state are unchanged

### Requirement: Hosts that can only download

A host that cannot write to a chosen file SHALL save by handing the bytes to the user as a download under the suggested name. On such a host, Save SHALL behave as Save As, the download SHALL count as saving, and the downloaded name SHALL become the drawing's file name.

#### Scenario: Save in a download-only host

- **WHEN** a drawing opened from `a.sketch.json` is saved on a host that can only download
- **THEN** the user receives a download named `a.sketch.json` with the drawing
- **AND** the drawing has no unsaved changes

### Requirement: A failed save keeps the unsaved state

If the host reports that writing failed, the app SHALL show the error, and the drawing's file and unsaved-changes state SHALL stay as before the save.

#### Scenario: Write error

- **WHEN** saving fails because the target cannot be written
- **THEN** the app shows the host's error
- **AND** the drawing still has unsaved changes

### Requirement: Unsaved changes compare with the saved bytes

A drawing SHALL have unsaved changes exactly when saving it now would give other bytes than its file held when last opened or saved. A drawing without a file SHALL have unsaved changes when its bytes differ from those it had when it was started. The state SHALL depend on the drawing only, not on the path through its history.

#### Scenario: Undo back to the saved drawing

- **WHEN** a saved drawing is edited and the edit is undone
- **THEN** the drawing has no unsaved changes

#### Scenario: New edit after undo

- **WHEN** a saved drawing is edited, the edit is undone and a different edit is applied
- **THEN** the drawing has unsaved changes

#### Scenario: Adjusted on load

- **WHEN** a file is opened whose values the load adjusted
- **THEN** the drawing has unsaved changes immediately

### Requirement: The title shows file and unsaved state

The app's window or page title SHALL show the drawing's file name, or `untitled` without a file, and SHALL mark a drawing with unsaved changes.

#### Scenario: Edit marks the title

- **WHEN** a drawing opened from `a.sketch.json` is edited
- **THEN** the title shows `a.sketch.json` with the unsaved mark

### Requirement: Unsaved changes are not discarded silently

Before New, Open, or closing the app replaces or discards a drawing with unsaved changes, the app SHALL ask the user to save, discard or cancel. Save SHALL proceed only after a successful save; cancel SHALL leave everything unchanged. A host that can only show its own generic leave warning SHALL show that one when the app is closed.

#### Scenario: Cancel before Open

- **WHEN** the drawing has unsaved changes and the user starts Open and chooses cancel
- **THEN** no file is requested and the drawing is unchanged

#### Scenario: Save before closing

- **WHEN** the drawing has unsaved changes and the user closes the app and chooses save
- **THEN** the drawing is saved before the app closes

#### Scenario: No question without changes

- **WHEN** the drawing has no unsaved changes and the user starts New
- **THEN** the empty drawing replaces it without a question

### Requirement: File commands wait for the session

File commands SHALL NOT replace or save the drawing while a drag is in progress, and SHALL NOT block input, solving or painting while the host completes a file request.

#### Scenario: Canvas stays live during the file request

- **WHEN** the host shows a file picker for Open
- **THEN** the Canvas keeps painting and accepting input until the request completes

### Requirement: The view fits a new or opened drawing

After New or a successful Open, the view SHALL show all visible geometry and the origins of all visible labels inside the Canvas, with a margin. An empty drawing SHALL get a default view. Edits SHALL NOT refit the view. Label text extents are not required, because labels may still be typesetting when the file opens.

#### Scenario: Drawing far from the origin

- **WHEN** a file is opened whose geometry lies between (40, 20) and (60, 30)
- **THEN** all of that geometry is inside the Canvas
