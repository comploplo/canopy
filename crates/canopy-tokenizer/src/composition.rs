//! Semantic composition utilities for building complex meanings
//!
//! This module provides tools for composing semantic representations
//! from primitive elements extracted by the semantic engines.

use crate::{
    LogicalForm, LogicalPredicate, LogicalTerm, QuantifierStructure, SemanticPredicate,
    SemanticResult, SemanticToken,
};
use canopy_core::ThetaRole;
use std::collections::HashMap;
use tracing::{debug, info};

/// Semantic composition engine
pub struct SemanticComposer {
    // Composition rules and patterns
    composition_rules: Vec<CompositionRule>,
    #[allow(dead_code)]
    type_checker: TypeChecker,
}

/// Composition rule for combining semantic elements
#[derive(Debug, Clone)]
pub struct CompositionRule {
    pub name: String,
    pub pattern: CompositionPattern,
    pub operation: CompositionOperation,
    pub conditions: Vec<CompositionCondition>,
}

/// Pattern for matching semantic structures
#[derive(Debug, Clone)]
pub enum CompositionPattern {
    /// Predicate-argument application
    PredicateApplication {
        predicate_class: String,
        argument_roles: Vec<ThetaRole>,
    },
    /// Modifier attachment
    ModifierAttachment {
        modifier_type: String,
        target_type: String,
    },
    /// Quantifier scoping
    QuantifierScoping {
        quantifier_type: String,
        scope_type: String,
    },
}

/// Composition operation to apply
#[derive(Debug, Clone)]
pub enum CompositionOperation {
    /// Function application (F(x))
    FunctionApplication,
    /// Predicate modification (P & Q)
    PredicateModification,
    /// Quantifier raising
    QuantifierRaising,
    /// Lambda abstraction
    LambdaAbstraction { variable: String },
}

/// Condition that must be satisfied for rule application
#[derive(Debug, Clone)]
pub enum CompositionCondition {
    /// Type constraint
    TypeConstraint {
        element: String,
        required_type: String,
    },
    /// Feature constraint
    FeatureConstraint {
        element: String,
        feature: String,
        value: String,
    },
    /// Semantic constraint
    SemanticConstraint { constraint: String },
}

/// Type checker for semantic composition
pub struct TypeChecker {
    #[allow(dead_code)]
    type_assignments: HashMap<String, SemanticType>,
}

/// Semantic types for composition
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticType {
    /// Entity type (e)
    Entity,
    /// Truth value type (t)
    Truth,
    /// Event type (v)
    Event,
    /// Function type (α → β)
    Function(Box<SemanticType>, Box<SemanticType>),
    /// Generalized quantifier type <<e,t>,t>
    GeneralizedQuantifier,
}

impl SemanticComposer {
    /// Create a new semantic composer with default rules
    pub fn new() -> SemanticResult<Self> {
        info!("Initializing semantic composer");

        let composition_rules = Self::default_composition_rules();
        let type_checker = TypeChecker::new();

        Ok(Self {
            composition_rules,
            type_checker,
        })
    }

    /// Compose semantic tokens into a logical form
    pub fn compose_tokens(&self, tokens: &[SemanticToken]) -> SemanticResult<LogicalForm> {
        debug!("Composing {} semantic tokens", tokens.len());

        let mut logical_predicates = Vec::new();
        let mut variables = HashMap::new();
        let mut quantifiers = Vec::new();

        // Extract predicates from tokens
        for (i, token) in tokens.iter().enumerate() {
            match token.semantic_class {
                crate::SemanticClass::Predicate => {
                    if let Some(verbnet_class) = token.verbnet_classes.first() {
                        let predicate = LogicalPredicate {
                            name: token.lemma.clone(),
                            arguments: verbnet_class
                                .themroles
                                .iter()
                                .enumerate()
                                .map(|(j, _role)| LogicalTerm::Variable(format!("x{j}")))
                                .collect(),
                            arity: verbnet_class.themroles.len() as u8,
                        };
                        logical_predicates.push(predicate);
                    }
                }
                crate::SemanticClass::Argument => {
                    // Create entity variables for arguments
                    variables.insert(format!("x{i}"), LogicalTerm::Constant(token.lemma.clone()));
                }
                crate::SemanticClass::Quantifier => {
                    // Handle quantifier structures
                    let quantifier = QuantifierStructure {
                        quantifier_type: self.determine_quantifier_type(&token.text),
                        variable: format!("x{i}"),
                        restriction: LogicalPredicate {
                            name: "entity".to_string(),
                            arguments: vec![LogicalTerm::Variable(format!("x{i}"))],
                            arity: 1,
                        },
                        scope: LogicalPredicate {
                            name: "true".to_string(),
                            arguments: vec![],
                            arity: 0,
                        },
                    };
                    quantifiers.push(quantifier);
                }
                _ => {
                    // Handle other semantic classes
                    debug!(
                        "Skipping token '{}' with class {:?}",
                        token.text, token.semantic_class
                    );
                }
            }
        }

        Ok(LogicalForm {
            predicates: logical_predicates,
            variables,
            quantifiers,
        })
    }

