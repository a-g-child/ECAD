//! Offline template publication/reopen with complete immutable paper variants.
use ecad_adapters::TemplateLibraryFile;
use ecad_app::{Recommendation, TemplatePublisher};
use ecad_domain::*;
use std::{error::Error, sync::Arc};
fn main() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let store = TemplateLibraryFile::new(directory.path().join("drawing-templates.json"));
    let reference = TemplateRevisionRef {
        definition: TemplateDefinitionId::new(1)?,
        revision: TemplateRevisionId::new(1)?,
    };
    let variants = [
        (1, SheetPaper::default()),
        (
            2,
            SheetPaper {
                width: 420_000,
                height: 297_000,
                ..SheetPaper::default()
            },
        ),
    ]
    .into_iter()
    .map(|(id, paper)| {
        SavedTemplateVariant::new(
            TemplateVariantRef {
                definition: reference.definition,
                revision: reference.revision,
                variant: TemplateVariantId::new(id).expect("fixed positive fixture ID"),
            },
            format!("Custom {} × {} µm", paper.width, paper.height),
            SheetTemplate::StandardV1
                .layout(&paper)
                .expect("valid fixture paper"),
        )
    })
    .collect::<Result<Vec<_>, _>>()?;
    let revision = Arc::new(TemplateRevision::new(reference, variants)?);
    let mut publisher = TemplatePublisher::new(store.clone());
    publisher.publish(revision.clone(), Recommendation::RecommendPublished)?;
    let reopened = TemplatePublisher::open(store)?;
    assert_eq!(
        reopened.catalog().revision(reference),
        Some(revision.as_ref())
    );
    assert!(!publisher.publish(revision, Recommendation::KeepCurrent)?);
    println!("Published and reopened one immutable template revision with two saved variants.");
    Ok(())
}
