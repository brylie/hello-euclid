//! Pure Euclidean rhythm generation using the Bjorklund algorithm.
//! No real-time dependencies — fully unit-testable in isolation.

/// Generate a Euclidean rhythm pattern using the Bjorklund algorithm.
///
/// Returns a vector of booleans representing active (`true`) and inactive (`false`) steps.
/// The pattern is deterministic and depends only on the number of pulses and steps.
///
/// # Arguments
/// * `pulses` - Number of active steps (hits) in the pattern
/// * `steps` - Total number of steps in the pattern
///
/// # Examples
/// ```
/// // Cuban tresillo: X..X..X.
/// let pattern = bjorklund(3, 8);
/// assert_eq!(pattern, vec![true, false, false, true, false, false, true, false]);
/// ```
#[allow(dead_code)] // Used in Phase 3 and tested
pub fn bjorklund(pulses: usize, steps: usize) -> Vec<bool> {
    if pulses == 0 || steps == 0 {
        return vec![false; steps];
    }
    if pulses >= steps {
        return vec![true; steps];
    }

    let mut groups: Vec<Vec<bool>> = (0..pulses).map(|_| vec![true]).collect();
    let mut remainders: Vec<Vec<bool>> = (0..(steps - pulses)).map(|_| vec![false]).collect();

    while remainders.len() > 1 {
        let combine_count = groups.len().min(remainders.len());
        let mut new_groups = Vec::with_capacity(combine_count);

        for i in 0..combine_count {
            let mut g = groups[i].clone();
            g.extend(remainders[i].clone());
            new_groups.push(g);
        }

        let leftover_groups = groups.split_off(combine_count.min(groups.len()));
        let leftover_remainders = remainders.split_off(combine_count.min(remainders.len()));

        groups = new_groups;
        remainders = if leftover_groups.is_empty() {
            leftover_remainders
        } else {
            leftover_groups
        };
    }

    groups.into_iter().chain(remainders).flatten().collect()
}

/// Rotate a pattern by the given offset (in steps).
///
/// This allows phase shifting the pattern without changing its density.
/// Used for the "rotation" parameter to create groove variations.
///
/// # Arguments
/// * `pattern` - The input pattern to rotate
/// * `offset` - Number of steps to rotate by (wraps around)
///
/// # Examples
/// ```
/// let pattern = vec![true, false, false, true];
/// assert_eq!(rotate(&pattern, 1), vec![false, false, true, true]);
/// ```
#[allow(dead_code)] // Used in Phase 3 and tested
pub fn rotate(pattern: &[bool], offset: usize) -> Vec<bool> {
    if pattern.is_empty() {
        return vec![];
    }
    let n = pattern.len();
    let offset = offset % n;
    pattern[offset..]
        .iter()
        .chain(pattern[..offset].iter())
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e_3_8() {
        // Cuban tresillo: X..X..X.
        let result = bjorklund(3, 8);
        assert_eq!(
            result,
            vec![true, false, false, true, false, false, true, false]
        );
    }

    #[test]
    fn test_e_5_8() {
        // Classic bell pattern
        let result = bjorklund(5, 8);
        assert_eq!(
            result,
            vec![true, false, true, true, false, true, true, false]
        );
    }

    #[test]
    fn test_e_2_5() {
        let result = bjorklund(2, 5);
        assert_eq!(result, vec![true, false, true, false, false]);
    }

    #[test]
    fn test_e_7_16() {
        // 7 pulses, 16 steps - Euclidean distribution
        let result = bjorklund(7, 16);
        // Verify we get exactly 7 active steps
        assert_eq!(result.iter().filter(|&&x| x).count(), 7);
        // Verify the pattern is evenly distributed
        let result = bjorklund(7, 16);
        assert_eq!(
            result,
            vec![
                true, false, false, true, false, true, false, true, false, false, true, false,
                true, false, true, false
            ]
        );
    }

    #[test]
    fn test_pulses_equal_steps() {
        assert_eq!(bjorklund(4, 4), vec![true, true, true, true]);
    }

    #[test]
    fn test_zero_pulses() {
        assert_eq!(bjorklund(0, 8), vec![false; 8]);
    }

    #[test]
    fn test_zero_steps() {
        let result: Vec<bool> = bjorklund(5, 0);
        let expected: Vec<bool> = vec![];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_more_pulses_than_steps() {
        // Should fill all steps
        assert_eq!(bjorklund(10, 8), vec![true; 8]);
    }

    #[test]
    fn test_single_pulse_single_step() {
        assert_eq!(bjorklund(1, 1), vec![true]);
    }

    #[test]
    fn test_single_pulse_multiple_steps() {
        assert_eq!(bjorklund(1, 5), vec![true, false, false, false, false]);
    }

    #[test]
    fn test_rotation() {
        let pattern = vec![true, false, false, true];
        assert_eq!(rotate(&pattern, 1), vec![false, false, true, true]);
    }

    #[test]
    fn test_rotation_full_circle() {
        let pattern = vec![true, false, true, false];
        assert_eq!(rotate(&pattern, 4), pattern);
    }

    #[test]
    fn test_rotation_zero() {
        let pattern = vec![true, false, true, false];
        assert_eq!(rotate(&pattern, 0), pattern);
    }

    #[test]
    fn test_rotation_empty_pattern() {
        let pattern: Vec<bool> = vec![];
        let result: Vec<bool> = rotate(&pattern, 5);
        let expected: Vec<bool> = vec![];
        assert_eq!(result, expected);
    }
}
