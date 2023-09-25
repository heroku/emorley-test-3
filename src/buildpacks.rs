use libcnb_common::toml_file::{read_toml_file, TomlFileError};
use libcnb_data::buildpack::BuildpackDescriptor;
use libcnb_package::find_buildpack_dirs;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[derive(Debug)]
pub(crate) enum CalculateDigestError {
    CommandFailure(String, std::io::Error),
    ExitStatus(String, ExitStatus),
}

pub(crate) fn calculate_digest(digest_url: &String) -> Result<String, CalculateDigestError> {
    let output = Command::new("crane")
        .args(["digest", digest_url])
        .output()
        .map_err(|e| CalculateDigestError::CommandFailure(digest_url.clone(), e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(CalculateDigestError::ExitStatus(
            digest_url.clone(),
            output.status,
        ))
    }
}

pub(crate) fn read_image_repository_metadata(
    buildpack_descriptor: &BuildpackDescriptor,
) -> Option<String> {
    let metadata = match buildpack_descriptor {
        BuildpackDescriptor::Component(descriptor) => &descriptor.metadata,
        BuildpackDescriptor::Composite(descriptor) => &descriptor.metadata,
    };

    #[allow(clippy::redundant_closure_for_method_calls)]
    metadata
        .as_ref()
        .and_then(|metadata| metadata.get("release").and_then(|value| value.as_table()))
        .and_then(|release| release.get("image").and_then(|value| value.as_table()))
        .and_then(|docker| docker.get("repository").and_then(|value| value.as_str()))
        .map(|value| value.to_string())
}

pub(crate) fn find_releasable_buildpacks(
    starting_dir: &Path,
) -> Result<Vec<PathBuf>, FindReleasableBuildpacksError> {
    find_buildpack_dirs(starting_dir)
        .map(|results| {
            results
                .into_iter()
                .filter(|dir| dir.join("CHANGELOG.md").exists())
                .collect()
        })
        .map_err(|e| FindReleasableBuildpacksError(starting_dir.to_path_buf(), e))
}
#[derive(Debug)]
pub(crate) struct FindReleasableBuildpacksError(PathBuf, ignore::Error);

impl Display for FindReleasableBuildpacksError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let path = &self.0;
        let error = &self.1;
        write!(
            f,
            "I/O error while finding buildpacks\nPath: {}\nError: {error}",
            path.display()
        )
    }
}

pub(crate) fn read_buildpack_descriptor(
    dir: &Path,
) -> Result<BuildpackDescriptor, ReadBuildpackDescriptorError> {
    let buildpack_path = dir.join("buildpack.toml");
    read_toml_file::<BuildpackDescriptor>(&buildpack_path)
        .map_err(|e| ReadBuildpackDescriptorError(buildpack_path, e))
}

#[derive(Debug)]
pub(crate) struct ReadBuildpackDescriptorError(PathBuf, TomlFileError);

impl Display for ReadBuildpackDescriptorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let buildpack_path = &self.0;
        let error = &self.1;
        match error {
            TomlFileError::IoError(source) => {
                write!(
                    f,
                    "Failed to read buildpack\nPath: {}\nError: {source}",
                    buildpack_path.display()
                )
            }
            TomlFileError::TomlDeserializationError(source) => {
                write!(
                    f,
                    "Failed to deserialize buildpack\nPath: {}\nError: {source}",
                    buildpack_path.display()
                )
            }
            TomlFileError::TomlSerializationError(source) => {
                write!(
                    f,
                    "Failed to serialize buildpack\nPath: {}\nError: {source}",
                    buildpack_path.display()
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::buildpacks::read_image_repository_metadata;
    use libcnb_data::buildpack::BuildpackDescriptor;

    #[test]
    fn test_read_image_repository_metadata() {
        let data = r#"
api = "0.9"

[buildpack]
id = "foo/bar"
version = "0.0.1"

[[stacks]]
id = "*"

[metadata.release.image]
repository = "repository value"
"#;

        let buildpack_descriptor = toml::from_str::<BuildpackDescriptor>(data).unwrap();
        assert_eq!(
            read_image_repository_metadata(&buildpack_descriptor),
            Some("repository value".to_string())
        );
    }

    #[test]
    fn test_read_image_repository_metadata_empty() {
        let data = r#"
api = "0.9"

[buildpack]
id = "foo/bar"
version = "0.0.1"

[[stacks]]
id = "*"
"#;

        let buildpack_descriptor = toml::from_str::<BuildpackDescriptor>(data).unwrap();
        assert_eq!(read_image_repository_metadata(&buildpack_descriptor), None);
    }
}
