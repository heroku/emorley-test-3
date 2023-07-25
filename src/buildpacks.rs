use libcnb_data::buildpack::BuildpackDescriptor;
use libcnb_package::{find_buildpack_dirs, GenericMetadata};
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
    buildpack_descriptor: &BuildpackDescriptor<GenericMetadata>,
) -> Option<String> {
    let metadata = match buildpack_descriptor {
        BuildpackDescriptor::Single(descriptor) => &descriptor.metadata,
        BuildpackDescriptor::Meta(descriptor) => &descriptor.metadata,
    };

    #[allow(clippy::redundant_closure_for_method_calls)]
    metadata
        .as_ref()
        .and_then(|metadata| metadata.get("release").and_then(|value| value.as_table()))
        .and_then(|release| {
            release
                .get("image")
                // TODO: remove this once each buildpack.toml metadata is updated to
                //       replace [metadata.release.docker] with [metadata.release.image]
                .or_else(|| release.get("docker"))
                .and_then(|value| value.as_table())
        })
        .and_then(|docker| docker.get("repository").and_then(|value| value.as_str()))
        .map(|value| value.to_string())
}

pub(crate) fn find_releasable_buildpacks(starting_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    find_buildpack_dirs(starting_dir, &[starting_dir.join("target")]).map(|results| {
        results
            .into_iter()
            .filter(|dir| dir.join("CHANGELOG.md").exists())
            .collect()
    })
}

#[cfg(test)]
mod test {
    use crate::buildpacks::read_image_repository_metadata;
    use libcnb_data::buildpack::BuildpackDescriptor;
    use libcnb_package::GenericMetadata;

    #[test]
    fn test_read_image_repository_metadata_deprecated() {
        let data = r#"
api = "0.9"

[buildpack]
id = "foo/bar"
version = "0.0.1"

[[stacks]]
id = "*"

[metadata.release.docker]
repository = "deprecated repository value"
"#;

        let buildpack_descriptor =
            toml::from_str::<BuildpackDescriptor<GenericMetadata>>(data).unwrap();
        assert_eq!(
            read_image_repository_metadata(&buildpack_descriptor),
            Some("deprecated repository value".to_string())
        );
    }

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

        let buildpack_descriptor =
            toml::from_str::<BuildpackDescriptor<GenericMetadata>>(data).unwrap();
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

        let buildpack_descriptor =
            toml::from_str::<BuildpackDescriptor<GenericMetadata>>(data).unwrap();
        assert_eq!(read_image_repository_metadata(&buildpack_descriptor), None);
    }
}
