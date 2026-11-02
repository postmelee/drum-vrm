//! Bounded deterministic fitting of one fixed physical hand-curve basis.
//! Positive residuals are violations. Numerical convergence here is not physical
//! acceptance: the caller owns adaptive collocation and the dense final gate.

pub(crate) type Parameters = [f64; 8];
pub(crate) const MAX_COEFFICIENT: f64 = std::f64::consts::PI;
const VARIABLES: usize = 8;
const FIT_ITERATIONS: usize = 48;
const MOTION_ITERATIONS: usize = 16;
const BACKTRACKS: i32 = 12;
const PROJECTION_PASSES: usize = 12;
const DIFFERENCE: f64 = 1e-5;
const DAMPING: f64 = 1e-5;
const TRUST_STEP: f64 = 0.15;
const CONVERGENCE: f64 = 1e-24;

/// Fit violated constraints, then reduce the caller's physical-motion cost while
/// retaining numerical feasibility. Failed fits return their best bounded
/// parameters; they are never labelled feasible or accepted by this module.
pub(crate) fn optimize(
    initial: Parameters,
    residuals: impl Fn(&Parameters) -> Vec<f64>,
    motion_cost: impl Fn(&Parameters) -> f64,
) -> Parameters {
    optimize_active(initial, residuals, motion_cost, VARIABLES)
}

/// First try plane/twist corrections while retaining the original local
/// forearm curve. Frozen coefficients are zero even for a nonzero seed, and
/// never consume finite-difference queries in either optimization stage.
pub(crate) fn optimize_plane_twist(
    mut initial: Parameters,
    residuals: impl Fn(&Parameters) -> Vec<f64>,
    motion_cost: impl Fn(&Parameters) -> f64,
) -> Parameters {
    initial[4..].fill(0.);
    optimize_active(initial, residuals, motion_cost, 4)
}

fn optimize_active(
    initial: Parameters,
    residuals: impl Fn(&Parameters) -> Vec<f64>,
    motion_cost: impl Fn(&Parameters) -> f64,
    active: usize,
) -> Parameters {
    let mut value = initial.map(|v| {
        if v.is_finite() {
            v.clamp(-MAX_COEFFICIENT, MAX_COEFFICIENT)
        } else {
            0.
        }
    });
    let mut residual = residuals(&value);
    let Some(mut score) = violation(&residual) else {
        return value;
    };
    for _ in 0..FIT_ITERATIONS {
        if score <= CONVERGENCE {
            break;
        }
        let Some(jacobian) = jacobian(&value, &residual, &residuals, active) else {
            break;
        };
        let mut matrix = [[0.; VARIABLES]; VARIABLES];
        let mut rhs = [0.; VARIABLES];
        for i in 0..VARIABLES {
            matrix[i][i] = DAMPING;
            for (k, &r) in residual.iter().enumerate().filter(|(_, r)| **r > 0.) {
                rhs[i] -= jacobian[i][k] * r;
                for (j, row) in jacobian.iter().enumerate() {
                    matrix[i][j] += jacobian[i][k] * row[k];
                }
            }
        }
        let Some(mut step) = solve(matrix, rhs) else {
            break;
        };
        limit_step(&mut step);
        let mut improved = false;
        for n in 0..BACKTRACKS {
            let candidate = trial(value, step, 0.5_f64.powi(n));
            let next = residuals(&candidate);
            if next.len() != residual.len() {
                return value;
            }
            let Some(next_score) = violation(&next) else {
                continue;
            };
            if next_score < score - CONVERGENCE {
                value = candidate;
                residual = next;
                score = next_score;
                improved = true;
                break;
            }
        }
        if !improved {
            break;
        }
    }
    if score > CONVERGENCE {
        return value;
    }
    refine_motion(value, residual, &residuals, &motion_cost, active)
}