    /// Compose semantic predicates into structured events
    pub fn compose_predicates(
        &self,
        predicates: &[SemanticPredicate],
    ) -> SemanticResult<LogicalForm> {
        debug!("Composing {} semantic predicates", predicates.len());

        let logical_predicates: Vec<LogicalPredicate> = predicates
            .iter()
            .map(|p| LogicalPredicate {
                name: p.lemma.clone(),
                arguments: p
                    .theta_grid
                    .iter()
                    .enumerate()
                    .map(|(i, _role)| LogicalTerm::Variable(format!("x{i}")))
                    .collect(),
                arity: p.theta_grid.len() as u8,
            })
            .collect();

        Ok(LogicalForm {
            predicates: logical_predicates,
            variables: HashMap::new(),
            quantifiers: Vec::new(),
        })
    }

    /// Apply composition rules to combine semantic elements
    pub fn apply_rules(
        &self,
        elements: &[CompositionElement],
    ) -> SemanticResult<ComposedStructure> {
        debug!("Applying composition rules to {} elements", elements.len());

        let mut result = ComposedStructure::new();

        for rule in &self.composition_rules {
            if self.matches_pattern(&rule.pattern, elements)
                && self.check_conditions(&rule.conditions, elements)
            {
                result = self.apply_operation(&rule.operation, elements, result)?;
            }
        }

        Ok(result)
    }

    /// Default composition rules for semantic analysis
    fn default_composition_rules() -> Vec<CompositionRule> {
        vec![
            // Predicate-argument application rule
            CompositionRule {
                name: "predicate_application".to_string(),
                pattern: CompositionPattern::PredicateApplication {
                    predicate_class: "verb".to_string(),
                    argument_roles: vec![ThetaRole::Agent, ThetaRole::Theme],
                },
                operation: CompositionOperation::FunctionApplication,
                conditions: vec![CompositionCondition::TypeConstraint {
                    element: "predicate".to_string(),
                    required_type: "function".to_string(),
                }],
            },
            // Modifier attachment rule
            CompositionRule {
                name: "modifier_attachment".to_string(),
                pattern: CompositionPattern::ModifierAttachment {
                    modifier_type: "adjective".to_string(),
                    target_type: "noun".to_string(),
                },
                operation: CompositionOperation::PredicateModification,
                conditions: vec![],
            },
            // Quantifier scoping rule
            CompositionRule {
                name: "quantifier_scoping".to_string(),
                pattern: CompositionPattern::QuantifierScoping {
                    quantifier_type: "determiner".to_string(),
                    scope_type: "sentence".to_string(),
                },
                operation: CompositionOperation::QuantifierRaising,
                conditions: vec![],
            },
        ]
    }

    /// Check if elements match a composition pattern
    fn matches_pattern(
        &self,
        _pattern: &CompositionPattern,
        _elements: &[CompositionElement],
    ) -> bool {
        // Simplified pattern matching
        // A full implementation would check semantic classes, types, etc.
        true
    }

    /// Check if composition conditions are satisfied
    fn check_conditions(
        &self,
        _conditions: &[CompositionCondition],
        _elements: &[CompositionElement],
    ) -> bool {
        // Simplified condition checking
        // A full implementation would verify type constraints, features, etc.
        true
    }

    /// Apply a composition operation
    fn apply_operation(
        &self,
        _operation: &CompositionOperation,
        _elements: &[CompositionElement],
        result: ComposedStructure,
    ) -> SemanticResult<ComposedStructure> {
        // Simplified operation application
        // A full implementation would perform actual semantic composition
        Ok(result)
    }

    /// Determine quantifier type from text
    fn determine_quantifier_type(&self, text: &str) -> crate::QuantifierType {
        match text.to_lowercase().as_str() {
            "every" | "all" => crate::QuantifierType::Universal,
            "some" | "a" | "an" => crate::QuantifierType::Existential,
            "the" => crate::QuantifierType::Definite,
            _ => crate::QuantifierType::Indefinite,
        }
    }
}

