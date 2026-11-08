//! Convert shared action results into desktop requests and display projections.
use super::*;
impl DesktopBridge {
    pub(super) fn outcome(&mut self, outcome: ActionOutcome) -> Result<Response, BridgeError> {
        let mut response = self.snapshot();
        match outcome {
            ActionOutcome::Edited(_) => {}
            ActionOutcome::Saved => response.message = Some("Project saved".into()),
            ActionOutcome::Properties(p) => {
                response.properties = Some(properties::inspect(
                    self.document.session().project(),
                    p.objects,
                )?);
            }
            ActionOutcome::Request(r) => {
                response.request = Some(
                    match r.kind {
                        ActionRequestKind::NewSheet(kind) => match kind {
                            ecad_domain::SheetKind::Schematic => "new_schematic",
                            ecad_domain::SheetKind::Cover => "new_cover",
                            ecad_domain::SheetKind::DrawingList => "new_drawing_list",
                        },
                        ActionRequestKind::ExplorerPages => "explorer_pages",
                        ActionRequestKind::ExplorerElectrical => "explorer_electrical",
                        ActionRequestKind::Sheets => "sheets",
                        ActionRequestKind::CompanyProfiles => {
                            response.profiles = self.profiles.load()?.profiles;
                            "profiles"
                        }
                        ActionRequestKind::ProjectMetadata => {
                            response.metadata = Some(metadata::project(
                                self.document.session().project().metadata(),
                            ));
                            response.project_templates =
                                Some(crate::project_templates::TemplateCatalog::new(
                                    self.document.session().project().template_defaults(),
                                ));
                            "metadata"
                        }
                        ActionRequestKind::DrawingSettings => {
                            response.defaults = Some(defaults::project(
                                r.settings.as_ref().expect("settings request"),
                            ));
                            "defaults"
                        }
                        ActionRequestKind::Palette => "palette",
                        ActionRequestKind::OpenProject => "open",
                        ActionRequestKind::FitDrawing => "fit",
                        ActionRequestKind::MoveSelection => "move",
                        ActionRequestKind::LabelPlacement => "annotation",
                        ActionRequestKind::PolylinePlacement => "path",
                        ActionRequestKind::RotateSelection => "rotate",
                        ActionRequestKind::MirrorSelection => "mirror",
                        ActionRequestKind::AlignSelection => "align",
                        ActionRequestKind::DistributeSelection => "distribute",
                        ActionRequestKind::CancelPlacement => {
                            let message = if self.path_draft.take().is_some() {
                                "Path cancelled"
                            } else if self.annotation.take().is_some() {
                                "Annotation cancelled"
                            } else if self.layout.take().is_some() {
                                "Transform cancelled"
                            } else {
                                "Move cancelled"
                            };
                            self.movement = None;
                            response.message = Some(message.into());
                            "cancel_move"
                        }
                        _ => "unsupported",
                    }
                    .into(),
                )
            }
            ActionOutcome::ConnectionReport(report) => {
                let mut rows = vec![vec![
                    "Conductor".into(),
                    "End A".into(),
                    "End B".into(),
                    "Cable".into(),
                ]];
                for w in report.conductors() {
                    let ends = w.endpoints.map(|id| {
                        id.map_or_else(
                            || "Free end".into(),
                            |id| {
                                let t = report.terminal(id).expect("report terminal");
                                format!(
                                    "{} : {}",
                                    report.device(t.device).expect("owner").designation,
                                    t.number
                                )
                            },
                        )
                    });
                    rows.push(vec![
                        w.id.to_string(),
                        ends[0].clone(),
                        ends[1].clone(),
                        w.cable.map_or_else(
                            || "—".into(),
                            |id| report.cable(id).expect("cable").designation.clone(),
                        ),
                    ]);
                }
                response.report = Some(rows);
            }
            _ => {
                return Err(BridgeError::new(
                    "unsupported",
                    "This tool's UI is not available in the first desktop slice",
                ));
            }
        }
        Ok(response)
    }
}
