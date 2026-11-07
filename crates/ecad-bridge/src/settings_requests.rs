//! Settings transport orchestration; retained profile state is separate from document history.
use super::*;
impl DesktopBridge {
    pub(super) fn settings_request(&mut self, request: Request) -> Result<Response, BridgeError> {
        match request {
            Request::UpdateMetadata {
                context,
                values,
                input,
            } => {
                let context = self.context(context)?;
                let values = metadata::parse(values)?;
                let revision = self.document.session().revision();
                let invocation = ActionInvocation {
                    input_id: input_id(&input)?,
                    action: ActionId::new("document.metadata.edit").expect("builtin"),
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters: ActionParameters::ProjectMetadata(values),
                };
                let mut response = self.invoke(invocation)?;
                response.message = Some(
                    if self.document.session().revision() == revision {
                        "Project information already matches · no document changes"
                    } else {
                        "Project information updated · one undo operation"
                    }
                    .into(),
                );
                Ok(response)
            }
            Request::UpdateDefaults {
                context,
                defaults,
                input,
            } => {
                let context = self.context(context)?;
                let settings = defaults::replace(
                    self.document.session().project().drawing().settings(),
                    defaults,
                )?;
                let revision = self.document.session().revision();
                let invocation = ActionInvocation {
                    input_id: input_id(&input)?,
                    action: ActionId::new("drawing.styles.edit").expect("builtin"),
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters: ActionParameters::DrawingSettings(settings),
                };
                let mut response = self.invoke(invocation)?;
                response.message = Some(
                    if self.document.session().revision() == revision {
                        "Drawing defaults already match · no document changes"
                    } else {
                        "Drawing defaults updated · one undo operation"
                    }
                    .into(),
                );
                Ok(response)
            }
            Request::CancelProfileImport {} => {
                self.profiles.invalidate();
                Ok(Response::default())
            }
            Request::LoadProfiles {} => self.profiles.load(),
            Request::SaveProfile {
                token,
                id,
                name,
                values,
                ..
            } => self.profiles.save(&token, id, name, values),
            Request::PreviewProfileImport {
                context,
                token,
                id,
                fields,
            } => {
                self.profiles.invalidate();
                self.context(context)?;
                self.profiles
                    .prepare(self.document.session(), &token, &id, fields)
            }
            Request::CommitProfileImport { token, input } => {
                let preview = self.profiles.take(&token)?;
                let mut context = ActionContext::for_session(self.document.session());
                context.revision = Some(preview.revision());
                let revision = self.document.session().revision();
                let invocation = ActionInvocation {
                    input_id: input_id(&input)?,
                    action: ActionId::new("document.metadata.edit").expect("builtin"),
                    source: InvocationSource::Toolbar,
                    repeat: false,
                    context,
                    parameters: ActionParameters::ProfileImport(preview),
                };
                let mut response = self.invoke(invocation)?;
                response.message = Some(
                    if revision == self.document.session().revision() {
                        "Profile values already match · no document changes"
                    } else {
                        "Profile values imported · one undo operation"
                    }
                    .into(),
                );
                Ok(response)
            }
            _ => unreachable!("settings routing"),
        }
    }
}
