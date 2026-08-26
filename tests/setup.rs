//! Verify project setup and hooks are properly configured.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[test]
fn pre_commit_hook_exists_and_is_executable() {
    let hook_path = PathBuf::from(".git/hooks/pre-commit");

    // Check that the hook file exists
    assert!(
        hook_path.exists(),
        "pre-commit hook not found at {}",
        hook_path.display()
    );

    // Check that it's a file
    assert!(
        hook_path.is_file(),
        "pre-commit hook is not a regular file: {}",
        hook_path.display()
    );

    // Check that it's executable
    let metadata = fs::metadata(&hook_path).expect("failed to read hook metadata");
    let permissions = metadata.permissions();
    let mode = permissions.mode();
    let is_executable = (mode & 0o111) != 0;

    assert!(
        is_executable,
        "pre-commit hook is not executable (mode: {:o})",
        mode
    );
}

#[test]
fn pre_commit_hook_runs_successfully_when_code_is_clean() {
    use std::process::Command;

    let output = Command::new("bash")
        .arg(".git/hooks/pre-commit")
        .output()
        .expect("failed to execute pre-commit hook");

    assert!(
        output.status.success(),
        "pre-commit hook failed with: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
