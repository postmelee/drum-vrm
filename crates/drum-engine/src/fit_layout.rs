//! A selected conventional right-handed five-piece arrangement is a geometric
//! contract, not a side effect of independently reachable instrument targets.
//! Distances scale with measured playing radii. These bounds define this layout
//! family; they are not universal rules for all drummers or kit configurations.
use crate::model::{Instrument, KitPiece};

pub(crate) fn violations(profile: &crate::rig::RigProfile, kit: &[KitPiece]) -> Vec<String> {
    use Instrument::*;
    let one = |instrument| kit.iter().find(|piece| piece.instrument == instrument);
    let center = |p: &KitPiece| crate::equipment::surface(p).center;
    let radius = |p: &KitPiece| crate::equipment::surface(p).radius;
    let mut out = Vec::new();
    let Some(snare) = one(Snare) else {
        return out;
    };
    let s = center(snare);
    let sr = radius(snare);
    // The snare is between the knees in this selected layout. A local solver
    // cannot hide an inaccessible centred posture by moving it to one flank.
    if (s[0] - profile.pelvis[0]).abs() > sr * 0.5 + 0.02 {
        out.push("snare-body-center".into());
    }
    let high = one(TomHigh);
    let mid = one(TomMid);
    if let (Some(high), Some(mid)) = (high, mid) {
        let h = center(high);
        let m = center(mid);
        let sum = radius(high) + radius(mid);
        // Rack toms occupy one shallow row, ordered left to right. A front/back
        // escape or putting a rack tom below the snare is not a valid repair.
        if m[0] - h[0] < sum * 0.7
            || (m[2] - h[2]).abs() > sum * 0.25 + 0.02
            || (m[1] - h[1]).abs() > sum * 0.25 + 0.02
        {
            out.push("rack-row".into());
        }
    }
    for p in [high, mid].into_iter().flatten() {
        let c = center(p);
        let sum = sr + radius(p);
        if c[2] - s[2] < sum * 0.65 || c[1] - s[1] < sum * 0.08 {
            out.push(format!("{}-ahead-of-snare", p.id));
        }
    }
    if let Some(floor) = one(TomLow) {
        let f = center(floor);
        let sum = sr + radius(floor);
        if f[0] - s[0] < sum * 0.7
            || (f[2] - s[2]).abs() > sum * 0.55 + 0.03
            || (f[1] - s[1]).abs() > sum * 0.3 + 0.02
        {
            out.push("floor-flank".into());
        }
    }
    if let Some(hat) = one(Hihat) {
        let h = center(hat);
        if s[0] - h[0] < (sr + radius(hat)) * 0.65 || h[1] < s[1] + 0.015 {
            out.push("hihat-left-above-snare".into());
        }
    }
    if let Some(ride) = one(Ride) {
        let r = center(ride);
        if r[0] - s[0] < (sr + radius(ride)) * 0.65
            || r[2] < s[2] - radius(ride) * 0.25
            || r[1] < s[1] + 0.04
        {
            out.push("ride-right-sector".into());
        }
    }
    // Left/right crash identities are semantic kit slots, not avatar identities.
    for p in kit.iter().filter(|p| p.instrument == Crash) {
        let c = center(p);
        let left = p.id != "crashRight";
        if (if left { s[0] - c[0] } else { c[0] - s[0] }) < (sr + radius(p)) * 0.65
            || c[2] < s[2]
            || c[1] < s[1] + 0.10
        {
            out.push(format!("{}-outer-sector", p.id));
        }
    }
    out
}