/// Element for semantic composition
#[derive(Debug, Clone)]
pub struct CompositionElement {
    pub semantic_type: SemanticType,
    pub content: String,
    pub features: HashMap<String, String>,
}

/// Result of semantic composition
#[derive(Debug, Clone)]
pub struct ComposedStructure {
    pub components: Vec<CompositionElement>,
    pub relations: Vec<CompositionRelation>,
    pub logical_form: Option<LogicalForm>,
}

/// Relation between composed elements
#[derive(Debug, Clone)]
pub struct CompositionRelation {
    pub relation_type: String,
    pub source: usize,
    pub target: usize,
    pub properties: HashMap<String, String>,
}

impl ComposedStructure {
    fn new() -> Self {
        Self {
            components: Vec::new(),
            relations: Vec::new(),
            logical_form: None,
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        Self {
            type_assignments: HashMap::new(),
        }
    }

    /// Check if types are compatible for composition
    pub fn check_compatibility(type1: &SemanticType, type2: &SemanticType) -> bool {
        match (type1, type2) {
            (SemanticType::Function(input, _output), input_type) => {
                Self::check_compatibility(input, input_type)
            }
            (SemanticType::Entity, SemanticType::Entity) => true,
            (SemanticType::Truth, SemanticType::Truth) => true,
            (SemanticType::Event, SemanticType::Event) => true,
            _ => false,
        }
    }

    /// Infer the result type of composition
    pub fn infer_result_type(
        &self,
        type1: &SemanticType,
        type2: &SemanticType,
    ) -> Option<SemanticType> {
        match (type1, type2) {
            (SemanticType::Function(_input, output), _) => Some((**output).clone()),
            _ => None,
        }
    }
}

impl Default for SemanticComposer {
    fn default() -> Self {
        Self::new().expect("Failed to initialize semantic composer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composer_creation() {
        let composer = SemanticComposer::new().unwrap();
        assert!(!composer.composition_rules.is_empty());
    }

    #[test]
    fn test_type_compatibility() {
        let _type_checker = TypeChecker::new();

        let entity = SemanticType::Entity;
        let function = SemanticType::Function(
            Box::new(SemanticType::Entity),
            Box::new(SemanticType::Truth),
        );

        assert!(TypeChecker::check_compatibility(&function, &entity));
        assert!(TypeChecker::check_compatibility(&entity, &entity));
    }

    #[test]
    fn test_type_inference() {
        let type_checker = TypeChecker::new();

        let function = SemanticType::Function(
            Box::new(SemanticType::Entity),
            Box::new(SemanticType::Truth),
        );
        let entity = SemanticType::Entity;

        let result = type_checker.infer_result_type(&function, &entity);
        assert_eq!(result, Some(SemanticType::Truth));
    }

    #[test]
    fn test_quantifier_type_determination() {
        let composer = SemanticComposer::new().unwrap();

        assert_eq!(
            composer.determine_quantifier_type("every"),
            crate::QuantifierType::Universal
        );
        assert_eq!(
            composer.determine_quantifier_type("some"),
            crate::QuantifierType::Existential
        );
        assert_eq!(
            composer.determine_quantifier_type("the"),
            crate::QuantifierType::Definite
        );
    }

    #[test]
    fn test_compose_tokens_with_predicates() {
        let composer = SemanticComposer::new().unwrap();

        // Create test tokens with VerbNet classes
        use canopy_verbnet::{SelectionalRestrictions, ThematicRole, VerbClass};

        let give_verbnet_class = VerbClass {
            id: "give-13.1".to_string(),
            class_name: "Giving".to_string(),
            parent_class: None,
            members: vec![],
            themroles: vec![
                ThematicRole {
                    role_type: "Agent".to_string(),
                    selrestrs: SelectionalRestrictions {
                        logic: None,
                        restrictions: vec![],
                    },
                },
                ThematicRole {
                    role_type: "Theme".to_string(),
                    selrestrs: SelectionalRestrictions {
                        logic: None,
                        restrictions: vec![],
                    },
                },
            ],
            frames: vec![],
            subclasses: vec![],
        };

        let predicate_token = crate::SemanticToken {
            text: "give".to_string(),
            lemma: "give".to_string(),
            semantic_class: crate::SemanticClass::Predicate,
            frames: vec![],
            verbnet_classes: vec![give_verbnet_class],
            wordnet_senses: vec![],
            morphology: crate::MorphologicalAnalysis {
                lemma: "give".to_string(),
                features: std::collections::HashMap::new(),
                inflection_type: crate::InflectionType::None,
                is_recognized: true,
            },
            confidence: 0.9,
        };

        let argument_token = crate::SemanticToken {
            text: "book".to_string(),
            lemma: "book".to_string(),
            semantic_class: crate::SemanticClass::Argument,
            frames: vec![],
            verbnet_classes: vec![],
            wordnet_senses: vec![],
            morphology: crate::MorphologicalAnalysis {
                lemma: "book".to_string(),
                features: std::collections::HashMap::new(),
                inflection_type: crate::InflectionType::None,
                is_recognized: true,
            },
            confidence: 0.8,
        };

        let tokens = vec![predicate_token, argument_token];
        let logical_form = composer.compose_tokens(&tokens).unwrap();

        // Should have predicates and variables
        assert!(!logical_form.predicates.is_empty());
        assert!(!logical_form.variables.is_empty());

        // Check predicate structure
        let predicate = &logical_form.predicates[0];
        assert_eq!(predicate.name, "give");
        assert_eq!(predicate.arity, 2); // Agent + Theme roles
    }

    #[test]
    fn test_compose_tokens_with_quantifiers() {
        let composer = SemanticComposer::new().unwrap();

        let quantifier_token = crate::SemanticToken {
            text: "every".to_string(),
            lemma: "every".to_string(),
            semantic_class: crate::SemanticClass::Quantifier,
            frames: vec![],
            verbnet_classes: vec![],
            wordnet_senses: vec![],
            morphology: crate::MorphologicalAnalysis {
                lemma: "every".to_string(),
                features: std::collections::HashMap::new(),
                inflection_type: crate::InflectionType::None,
                is_recognized: true,
            },
            confidence: 0.9,
        };

        let tokens = vec![quantifier_token];
        let logical_form = composer.compose_tokens(&tokens).unwrap();

        // Should have quantifier structures
        assert!(!logical_form.quantifiers.is_empty());

        let quantifier = &logical_form.quantifiers[0];
        assert_eq!(quantifier.quantifier_type, crate::QuantifierType::Universal);
        assert!(quantifier.variable.starts_with("x"));
    }

    #[test]
    fn test_compose_predicates() {
        let composer = SemanticComposer::new().unwrap();

        let predicate = crate::SemanticPredicate {
            lemma: "love".to_string(),
            verbnet_class: Some("love-31.2".to_string()),
            theta_grid: vec![canopy_core::ThetaRole::Agent, canopy_core::ThetaRole::Theme],
            selectional_restrictions: std::collections::HashMap::new(),
            aspectual_class: crate::AspectualClass::Activity,
            confidence: 0.9,
        };

        let predicates = vec![predicate];
        let logical_form = composer.compose_predicates(&predicates).unwrap();

        assert!(!logical_form.predicates.is_empty());
        let logical_pred = &logical_form.predicates[0];
        assert_eq!(logical_pred.name, "love");
        assert_eq!(logical_pred.arity, 2);
    }

    #[test]
    fn test_semantic_type_functions() {
        // Test function type creation
        let entity_to_truth = SemanticType::Function(
            Box::new(SemanticType::Entity),
            Box::new(SemanticType::Truth),
        );

        if let SemanticType::Function(domain, codomain) = &entity_to_truth {
            assert_eq!(**domain, SemanticType::Entity);
            assert_eq!(**codomain, SemanticType::Truth);
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_composition_rule_structure() {
        let rule = CompositionRule {
            name: "test_rule".to_string(),
            pattern: CompositionPattern::PredicateApplication {
                predicate_class: "verb".to_string(),
                argument_roles: vec![canopy_core::ThetaRole::Agent],
            },
            operation: CompositionOperation::FunctionApplication,
            conditions: vec![CompositionCondition::TypeConstraint {
                element: "predicate".to_string(),
                required_type: "function".to_string(),
            }],
        };

        assert_eq!(rule.name, "test_rule");
        assert!(matches!(
            rule.pattern,
            CompositionPattern::PredicateApplication { .. }
        ));
        assert!(matches!(
            rule.operation,
            CompositionOperation::FunctionApplication
        ));
        assert_eq!(rule.conditions.len(), 1);
    }

    #[test]
    fn test_composition_patterns() {
        // Test different composition patterns
        let predicate_app = CompositionPattern::PredicateApplication {
            predicate_class: "give".to_string(),
            argument_roles: vec![canopy_core::ThetaRole::Agent, canopy_core::ThetaRole::Theme],
        };

        let _modifier_attach = CompositionPattern::ModifierAttachment {
            modifier_type: "adverb".to_string(),
            target_type: "verb".to_string(),
        };

        let _quantifier_scope = CompositionPattern::QuantifierScoping {
            quantifier_type: "universal".to_string(),
            scope_type: "proposition".to_string(),
        };

        // Test pattern matching (basic structure verification)
        match predicate_app {
            CompositionPattern::PredicateApplication {
                predicate_class, ..
            } => {
                assert_eq!(predicate_class, "give");
            }
            _ => panic!("Wrong pattern type"),
        }
    }

    #[test]
    fn test_composition_operations() {
        // Test different operation types
        let func_app = CompositionOperation::FunctionApplication;
        let pred_mod = CompositionOperation::PredicateModification;
        let quant_raising = CompositionOperation::QuantifierRaising;
        let lambda_abs = CompositionOperation::LambdaAbstraction {
            variable: "x1".to_string(),
        };

        // Verify structure
        assert!(matches!(
            func_app,
            CompositionOperation::FunctionApplication
        ));
        assert!(matches!(
            pred_mod,
            CompositionOperation::PredicateModification
        ));
        assert!(matches!(
            quant_raising,
            CompositionOperation::QuantifierRaising
        ));

        if let CompositionOperation::LambdaAbstraction { variable } = lambda_abs {
            assert_eq!(variable, "x1");
        }
    }

    #[test]
    fn test_composition_conditions() {
        // Test different condition types
        let type_constraint = CompositionCondition::TypeConstraint {
            element: "arg1".to_string(),
            required_type: "entity".to_string(),
        };

        let _feature_constraint = CompositionCondition::FeatureConstraint {
            element: "pred".to_string(),
            feature: "tense".to_string(),
            value: "present".to_string(),
        };

        let _semantic_constraint = CompositionCondition::SemanticConstraint {
            constraint: "animate_agent".to_string(),
        };

        // Verify structure
        match type_constraint {
            CompositionCondition::TypeConstraint {
                element,
                required_type,
            } => {
                assert_eq!(element, "arg1");
                assert_eq!(required_type, "entity");
            }
            _ => panic!("Wrong condition type"),
        }
    }

    #[test]
    fn test_empty_token_composition() {
        let composer = SemanticComposer::new().unwrap();
        let logical_form = composer.compose_tokens(&[]).unwrap();

        // Empty input should produce empty logical form
        assert!(logical_form.predicates.is_empty());
        assert!(logical_form.variables.is_empty());
        assert!(logical_form.quantifiers.is_empty());
    }

    #[test]
    fn test_mixed_semantic_classes() {
        let composer = SemanticComposer::new().unwrap();

        // Create tokens with different semantic classes
        let function_token = crate::SemanticToken {
            text: "the".to_string(),
            lemma: "the".to_string(),
            semantic_class: crate::SemanticClass::Function,
            frames: vec![],
            verbnet_classes: vec![],
            wordnet_senses: vec![],
            morphology: crate::MorphologicalAnalysis {
                lemma: "the".to_string(),
                features: std::collections::HashMap::new(),
                inflection_type: crate::InflectionType::None,
                is_recognized: true,
            },
            confidence: 0.9,
        };

        let modifier_token = crate::SemanticToken {
            text: "quickly".to_string(),
            lemma: "quickly".to_string(),
            semantic_class: crate::SemanticClass::Modifier,
            frames: vec![],
            verbnet_classes: vec![],
            wordnet_senses: vec![],
            morphology: crate::MorphologicalAnalysis {
                lemma: "quickly".to_string(),
                features: std::collections::HashMap::new(),
                inflection_type: crate::InflectionType::None,
                is_recognized: true,
            },
            confidence: 0.8,
        };

        let unknown_token = crate::SemanticToken {
            text: "xyz".to_string(),
            lemma: "xyz".to_string(),
            semantic_class: crate::SemanticClass::Unknown,
            frames: vec![],
            verbnet_classes: vec![],
            wordnet_senses: vec![],
            morphology: crate::MorphologicalAnalysis {
                lemma: "xyz".to_string(),
                features: std::collections::HashMap::new(),
                inflection_type: crate::InflectionType::None,
                is_recognized: false,
            },
            confidence: 0.1,
        };

        let tokens = vec![function_token, modifier_token, unknown_token];
        let logical_form = composer.compose_tokens(&tokens).unwrap();

        // Should handle mixed classes gracefully
        // These tokens don't create predicates, variables, or quantifiers
        assert!(logical_form.predicates.is_empty());
        assert!(logical_form.variables.is_empty());
        assert!(logical_form.quantifiers.is_empty());
    }
}
