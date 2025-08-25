//! Additional coverage tests for lib.rs uncovered lines

use canopy_core::{Entity, LittleV, Path, PossessionType};

#[cfg(test)]
mod lib_coverage_tests {
    use super::*;

    #[test]
    fn test_little_v_go_external_argument() {
        let theme = Entity {
            id: 1,
            text: "book".to_string(),
            animacy: None,
            definiteness: None,
        };

        let path = Path {
            source: None,
            goal: None,
            route: None,
            direction: None,
        };

        let little_v = LittleV::Go {
            theme: theme.clone(),
            path,
        };

        // This should cover line 838
        let external_arg = little_v.external_argument();
        assert_eq!(external_arg, Some(&theme));
    }

    #[test]
    fn test_little_v_have_external_argument() {
        let possessor = Entity {
            id: 2,
            text: "John".to_string(),
            animacy: None,
            definiteness: None,
        };

        let possessee = Entity {
            id: 3,
            text: "car".to_string(),
            animacy: None,
            definiteness: None,
        };

        let little_v = LittleV::Have {
            possessor: possessor.clone(),
            possessee,
            possession_type: PossessionType::Legal,
        };

        // This should cover line 839
        let external_arg = little_v.external_argument();
        assert_eq!(external_arg, Some(&possessor));
    }
}
