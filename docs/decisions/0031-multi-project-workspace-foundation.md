# 0031: Independent open-project workspace sessions

Status: headless application foundation implemented. Native/bridge/explorer/tab integration remains pending.

## Decision

A workspace can hold several projects; the primary explorer has project roots with document/sheet pages, while electrical/component structure is scoped to the selected project. Projects own independent models, metadata, revisions, histories, dirty/checkpoint state and store bindings. Sheet tabs within a project share that project session/history. Do not key workspace entries solely by persistent ProjectId: file copies, save-as and Git branches can share lineage and object IDs.

ProjectWorkspace<S> owns Document<S> entries in opening order. Allocate a process-local WorkspaceProjectId from the initial unique session epoch and retain it across reloads; reload still gives the document a new revision epoch. Closing drops the entry and never reuses its handle. Active selection is transient workspace state. Every operation supplies its target explicitly, so a captured callback for A can execute against A while B is selected. Cross-project revisions fail through existing session guards.

The host supplies a normalized opaque location key matching each concrete store. The application does not infer keys from display names/project metadata, canonicalize paths or perform concrete filesystem IO. Opening a reserved key focuses its current session without reloading/discarding edits. New-project creation and Save-as reject reserved destinations; failed saves do not rebind keys/checkpoints. The native adapter must normalize aliases/case/symlinks consistently before exposing this API; the headless contract does not establish filesystem-wide alias safety.

Workspace-targeted command, batch, undo/redo, save/save-as/reload and close APIs retain the existing Document contracts. Close requires the captured revision and clean state unless discard was explicitly chosen. List all dirty projects for eventual close/quit preflight, including inactive/unsaved entries. Closing an inactive project leaves focus alone; closing active selects its next neighbour or previous final entry.

A shared action dispatcher consumes one workspace-wide input stream across project switches. Rejected routing to a closed/unknown handle also consumes the input identity, preventing delayed delivery from replaying into another project. Central registry dispatch remains the action boundary; low-level workspace methods serve host/application orchestration.

Keep profile/library services and workspace preferences outside project undo. Do not keep a second authoritative Document inside each future bridge context after moving ownership into ProjectWorkspace. Bridge requests must carry project target and revision, and native dialogs capture the original target before awaiting. Detailed work order and rework risks are in [the implementation task list](../workspace-and-sheet-implementation.md).

## Evidence and next work

Five adapter workflows exercise two real same-lineage project files: independent electrical/metadata/history/dirty/checkpoint state, wrong-project revisions, targeted saves, duplicate focus, reserved/failed Save-as, failed opens/reloads, external conflicts, stable handles across reload, stale/dirty close, unsaved projects, transaction rollback and shared action replay including closed targets. The executable workspace example creates/cleans temporary files and demonstrates editing/saving A while B is active.

Local verification: cargo xtask full (310 behavior tests and two doctests), plus the offline workspace example. Project schema v8 and the UI are unchanged; UI/native checks are not evidence of multi-project support yet. Next work introduces logical document ownership and sheet migration, then unifies sheet editing and wires the multi-project host/bridge/explorers/tabs. Actual Windows/Linux execution, path normalization policy, aggregate quit handling, native picker races and large-workspace rendering still need implementation/validation.
