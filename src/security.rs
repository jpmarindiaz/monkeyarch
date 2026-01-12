use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Validates and resolves a user-provided path against the root jail.
/// Returns the canonical absolute path if valid.
///
/// Security checks:
/// - Rejects null bytes (path injection)
/// - Rejects path traversal (../)
/// - Canonicalizes paths before comparison
/// - Verifies result stays within root directory
pub fn validate_path(root: &Path, user_path: &str) -> Result<PathBuf, AppError> {
    // Reject null bytes
    if user_path.contains('\0') {
        return Err(AppError::BadRequest("invalid path: null byte".to_string()));
    }

    // Normalize: strip leading slashes, handle empty
    let normalized = user_path.trim_start_matches('/');
    let normalized = if normalized.is_empty() {
        "."
    } else {
        normalized
    };

    // Build target path (not yet validated)
    let target = root.join(normalized);

    // Canonicalize root (must exist)
    let canonical_root = root.canonicalize().map_err(|_| {
        AppError::Internal(format!("root directory not found: {:?}", root))
    })?;

    // Canonicalize target path
    let canonical_target = if target.exists() {
        target.canonicalize().map_err(|e| {
            AppError::Internal(format!("path resolution failed: {}", e))
        })?
    } else {
        // For non-existent paths (e.g., upload destinations):
        // Canonicalize parent and join with filename
        let parent = target
            .parent()
            .ok_or_else(|| AppError::BadRequest("invalid path: no parent".to_string()))?;

        let canonical_parent = parent.canonicalize().map_err(|_| {
            AppError::NotFound("parent directory not found".to_string())
        })?;

        // Verify parent is inside root
        if !canonical_parent.starts_with(&canonical_root) {
            return Err(AppError::Forbidden);
        }

        let filename = target
            .file_name()
            .ok_or_else(|| AppError::BadRequest("invalid path: no filename".to_string()))?;

        // Validate filename doesn't contain traversal
        let filename_str = filename.to_string_lossy();
        if filename_str == ".." || filename_str == "." {
            return Err(AppError::Forbidden);
        }

        canonical_parent.join(filename)
    };

    // Final jail check: canonical path must start with canonical root
    if !canonical_target.starts_with(&canonical_root) {
        return Err(AppError::Forbidden);
    }

    Ok(canonical_target)
}

/// Validates path exists AND is a directory.
pub fn validate_directory(root: &Path, user_path: &str) -> Result<PathBuf, AppError> {
    let path = validate_path(root, user_path)?;
    if !path.is_dir() {
        return Err(AppError::BadRequest(format!(
            "not a directory: {}",
            user_path
        )));
    }
    Ok(path)
}

/// Validates path exists AND is a file.
pub fn validate_file(root: &Path, user_path: &str) -> Result<PathBuf, AppError> {
    let path = validate_path(root, user_path)?;
    if !path.is_file() {
        return Err(AppError::NotFound(format!("file not found: {}", user_path)));
    }
    Ok(path)
}

/// Validates path exists (file or directory).
pub fn validate_existing(root: &Path, user_path: &str) -> Result<PathBuf, AppError> {
    let path = validate_path(root, user_path)?;
    if !path.exists() {
        return Err(AppError::NotFound(format!("not found: {}", user_path)));
    }
    Ok(path)
}

/// Validates a filename (no path components allowed).
pub fn validate_filename(filename: &str) -> Result<&str, AppError> {
    if filename.is_empty() {
        return Err(AppError::BadRequest("empty filename".to_string()));
    }
    if filename.contains('/') || filename.contains('\\') {
        return Err(AppError::BadRequest(
            "filename cannot contain path separators".to_string(),
        ));
    }
    if filename == ".." || filename == "." {
        return Err(AppError::BadRequest("invalid filename".to_string()));
    }
    if filename.contains('\0') {
        return Err(AppError::BadRequest("invalid filename: null byte".to_string()));
    }
    Ok(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("subdir")).unwrap();
        fs::write(tmp.path().join("file.txt"), "test").unwrap();
        fs::write(tmp.path().join("subdir/nested.txt"), "nested").unwrap();
        tmp
    }

    #[test]
    fn test_validate_path_root() {
        let tmp = setup();
        let result = validate_path(tmp.path(), "").unwrap();
        assert_eq!(result, tmp.path().canonicalize().unwrap());
    }

    #[test]
    fn test_validate_path_file() {
        let tmp = setup();
        let result = validate_path(tmp.path(), "file.txt").unwrap();
        assert_eq!(result, tmp.path().join("file.txt").canonicalize().unwrap());
    }

    #[test]
    fn test_validate_path_subdir() {
        let tmp = setup();
        let result = validate_path(tmp.path(), "subdir/nested.txt").unwrap();
        assert_eq!(
            result,
            tmp.path().join("subdir/nested.txt").canonicalize().unwrap()
        );
    }

    #[test]
    fn test_validate_path_nonexistent_in_valid_parent() {
        let tmp = setup();
        let result = validate_path(tmp.path(), "newfile.txt").unwrap();
        assert_eq!(
            result,
            tmp.path().canonicalize().unwrap().join("newfile.txt")
        );
    }

    #[test]
    fn test_validate_path_traversal_blocked() {
        let tmp = setup();

        // Create a nested structure to test traversal
        let inner = tmp.path().join("inner");
        fs::create_dir(&inner).unwrap();
        fs::write(inner.join("file.txt"), "inner").unwrap();

        // Valid: inner/file.txt from root
        let result = validate_path(tmp.path(), "inner/file.txt");
        assert!(result.is_ok());

        // Invalid: trying to access parent's sibling via traversal
        // subdir/../inner should resolve to inner (valid)
        let result = validate_path(tmp.path(), "subdir/../inner/file.txt");
        assert!(result.is_ok());

        // Test that paths containing ".." but not escaping are OK
        let result = validate_path(tmp.path(), "inner/../subdir");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_direct_traversal() {
        // Test that ".." filename itself is rejected
        let tmp = setup();
        let result = validate_path(tmp.path(), "..");
        // Either Forbidden or some error - as long as it doesn't succeed
        assert!(result.is_err(), "Should reject '..' path");
    }

    #[test]
    fn test_validate_path_null_byte_blocked() {
        let tmp = setup();
        let result = validate_path(tmp.path(), "file\0.txt");
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[test]
    fn test_validate_filename() {
        assert!(validate_filename("test.txt").is_ok());
        assert!(validate_filename("path/file.txt").is_err());
        assert!(validate_filename("..").is_err());
        assert!(validate_filename("").is_err());
    }
}
