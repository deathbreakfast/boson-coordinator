//! Fail-closed admin auth defaults for the coordinator Axum bridge.
//!
//! Upstream [`boson_axum`] only enforces admin auth when `BOSON_REQUIRE_ADMIN_AUTH` is set.
//! Product mounts via this crate default to **require admin auth** unless
//! [`OPEN_LAB_MODE_ENV`] opts into an open lab mount.

/// Environment variable: when `1`/`true`/`yes`, opt into an **open lab** mount (no admin verifier).
///
/// Without this, [`require_admin_auth_default`] is `true` and unauthenticated requests are rejected.
pub const OPEN_LAB_MODE_ENV: &str = "BOSON_OPEN_LAB_MODE";

/// Parse an open-lab flag string (`1` / `true` / `yes`, case-insensitive).
#[must_use]
pub fn parse_open_lab_mode(value: &str) -> bool {
    let v = value.trim().to_ascii_lowercase();
    matches!(v.as_str(), "1" | "true" | "yes")
}

/// Whether [`OPEN_LAB_MODE_ENV`] is set to a truthy value.
#[must_use]
pub fn open_lab_mode_from_env() -> bool {
    std::env::var(OPEN_LAB_MODE_ENV).is_ok_and(|v| parse_open_lab_mode(&v))
}

/// Whether require-admin is the default for new coordinator [`super::BosonState`] mounts.
///
/// **`true` by default (fail closed).** Only [`OPEN_LAB_MODE_ENV`] opts out.
#[must_use]
pub fn require_admin_auth_default() -> bool {
    !open_lab_mode_from_env()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_open_lab_mode_truthy_and_falsy() {
        for v in ["1", "true", "YES", " True "] {
            assert!(parse_open_lab_mode(v), "expected truthy: {v}");
        }
        for v in ["0", "false", "no", ""] {
            assert!(!parse_open_lab_mode(v), "expected falsy: {v}");
        }
    }

    #[test]
    fn require_admin_auth_default_is_fail_closed_without_open_lab() {
        std::env::remove_var(OPEN_LAB_MODE_ENV);
        assert!(
            require_admin_auth_default(),
            "default mount must require admin auth"
        );
    }

    #[test]
    fn require_admin_auth_default_allows_open_lab_env() {
        std::env::set_var(OPEN_LAB_MODE_ENV, "1");
        assert!(
            !require_admin_auth_default(),
            "open lab env must disable require-admin default"
        );
        std::env::remove_var(OPEN_LAB_MODE_ENV);
    }
}
