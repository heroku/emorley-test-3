use crate::buildpacks::{FindReleasableBuildpacksError, ReadBuildpackDescriptorError};
use crate::changelog::ChangelogError;
use crate::commands::ResolvePathError;
use crate::github::actions::SetActionOutputError;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    ResolvePath(ResolvePathError),
    #[error(transparent)]
    FindReleasableBuildpacks(FindReleasableBuildpacksError),
    #[error(transparent)]
    ReadBuildpackDescriptor(ReadBuildpackDescriptorError),
    #[error("Could not read changelog\nPath: {}\nError: {1}", .0.display())]
    ReadingChangelog(PathBuf, #[source] std::io::Error),
    #[error("Could not parse changelog\nPath: {}\nError: {1}", .0.display())]
    ParsingChangelog(PathBuf, #[source] ChangelogError),
    #[error(transparent)]
    SetActionOutput(SetActionOutputError),
}
