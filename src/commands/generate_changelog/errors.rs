use crate::buildpacks::{FindReleasableBuildpacksError, ReadBuildpackDescriptorError};
use crate::github::actions::SetActionOutputError;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Failed to get current directory\nError: {0}")]
    GetCurrentDir(std::io::Error),
    #[error(transparent)]
    FindReleasableBuildpacks(FindReleasableBuildpacksError),
    #[error(transparent)]
    InvalidVersion(keep_a_changelog::ParseVersionError),
    #[error(transparent)]
    ReadBuildpackDescriptor(ReadBuildpackDescriptorError),
    #[error("Could not read changelog\nPath: {}\nError: {1}", .0.display())]
    ReadingChangelog(PathBuf, #[source] std::io::Error),
    #[error("Could not parse changelog\nPath: {}\nError: {1}", .0.display())]
    ParsingChangelog(PathBuf, #[source] keep_a_changelog::ParseChangelogError),
    #[error(transparent)]
    SetActionOutput(SetActionOutputError),
}
