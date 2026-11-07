//! Advisory registered-location index; failures retain clearly labelled last-known usage.

use crate::{ProjectUsage, RevisionUse, VersionedProjectStore};
use ecad_domain::{DefinitionId, DefinitionRef};
use std::{collections::BTreeMap, error::Error, fmt};

/// Result of the most recent explicit load/check, never a continuous freshness guarantee.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsageStatus {
    /// Registered but no successful observation has been made.
    Unscanned,
    /// Last refresh loaded validated bytes; no later freshness check has been made.
    Observed,
    /// Last content check matched the observed bytes/location.
    MatchesObserved,
    /// Last content check differed; cached usage is historical until refreshed.
    Changed,
    /// Last content check found no destination; cached usage is historical.
    Missing,
    /// Last load/check failed; any retained usage is historical.
    Unavailable,
}

/// Last validated summary/version plus the outcome of the most recent explicit check.
pub struct RegisteredUsage<V> {
    observation: Option<(ProjectUsage, V)>,
    status: UsageStatus,
    diagnostic: Option<String>,
}
impl<V> RegisteredUsage<V> {
    /// Last validated usage, retained even if the file later disappears or becomes invalid.
    pub fn last_known(&self) -> Option<&ProjectUsage> {
        self.observation.as_ref().map(|(usage, _)| usage)
    }
    /// Last explicit operation's result; no filesystem watcher is implied.
    pub fn status(&self) -> UsageStatus {
        self.status
    }
    /// Human-readable last IO/validation failure; typed errors are also returned to callers.
    pub fn diagnostic(&self) -> Option<&str> {
        self.diagnostic.as_deref()
    }
}

/// An indexed operation did not run or could not read its registered source.
#[derive(Debug)]
pub enum UsageRefreshError<E> {
    /// Registration is required; no store operation ran.
    Unregistered,
    /// IO/validation failed; last-known usage was retained and marked unavailable.
    Store(E),
}
impl<E: fmt::Display> fmt::Display for UsageRefreshError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unregistered => f.write_str("project location is not registered"),
            Self::Store(error) => error.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for UsageRefreshError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            Self::Unregistered => None,
        }
    }
}

/// One last-known revision reference at one registered location, with its freshness state.
pub struct KnownUse<'a, K, V> {
    /// Registered location, not a unique project lineage or global consumer identity.
    pub location: &'a K,
    /// Last observation and most recent explicit status/diagnostic.
    pub record: &'a RegisteredUsage<V>,
    /// Exact source revision referenced by the last validated observation.
    pub source: DefinitionRef,
    /// Pins and local origins; both empty means bundled-only content.
    pub usage: &'a RevisionUse,
}

/// Rebuildable, in-memory index keyed by caller-owned location identities.
/// Never writes projects, discovers unregistered copies, or blocks publication.
pub struct KnownUseIndex<K, V> {
    entries: BTreeMap<K, RegisteredUsage<V>>,
}
impl<K, V> Default for KnownUseIndex<K, V> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}
impl<K: Ord, V: Eq> KnownUseIndex<K, V> {
    /// Register a location without IO. Duplicate registration preserves cached observations.
    pub fn register(&mut self, key: K) -> bool {
        if self.entries.contains_key(&key) {
            return false;
        }
        self.entries.insert(
            key,
            RegisteredUsage {
                observation: None,
                status: UsageStatus::Unscanned,
                diagnostic: None,
            },
        );
        true
    }
    /// Forget one registered location and its advisory cache; changes no project.
    pub fn unregister(&mut self, key: &K) -> bool {
        self.entries.remove(key).is_some()
    }
    /// Inspect every registration, including unscanned/unreadable locations.
    pub fn entries(&self) -> impl ExactSizeIterator<Item = (&K, &RegisteredUsage<V>)> {
        self.entries.iter()
    }
    /// Inspect one location without conflating copies sharing a project ID.
    pub fn entry(&self, key: &K) -> Option<&RegisteredUsage<V>> {
        self.entries.get(key)
    }
    /// Load a validated snapshot and replace only this location's summary on success.
    /// # Errors
    /// Unknown keys perform no IO. Store failures mark unavailable and retain prior usage.
    pub fn refresh<S: VersionedProjectStore<Version = V>>(
        &mut self,
        key: &K,
        store: &S,
    ) -> Result<(), UsageRefreshError<S::Error>> {
        let record = self
            .entries
            .get_mut(key)
            .ok_or(UsageRefreshError::Unregistered)?;
        match store.load_snapshot() {
            Ok(snapshot) => {
                record.observation = Some((
                    ProjectUsage::from_project(&snapshot.project),
                    snapshot.version,
                ));
                record.status = UsageStatus::Observed;
                record.diagnostic = None;
                Ok(())
            }
            Err(error) => {
                record.status = UsageStatus::Unavailable;
                record.diagnostic = Some(error.to_string());
                Err(UsageRefreshError::Store(error))
            }
        }
    }
    /// Compare the observed version with the source's current version, without loading it.
    /// A matching check never manufactures an observation after a failed first load.
    /// # Errors
    /// Unknown keys perform no IO. Store failures mark unavailable and retain prior usage.
    pub fn check_freshness<S: VersionedProjectStore<Version = V>>(
        &mut self,
        key: &K,
        store: &S,
    ) -> Result<UsageStatus, UsageRefreshError<S::Error>> {
        let record = self
            .entries
            .get_mut(key)
            .ok_or(UsageRefreshError::Unregistered)?;
        match store.current_version() {
            Ok(version) => {
                record.status = match (&record.observation, version) {
                    (_, None) => UsageStatus::Missing,
                    (Some((_, previous)), Some(current)) if *previous == current => {
                        UsageStatus::MatchesObserved
                    }
                    (Some(_), Some(_)) => UsageStatus::Changed,
                    (None, Some(_)) => UsageStatus::Unscanned,
                };
                record.diagnostic = None;
                Ok(record.status)
            }
            Err(error) => {
                record.status = UsageStatus::Unavailable;
                record.diagnostic = Some(error.to_string());
                Err(UsageRefreshError::Store(error))
            }
        }
    }
    /// Query a definition lineage across all last-known observations, including stale hits.
    /// Empty results mean no recorded references within these registrations, never global absence.
    pub fn known_uses(&self, lineage: DefinitionId) -> impl Iterator<Item = KnownUse<'_, K, V>> {
        self.entries.iter().flat_map(move |(location, record)| {
            record
                .last_known()
                .into_iter()
                .flat_map(move |observation| {
                    observation
                        .revisions()
                        .filter(move |(source, _)| source.definition() == lineage)
                        .map(move |(source, usage)| KnownUse {
                            location,
                            record,
                            source,
                            usage,
                        })
                })
        })
    }
}
