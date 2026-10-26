//! Shared rendered/checking dimensions, in metres. This is an engineering
//! approximation with a point terminal, not a measured commercial stick model.
use std::sync::OnceLock;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StickProfile {
    pub butt_length: f64,
    /// Increasing distance back from the contact tip, and the radial extent.
    pub terminal: Vec<[f64; 2]>,
}

pub fn profile() -> &'static StickProfile {
    static PROFILE: OnceLock<StickProfile> = OnceLock::new();
    PROFILE.get_or_init(|| {
        serde_json::from_str(include_str!("../../../shared/stick-profile.json"))
            .expect("valid shared stick dimensions")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_terminal_is_monotonic_and_ends_at_contact() {
        let p = profile();
        assert!(p.butt_length > 0.);
        assert_eq!(p.terminal[0], [0., 0.]);
        assert!(p
            .terminal
            .windows(2)
            .all(|pair| pair[0][0] < pair[1][0] && pair[0][1] <= pair[1][1]));
        assert!(p
            .terminal
            .iter()
            .flatten()
            .all(|n| n.is_finite() && *n >= 0.));
    }
}