fn refine_motion(
    mut value: Parameters,
    mut residual: Vec<f64>,
    residuals: &impl Fn(&Parameters) -> Vec<f64>,
    motion_cost: &impl Fn(&Parameters) -> f64,
    active: usize,
) -> Parameters {
    let mut cost = motion_cost(&value);
    if !cost.is_finite() {
        return value;
    }
    for _ in 0..MOTION_ITERATIONS {
        let Some(jacobian) = jacobian(&value, &residual, residuals, active) else {
            break;
        };
        let mut step = [0.; VARIABLES];
        for (i, direction) in step.iter_mut().enumerate().take(active) {
            let (lower, upper, width) = probes(value, i);
            let (a, b) = (motion_cost(&lower), motion_cost(&upper));
            if !a.is_finite() || !b.is_finite() {
                return value;
            }
            *direction = -(b - a) / width;
        }
        if step.iter().any(|v| !v.is_finite()) {
            break;
        }
        limit_step(&mut step);
        // Project the proposed descent onto the linearized constraint half
        // spaces. This permits motion along an active boundary instead of
        // discarding every direction because one coefficient cannot decrease.
        for _ in 0..PROJECTION_PASSES {
            for (k, &r) in residual.iter().enumerate() {
                if !r.is_finite() {
                    continue;
                }
                let normal: Parameters = std::array::from_fn(|i| jacobian[i][k]);
                let norm = normal.iter().map(|v| v * v).sum::<f64>();
                let excess = r + dot(normal, step);
                if excess > 0. && norm > 1e-20 {
                    for i in 0..VARIABLES {
                        step[i] -= normal[i] * excess / norm;
                    }
                }
            }
            for i in 0..VARIABLES {
                step[i] = step[i].clamp(-MAX_COEFFICIENT - value[i], MAX_COEFFICIENT - value[i]);
            }
            limit_step(&mut step);
        }
        if step.iter().any(|v| !v.is_finite()) || dot(step, step) <= 1e-24 {
            break;
        }
        let mut improved = false;
        for n in 0..BACKTRACKS {
            let candidate = trial(value, step, 0.5_f64.powi(n));
            let next = residuals(&candidate);
            if next.len() != residual.len() {
                return value;
            }
            if !violation(&next).is_some_and(|score| score <= CONVERGENCE) {
                continue;
            }
            let next_cost = motion_cost(&candidate);
            if next_cost.is_finite() && next_cost < cost - 1e-12 * cost.abs().max(1.) {
                value = candidate;
                residual = next;
                cost = next_cost;
                improved = true;
                break;
            }
        }
        if !improved {
            break;
        }
    }
    value
}

fn violation(values: &[f64]) -> Option<f64> {
    // Negative infinity is an absent obstacle, not a derivative to evaluate.
    if values.iter().any(|v| v.is_nan() || *v == f64::INFINITY) {
        return None;
    }
    let result = values.iter().map(|v| v.max(0.).powi(2)).sum::<f64>();
    result.is_finite().then_some(result)
}

fn probes(value: Parameters, index: usize) -> (Parameters, Parameters, f64) {
    let (mut lower, mut upper) = (value, value);
    lower[index] = (lower[index] - DIFFERENCE).max(-MAX_COEFFICIENT);
    upper[index] = (upper[index] + DIFFERENCE).min(MAX_COEFFICIENT);
    (lower, upper, upper[index] - lower[index])
}

fn jacobian(
    value: &Parameters,
    current: &[f64],
    residuals: &impl Fn(&Parameters) -> Vec<f64>,
    active: usize,
) -> Option<[Vec<f64>; VARIABLES]> {
    let mut result: [Vec<f64>; VARIABLES] = std::array::from_fn(|_| Vec::new());
    for (i, column) in result.iter_mut().enumerate() {
        if i >= active {
            column.resize(current.len(), 0.);
            continue;
        }
        let (lower, upper, width) = probes(*value, i);
        let (a, b) = (residuals(&lower), residuals(&upper));
        if a.len() != current.len() || b.len() != current.len() {
            return None;
        }
        for ((&r, a), b) in current.iter().zip(a).zip(b) {
            let derivative = if r == f64::NEG_INFINITY {
                0.
            } else {
                (b - a) / width
            };
            if !derivative.is_finite() {
                return None;
            }
            column.push(derivative);
        }
    }
    Some(result)
}

fn dot(a: Parameters, b: Parameters) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

fn limit_step(step: &mut Parameters) {
    let length = step
        .iter()
        .fold(0_f64, |length, value| length.hypot(*value));
    if length > TRUST_STEP {
        for value in step {
            *value *= TRUST_STEP / length;
        }
    }
}

fn trial(value: Parameters, step: Parameters, scale: f64) -> Parameters {
    std::array::from_fn(|i| (value[i] + step[i] * scale).clamp(-MAX_COEFFICIENT, MAX_COEFFICIENT))
}

