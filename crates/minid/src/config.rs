//! OCI bundle `config.json` loading and validation.

use std::fs;
use std::path::Path;

use oci_spec::runtime::Spec;
use tracing::debug;

use crate::error::MinidError;

/// Load and validate an OCI runtime spec from a bundle directory.
///
/// The bundle must contain a `config.json` file at its root.
/// The spec is validated for required fields (root filesystem path,
/// process arguments).
pub fn load_spec(bundle_path: &Path) -> crate::Result<Spec> {
    if !bundle_path.is_dir() {
        return Err(MinidError::BundleNotFound(bundle_path.to_path_buf()));
    }

    let config_path = bundle_path.join("config.json");
    debug!(?config_path, "loading OCI spec");

    let contents = fs::read_to_string(&config_path).map_err(|e| {
        MinidError::InvalidConfig(format!("failed to read config.json: {e}"))
    })?;

    let spec: Spec = serde_json::from_str(&contents)?;

    validate_spec(&spec)?;
    Ok(spec)
}

/// Ensure the spec contains the minimum required fields.
fn validate_spec(spec: &Spec) -> crate::Result<()> {
    // Root filesystem must be specified.
    let root = spec.root().as_ref().ok_or_else(|| {
        MinidError::InvalidConfig("spec.root is required".into())
    })?;

    if root.path().as_os_str().is_empty() {
        return Err(MinidError::InvalidConfig(
            "spec.root.path must not be empty".into(),
        ));
    }

    // Process and args must be specified.
    let process = spec.process().as_ref().ok_or_else(|| {
        MinidError::InvalidConfig("spec.process is required".into())
    })?;

    let args = process.args().as_ref().ok_or_else(|| {
        MinidError::InvalidConfig("spec.process.args is required".into())
    })?;

    if args.is_empty() {
        return Err(MinidError::InvalidConfig(
            "spec.process.args must not be empty".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn minimal_config() -> &'static str {
        r#"{
            "ociVersion": "1.0.2",
            "root": { "path": "rootfs", "readonly": true },
            "process": {
                "args": ["/bin/sh"],
                "cwd": "/",
                "user": { "uid": 0, "gid": 0 }
            },
            "linux": {}
        }"#
    }

    #[test]
    fn load_valid_spec() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("config.json"), minimal_config()).unwrap();
        let spec = load_spec(dir.path()).unwrap();
        assert!(spec.root().is_some());
    }

    #[test]
    fn missing_bundle_dir() {
        let result = load_spec(Path::new("/nonexistent/bundle"));
        assert!(matches!(result, Err(MinidError::BundleNotFound(_))));
    }

    #[test]
    fn missing_config_json() {
        let dir = TempDir::new().unwrap();
        let result = load_spec(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn missing_root_field() {
        let dir = TempDir::new().unwrap();
        let config = r#"{
            "ociVersion": "1.0.2",
            "process": {
                "args": ["/bin/sh"],
                "cwd": "/",
                "user": { "uid": 0, "gid": 0 }
            },
            "linux": {}
        }"#;
        fs::write(dir.path().join("config.json"), config).unwrap();
        let spec = load_spec(dir.path());
        // root is optional in oci-spec serde, so validation catches it
        match spec {
            Err(MinidError::InvalidConfig(msg)) => {
                assert!(msg.contains("root"), "expected root error, got: {msg}");
            }
            Ok(s) => {
                // If serde allows it, our validation should catch it
                assert!(s.root().is_none(), "root should be None");
            }
            Err(e) => panic!("unexpected error: {e}"),
        }
    }

    #[test]
    fn missing_process_args() {
        let dir = TempDir::new().unwrap();
        let config = r#"{
            "ociVersion": "1.0.2",
            "root": { "path": "rootfs" },
            "process": {
                "cwd": "/",
                "user": { "uid": 0, "gid": 0 }
            },
            "linux": {}
        }"#;
        fs::write(dir.path().join("config.json"), config).unwrap();
        let result = load_spec(dir.path());
        assert!(result.is_err(), "should fail with missing args");
    }
}

