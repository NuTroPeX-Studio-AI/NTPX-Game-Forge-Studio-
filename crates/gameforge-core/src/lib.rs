#![forbid(unsafe_code)]

/// Stable identity for the current `GameForge` foundation build.
pub const BUILD_ID: &str = "NTPX-GFΩ-G0-v0.1.0-dev.1-M01-B000001";
pub const VERSION: &str = "0.1.0-dev.1";
pub const MILESTONE: &str = "M01";
pub const BUILD_NUMBER: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildIdentity {
    pub product: &'static str,
    pub version: &'static str,
    pub milestone: &'static str,
    pub build_number: u64,
    pub build_id: &'static str,
}

impl BuildIdentity {
    #[must_use]
    pub const fn current() -> Self {
        Self {
            product: "NTPX GameForge Ω∞",
            version: VERSION,
            milestone: MILESTONE,
            build_number: BUILD_NUMBER,
            build_id: BUILD_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_build_identity_is_stable() {
        let identity = BuildIdentity::current();
        assert_eq!(identity.version, VERSION);
        assert_eq!(identity.milestone, "M01");
        assert_eq!(identity.build_number, 1);
        assert_eq!(identity.build_id, BUILD_ID);
    }
}