fn solve(mut matrix: [[f64; VARIABLES]; VARIABLES], mut rhs: Parameters) -> Option<Parameters> {
    if matrix.iter().flatten().chain(&rhs).any(|v| !v.is_finite()) {
        return None;
    }
    for i in 0..VARIABLES {
        let pivot =
            (i..VARIABLES).max_by(|&a, &b| matrix[a][i].abs().total_cmp(&matrix[b][i].abs()))?;
        matrix.swap(i, pivot);
        rhs.swap(i, pivot);
        let divisor = matrix[i][i];
        if divisor.abs() < 1e-20 {
            return None;
        }
        for entry in &mut matrix[i][i..] {
            *entry /= divisor;
        }
        rhs[i] /= divisor;
        let pivot = matrix[i];
        for j in 0..VARIABLES {
            if i == j {
                continue;
            }
            let factor = matrix[j][i];
            for (entry, pivot) in matrix[j][i..].iter_mut().zip(&pivot[i..]) {
                *entry -= factor * pivot;
            }
            rhs[j] -= factor * rhs[i];
        }
    }
    rhs.iter().all(|v| v.is_finite()).then_some(rhs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fits_coupled_nonlinear_constraints_deterministically() {
        let constraints = |p: &Parameters| {
            vec![
                (p[0] - 0.6).powi(2) + (p[1] + 0.8).powi(2) - 0.0025,
                0.2 - p[2],
            ]
        };
        let a = optimize([0.; 8], constraints, |_| 0.);
        let b = optimize([0.; 8], constraints, |_| 0.);
        assert_eq!(a, b);
        assert!(violation(&constraints(&a)).unwrap() <= CONVERGENCE);
        assert!(a
            .iter()
            .all(|v| v.is_finite() && v.abs() <= MAX_COEFFICIENT));
    }

    #[test]
    fn motion_refinement_follows_a_feasible_boundary() {
        let mut initial = [0.; 8];
        initial[0] = 0.6;
        initial[1] = 1.;
        let constraints = |p: &Parameters| vec![0.6 - p[0], f64::NEG_INFINITY];
        let motion = |p: &Parameters| (p[0] - 0.2).powi(2) + (p[1] + 0.3).powi(2);
        let result = optimize(initial, constraints, motion);
        assert!(violation(&constraints(&result)).unwrap() <= CONVERGENCE);
        assert!((result[0] - 0.6).abs() < 1e-10);
        assert!((result[1] + 0.3).abs() < 0.01);
        assert!(motion(&result) < motion(&initial) * 0.1);
    }

    #[test]
    fn plane_twist_subspace_freezes_forearm_and_skips_inactive_queries() {
        let solve = |restricted: bool| {
            let residual_calls = std::cell::Cell::new(0);
            let motion_calls = std::cell::Cell::new(0);
            let constraints = |p: &Parameters| {
                if restricted {
                    assert_eq!(p[4..], [0.; 4]);
                }
                residual_calls.set(residual_calls.get() + 1);
                vec![0.4 - p[0]]
            };
            let motion = |p: &Parameters| {
                if restricted {
                    assert_eq!(p[4..], [0.; 4]);
                }
                motion_calls.set(motion_calls.get() + 1);
                (p[0] - 0.1).powi(2) + (p[1] + 0.3).powi(2)
            };
            let mut initial = [0.; 8];
            let result = if restricted {
                initial[4..].fill(1.);
                optimize_plane_twist(initial, constraints, motion)
            } else {
                optimize(initial, constraints, motion)
            };
            (result, residual_calls.get(), motion_calls.get())
        };
        let full = solve(false);
        let limited = solve(true);
        assert_eq!(limited.0, full.0);
        assert!(limited.0[0] >= 0.4 - 1e-12);
        assert!((limited.0[1] + 0.3).abs() < 0.01);
        assert!(
            limited.1 < full.1,
            "inactive residual probes should be omitted"
        );
        assert!(
            limited.2 < full.2,
            "inactive motion probes should be omitted"
        );
    }

    #[test]
    fn impossible_problem_stays_bounded_and_does_not_refine_motion() {
        let calls = std::cell::Cell::new(0);
        let result = optimize(
            [0.; 8],
            |p| vec![MAX_COEFFICIENT + 0.2 - p[0]],
            |_| {
                calls.set(calls.get() + 1);
                0.
            },
        );
        assert_eq!(calls.get(), 0);
        assert!(result
            .iter()
            .all(|v| v.is_finite() && v.abs() <= MAX_COEFFICIENT));
        assert!((result[0] - MAX_COEFFICIENT).abs() < 1e-12);
    }

    #[test]
    fn malformed_queries_stop_without_losing_the_last_valid_candidate() {
        let initial = [0.1; 8];
        assert_eq!(optimize(initial, |_| vec![f64::NAN], |_| 0.), initial);
        assert_eq!(optimize(initial, |_| vec![-1.], |_| f64::NAN), initial);
        let changed_length = |p: &Parameters| {
            if p[0] == initial[0] {
                vec![1.]
            } else {
                vec![1., 2.]
            }
        };
        assert_eq!(optimize(initial, changed_length, |_| 0.), initial);
        let sanitized = optimize([f64::NAN; 8], |_| vec![f64::INFINITY], |_| 0.);
        assert_eq!(sanitized, [0.; 8]);
    }
}
