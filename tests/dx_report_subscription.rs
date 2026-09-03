//! Test that verifies dx report subscription completed successfully.
//!
//! Checks that:
//! 1. reports.dx file exists at project root
//! 2. dx report list . shows "subscribed" status
//! 3. Project key matches collision-resistant hex format (32+ chars)

use std::path::Path;
use std::process::Command;

#[test]
fn dx_report_subscription_configured() {
    let project_root = "/Users/alexwaldmann/Desktop/rui";

    // Check 1: reports.dx exists
    let reports_dx = Path::new(project_root).join("reports.dx");
    assert!(
        reports_dx.exists(),
        "reports.dx file not found at project root"
    );

    // Check 2: dx report list . shows "subscribed" (not "not subscribed")
    let output = Command::new("dx")
        .arg("report")
        .arg("list")
        .arg(".")
        .current_dir(project_root)
        .output()
        .expect("Failed to run dx report list command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("subscribed"),
        "dx report list output does not contain 'subscribed': {}",
        stdout
    );
    assert!(
        !stdout.contains("not subscribed"),
        "dx report list shows 'not subscribed': {}",
        stdout
    );

    // Check 3: Project key format matches pattern (hex string, 32+ chars)
    // Extract project key from "subscribed to `<key>`" pattern
    if let Some(start) = stdout.find("subscribed to `") {
        let after_prefix = &stdout[start + "subscribed to `".len()..];
        if let Some(end) = after_prefix.find('`') {
            let project_key = &after_prefix[..end];
            assert!(
                project_key.len() >= 32,
                "Project key too short (expected >=32 chars, got {}): {}",
                project_key.len(),
                project_key
            );
            assert!(
                project_key.chars().all(|c| c.is_ascii_hexdigit()),
                "Project key contains non-hex characters: {}",
                project_key
            );
        }
    }
}
