//! One invocation boundary for every UI surface; commands retain model/history authority.

use crate::*;

/// Serial action boundary; share one instance across native/web/menu surfaces.
/// A host must provide one monotonic ID stream, reusing an ID for duplicate deliveries.
/// Failed attempts also consume IDs so delayed callbacks cannot replay after state changes.
#[derive(Default)]
pub struct ActionDispatcher {
    last_input: Option<InputId>,
}
impl ActionDispatcher {
    /// Record rejected routing attempts too, so a closed target cannot release an input ID.
    pub(crate) fn record_input(&mut self, input: InputId) -> Result<(), InputId> {
        if self.last_input.is_some_and(|previous| input <= previous) {
            return Err(input);
        }
        self.last_input = Some(input);
        Ok(())
    }

    /// Revalidate and execute one invocation using actual commands and document lifecycle.
    /// # Errors
    /// Reports replay/protected/disabled/parameter failures or original model/save errors.
    pub fn invoke<S: VersionedProjectStore>(
        &mut self,
        registry: &ActionRegistry,
        invocation: ActionInvocation,
        mut document: Option<&mut Document<S>>,
    ) -> Result<ActionOutcome, ActionDispatchError<S::Error>> {
        self.record_input(invocation.input_id)
            .map_err(ActionDispatchError::ReplayedInput)?;
        if matches!(
            invocation.source,
            InvocationSource::Keyboard | InvocationSource::NativeAccelerator
        ) && crate::shortcut::protected_focus(&invocation.context)
        {
            return Err(ActionDispatchError::ProtectedInput);
        }
        if invocation.context.editor.is_some() {
            return Err(ActionDispatchError::Disabled(ActionDisabled::NoDocument));
        }
        let state = registry.state(
            &invocation.action,
            &invocation.context,
            document.as_ref().map(|document| document.session()),
        );
        if let Some(reason) = state.disabled {
            return Err(ActionDispatchError::Disabled(reason));
        }
        let definition = registry
            .get(&invocation.action)
            .expect("availability resolved registered action");
        if invocation.repeat && definition.repeat == ActionRepeat::Ignore {
            return Err(ActionDispatchError::RepeatedInput);
        }
        let handler = definition.handler;
        let request = |kind, settings| {
            ActionOutcome::Request(Box::new(ActionRequest {
                action: invocation.action.clone(),
                kind,
                context: invocation.context.clone(),
                settings,
            }))
        };
        match (handler, &invocation.parameters) {
            (ActionHandler::TemplateLibrary, ActionParameters::None) => {
                return Ok(request(ActionRequestKind::TemplateLibrary, None));
            }
            (ActionHandler::TemplateLibrary, _) => {
                return Err(ActionDispatchError::WrongParameters);
            }
            (ActionHandler::OpenProject, ActionParameters::None) => {
                return Ok(request(ActionRequestKind::OpenProject, None));
            }
            (ActionHandler::OpenProject, _) => return Err(ActionDispatchError::WrongParameters),
            (ActionHandler::Palette, ActionParameters::None) => {
                return Ok(request(ActionRequestKind::Palette, None));
            }
            (ActionHandler::CancelPlacement, ActionParameters::None) => {
                return Ok(request(ActionRequestKind::CancelPlacement, None));
            }
            (ActionHandler::Palette | ActionHandler::CancelPlacement, _) => {
                return Err(ActionDispatchError::WrongParameters);
            }
            _ => {}
        }
        let document = document
            .as_mut()
            .ok_or(ActionDispatchError::Disabled(ActionDisabled::NoDocument))?;
        let revision = invocation
            .context
            .revision
            .expect("document availability checked revision");
        let outcome = match (handler, invocation.parameters) {
            (ActionHandler::ToggleExplorer, ActionParameters::None) => {
                request(ActionRequestKind::ToggleExplorer, None)
            }
            (ActionHandler::ToggleProperties, ActionParameters::None) => {
                request(ActionRequestKind::ToggleProperties, None)
            }
            (ActionHandler::ExplorerPages, ActionParameters::None) => {
                request(ActionRequestKind::ExplorerPages, None)
            }
            (ActionHandler::ExplorerElectrical, ActionParameters::None) => {
                request(ActionRequestKind::ExplorerElectrical, None)
            }
            (ActionHandler::NewSheet(kind), ActionParameters::None) => {
                request(ActionRequestKind::NewSheet(kind), None)
            }
            (ActionHandler::FitDrawing, ActionParameters::None) => {
                request(ActionRequestKind::FitDrawing, None)
            }
            (ActionHandler::ConnectionReport, ActionParameters::None) => {
                ActionOutcome::ConnectionReport(Box::new(ConnectionReport::from_session(
                    document.session(),
                )))
            }
            (ActionHandler::Undo, ActionParameters::None) => {
                ActionOutcome::Edited(document.undo(revision).map_err(ActionDispatchError::Edit)?)
            }
            (ActionHandler::Redo, ActionParameters::None) => {
                ActionOutcome::Edited(document.redo(revision).map_err(ActionDispatchError::Edit)?)
            }
            (ActionHandler::Save, ActionParameters::None) => {
                document.save().map_err(ActionDispatchError::Store)?;
                ActionOutcome::Saved
            }
            (ActionHandler::Properties, ActionParameters::None) => ActionOutcome::Properties(
                inspect_properties(
                    document.session().project(),
                    invocation.context.selection.iter().copied(),
                )
                .map_err(ActionDispatchError::Model)?,
            ),
            (
                ActionHandler::MoveSelection,
                ActionParameters::PreviewMove {
                    reference,
                    placement,
                },
            ) => ActionOutcome::MovePreview(Box::new(
                preview_move(
                    document.session(),
                    invocation.context.selection.iter().copied(),
                    reference,
                    placement,
                )
                .map_err(ActionDispatchError::MovePreview)?,
            )),
            (ActionHandler::MoveSelection, ActionParameters::CommitMove(preview)) => {
                if preview.layout().selection() != &invocation.context.selection
                    || Some(preview.layout().view()) != invocation.context.view
                {
                    return Err(ActionDispatchError::WrongParameters);
                }
                ActionOutcome::Edited(
                    document
                        .apply_layout_preview(preview.into_layout())
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (ActionHandler::RotateSelection, ActionParameters::None) => {
                request(ActionRequestKind::RotateSelection, None)
            }
            (ActionHandler::MirrorSelection, ActionParameters::None) => {
                request(ActionRequestKind::MirrorSelection, None)
            }
            (ActionHandler::MoveSelection, ActionParameters::None) => {
                request(ActionRequestKind::MoveSelection, None)
            }
            (ActionHandler::AlignSelection, ActionParameters::None) => {
                request(ActionRequestKind::AlignSelection, None)
            }
            (ActionHandler::DistributeSelection, ActionParameters::None) => {
                request(ActionRequestKind::DistributeSelection, None)
            }
            (
                ActionHandler::MoveSelection
                | ActionHandler::RotateSelection
                | ActionHandler::MirrorSelection
                | ActionHandler::AlignSelection
                | ActionHandler::DistributeSelection,
                ActionParameters::Layout(operation),
            ) => {
                if !handler.accepts_layout(operation) {
                    return Err(ActionDispatchError::WrongParameters);
                }
                let plan = layout_plan(
                    document.session().project(),
                    invocation.context.selection.iter().copied(),
                    operation,
                )
                .map_err(ActionDispatchError::Layout)?;
                ActionOutcome::Edited(
                    document
                        .execute_batch(revision, plan.commands)
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (
                ActionHandler::MoveSelection
                | ActionHandler::RotateSelection
                | ActionHandler::MirrorSelection
                | ActionHandler::AlignSelection
                | ActionHandler::DistributeSelection,
                ActionParameters::PreviewLayout(operation),
            ) => {
                if !handler.accepts_layout(operation) {
                    return Err(ActionDispatchError::WrongParameters);
                }
                ActionOutcome::LayoutPreview(Box::new(
                    preview_layout(
                        document.session(),
                        invocation.context.selection.iter().copied(),
                        operation,
                    )
                    .map_err(ActionDispatchError::Layout)?,
                ))
            }
            (
                ActionHandler::MoveSelection
                | ActionHandler::RotateSelection
                | ActionHandler::MirrorSelection
                | ActionHandler::AlignSelection
                | ActionHandler::DistributeSelection,
                ActionParameters::CommitLayout(preview),
            ) => {
                if !handler.accepts_layout(preview.operation())
                    || preview.selection() != &invocation.context.selection
                    || Some(preview.view()) != invocation.context.view
                {
                    return Err(ActionDispatchError::WrongParameters);
                }
                ActionOutcome::Edited(
                    document
                        .apply_layout_preview(preview)
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (ActionHandler::CreateRoute, ActionParameters::None) => {
                request(ActionRequestKind::RoutePlacement, None)
            }
            (
                ActionHandler::CreateRoute,
                ActionParameters::Route {
                    id,
                    conductor,
                    anchors,
                    points,
                    mode,
                },
            ) => ActionOutcome::Edited(
                document
                    .execute(
                        revision,
                        Command::Route(RouteCommand::Create {
                            id,
                            conductor,
                            anchors,
                            points,
                            mode,
                            view: invocation
                                .context
                                .view
                                .expect("placement availability checked view"),
                        }),
                    )
                    .map_err(ActionDispatchError::Edit)?,
            ),
            (ActionHandler::CreateLabel, ActionParameters::None) => {
                request(ActionRequestKind::LabelPlacement, None)
            }
            (ActionHandler::CreatePolyline, ActionParameters::None) => {
                request(ActionRequestKind::PolylinePlacement, None)
            }
            (ActionHandler::DrawingDocuments, ActionParameters::None) => {
                request(ActionRequestKind::DrawingDocuments, None)
            }
            (ActionHandler::DrawingDocuments, ActionParameters::DrawingDocument(command)) => {
                ActionOutcome::Edited(
                    document
                        .execute(revision, Command::DrawingDocument(command))
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (ActionHandler::Sheets, ActionParameters::None) => {
                request(ActionRequestKind::Sheets, None)
            }
            (ActionHandler::Sheets, ActionParameters::SheetPreview(preview)) => {
                if preview.revision() != revision {
                    return Err(ActionDispatchError::WrongParameters);
                }
                ActionOutcome::Edited(
                    document
                        .execute_batch(revision, preview.commands)
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (ActionHandler::Sheets, ActionParameters::Sheet(command)) => ActionOutcome::Edited(
                document
                    .execute(revision, Command::Sheet(command))
                    .map_err(ActionDispatchError::Edit)?,
            ),
            (ActionHandler::CompanyProfiles, ActionParameters::None) => {
                request(ActionRequestKind::CompanyProfiles, None)
            }
            (ActionHandler::ProjectMetadata, ActionParameters::ProfileImport(preview)) => {
                if preview.revision() != revision {
                    return Err(ActionDispatchError::WrongParameters);
                }
                ActionOutcome::Edited(
                    document
                        .execute(revision, Command::ProjectMetadata(preview.proposed))
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (ActionHandler::ProjectMetadata, ActionParameters::None) => {
                request(ActionRequestKind::ProjectMetadata, None)
            }
            (
                ActionHandler::ProjectMetadata,
                ActionParameters::ProjectProperties {
                    metadata,
                    templates,
                },
            ) => ActionOutcome::Edited(
                document
                    .execute_batch(
                        revision,
                        vec![
                            Command::ProjectMetadata(metadata),
                            Command::TemplateDefaults(templates),
                        ],
                    )
                    .map_err(ActionDispatchError::Edit)?,
            ),
            (ActionHandler::ProjectMetadata, ActionParameters::ProjectMetadata(values)) => {
                ActionOutcome::Edited(
                    document
                        .execute(revision, Command::ProjectMetadata(values))
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (ActionHandler::DrawingSettings, ActionParameters::None) => request(
                ActionRequestKind::DrawingSettings,
                Some(document.session().project().drawing().settings().clone()),
            ),
            (ActionHandler::CreateLabel, ActionParameters::Label { id, text, position }) => {
                ActionOutcome::Edited(
                    document
                        .execute(
                            revision,
                            Command::Drawing(DrawingCommand::CreateLabel {
                                id,
                                text,
                                position,
                                view: invocation
                                    .context
                                    .view
                                    .expect("placement availability checked view"),
                            }),
                        )
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (ActionHandler::CreatePolyline, ActionParameters::Polyline { id, points }) => {
                ActionOutcome::Edited(
                    document
                        .execute(
                            revision,
                            Command::Drawing(DrawingCommand::CreatePolyline {
                                id,
                                points,
                                view: invocation
                                    .context
                                    .view
                                    .expect("placement availability checked view"),
                            }),
                        )
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            (ActionHandler::DrawingSettings, ActionParameters::DrawingSettings(settings)) => {
                ActionOutcome::Edited(
                    document
                        .execute(
                            revision,
                            Command::Drawing(DrawingCommand::Settings(settings)),
                        )
                        .map_err(ActionDispatchError::Edit)?,
                )
            }
            _ => return Err(ActionDispatchError::WrongParameters),
        };
        Ok(outcome)
    }
}
