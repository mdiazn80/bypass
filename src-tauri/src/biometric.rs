use std::process::Command;

/// Check if biometric authentication (Touch ID) is available on this machine.
pub fn is_available() -> bool {
    if !cfg!(target_os = "macos") {
        return false;
    }

    let script = r#"
import LocalAuthentication
let context = LAContext()
var error: NSError?
let available = context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)
if available { print("true") } else { print("false") }
"#;

    let output = Command::new("swift")
        .arg("-e")
        .arg(script)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout).trim() == "true"
        }
        _ => false,
    }
}

/// Authenticate using biometrics (Touch ID on macOS).
/// Returns Ok(()) on success, Err with message on failure.
/// On non-macOS or when biometrics are unavailable, returns Ok(()) (skip).
pub fn authenticate(reason: &str) -> Result<(), String> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }

    if !is_available() {
        return Ok(());
    }

    let script = format!(
        r#"
import Foundation
import LocalAuthentication

let context = LAContext()
var error: NSError?

guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) else {{
    fputs("Biometrics not available", stderr)
    exit(1)
}}

let semaphore = DispatchSemaphore(value: 0)
var authSuccess = false

context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: "{}") {{ success, authError in
    authSuccess = success
    if !success {{
        if let e = authError {{
            fputs(e.localizedDescription, stderr)
        }}
    }}
    semaphore.signal()
}}

semaphore.wait()
exit(authSuccess ? 0 : 2)
"#,
        reason.replace('\\', "\\\\").replace('"', "\\\"")
    );

    let output = Command::new("swift")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to run biometric auth: {}", e))?;

    match output.status.code() {
        Some(0) => Ok(()),
        Some(2) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Biometric authentication failed: {}", stderr.trim()))
        }
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Biometric check error: {}", stderr.trim()))
        }
    }
}
