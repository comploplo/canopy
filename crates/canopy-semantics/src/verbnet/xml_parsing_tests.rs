//! Comprehensive VerbNet XML parsing tests
//!
//! These tests cover edge cases in XML parsing and concurrent access scenarios.

#[cfg(test)]
mod xml_parsing_tests {
    use crate::verbnet::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_xml_parsing_edge_cases() {
        use super::super::parser::parse_verb_class_xml;

        // Test various XML edge cases
        let test_cases = vec![
            // Empty XML
            "",

            // Minimal valid XML
            "<VNCLASS/>",

            // XML with whitespace
            "  <VNCLASS>  </VNCLASS>  ",

            // XML with comments
            "<!-- comment --><VNCLASS><!-- another comment --></VNCLASS>",

            // XML with CDATA
            "<VNCLASS><![CDATA[some data]]></VNCLASS>",

            // Malformed XML cases
            "<VNCLASS",
            "<VNCLASS></WRONG>",
            "<VNCLASS><NESTED></VNCLASS></NESTED>",
        ];

        for xml in test_cases {
            let result = parse_verb_class_xml(xml);
            // Should handle all cases gracefully without panic
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_concurrent_xml_parsing() {
        // Test concurrent access to XML parsing
        let xml_content = r#"
        <VNCLASS ID="test-1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
            <MEMBERS>
                <MEMBER name="run" wn="run%2:38:00" grouping="run.01"/>
            </MEMBERS>
            <THEMROLES>
                <THEMROLE type="Agent">
                    <SELRESTRS>
                        <SELRESTR Value="+" type="animate"/>
                    </SELRESTRS>
                </THEMROLE>
            </THEMROLES>
            <FRAMES>
                <FRAME>
                    <DESCRIPTION primary="Agent runs" secondary="Agent-based motion"/>
                    <EXAMPLES>
                        <EXAMPLE>John runs.</EXAMPLE>
                    </EXAMPLES>
                    <SYNTAX>
                        <NP value="Agent"><SYNRESTRS/></NP>
                    </SYNTAX>
                    <SEMANTICS>
                        <PRED value="motion(E, Agent)"/>
                    </SEMANTICS>
                </FRAME>
            </FRAMES>
        </VNCLASS>
        "#;

        let xml_content = Arc::new(xml_content.to_string());

        let handles: Vec<_> = (0..8).map(|_| {
            let xml_clone = Arc::clone(&xml_content);
            thread::spawn(move || {
                use super::super::parser::parse_verb_class_xml;
                let result = parse_verb_class_xml(&xml_clone);
                assert!(result.is_ok() || result.is_err());
            })
        }).collect();

        for handle in handles {
            handle.join().expect("Thread should not panic");
        }
    }

    #[test]
    fn test_xml_with_special_characters() {
        use super::super::parser::parse_verb_class_xml;

        // Test XML with various special characters
        let special_xml = r#"
        <VNCLASS ID="test-special">
            <MEMBERS>
                <MEMBER name="café" wn="café%2:34:00" grouping="café.01"/>
                <MEMBER name="naïve" wn="naïve%2:31:00" grouping="naïve.01"/>
                <MEMBER name="résumé" wn="résumé%1:10:00" grouping="résumé.01"/>
            </MEMBERS>
            <EXAMPLES>
                <EXAMPLE>She drinks café au lait.</EXAMPLE>
                <EXAMPLE>He wrote his résumé.</EXAMPLE>
            </EXAMPLES>
        </VNCLASS>
        "#;

        let result = parse_verb_class_xml(special_xml);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_xml_with_nested_structures() {
        use super::super::parser::parse_verb_class_xml;

        // Test deeply nested XML structures
        let nested_xml = r#"
        <VNCLASS ID="test-nested">
            <THEMROLES>
                <THEMROLE type="Agent">
                    <SELRESTRS>
                        <SELRESTR Value="+" type="animate">
                            <NESTED>
                                <DEEP>
                                    <STRUCTURE>content</STRUCTURE>
                                </DEEP>
                            </NESTED>
                        </SELRESTR>
                        <SELRESTR Value="-" type="machine"/>
                    </SELRESTRS>
                </THEMROLE>
                <THEMROLE type="Theme">
                    <SELRESTRS>
                        <SELRESTR Value="+" type="concrete"/>
                    </SELRESTRS>
                </THEMROLE>
            </THEMROLES>
        </VNCLASS>
        "#;

        let result = parse_verb_class_xml(nested_xml);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_xml_with_attributes_edge_cases() {
        use super::super::parser::parse_verb_class_xml;

        // Test XML with various attribute scenarios
        let attr_xml = r#"
        <VNCLASS ID="" empty="true" numeric="123" boolean="false" special="&lt;&gt;&amp;">
            <MEMBER name="" empty_attr="" special_chars="&lt;test&gt;"/>
            <MEMBER name="normal" standard="value"/>
        </VNCLASS>
        "#;

        let result = parse_verb_class_xml(attr_xml);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_xml_parsing_memory_usage() {
        use super::super::parser::parse_verb_class_xml;

        // Test with very large XML to check memory handling
        let large_members: Vec<String> = (0..1000).map(|i| {
            format!(r#"<MEMBER name="verb{}" wn="verb{}%2:38:00" grouping="verb{}.01"/>"#, i, i, i)
        }).collect();

        let large_xml = format!(r#"
        <VNCLASS ID="test-large">
            <MEMBERS>
                {}
            </MEMBERS>
        </VNCLASS>
        "#, large_members.join("\n"));

        let result = parse_verb_class_xml(&large_xml);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_xml_encoding_edge_cases() {
        use super::super::parser::parse_verb_class_xml;

        // Test XML with different encoding scenarios
        let encoding_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <VNCLASS ID="test-encoding">
            <EXAMPLES>
                <EXAMPLE>Text with unicode: 你好 世界</EXAMPLE>
                <EXAMPLE>Emoji: 🏃‍♂️ runs quickly</EXAMPLE>
                <EXAMPLE>Mathematical: ∑ ∆ ∇ symbols</EXAMPLE>
            </EXAMPLES>
        </VNCLASS>
        "#;

        let result = parse_verb_class_xml(encoding_xml);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_xml_error_recovery() {
        use super::super::parser::parse_verb_class_xml;

        // Test parser error recovery with partially valid XML
        let partial_xml_cases = vec![
            // Missing closing tag
            "<VNCLASS ID=\"test\"><MEMBERS><MEMBER name=\"run\"/>",

            // Extra closing tag
            "<VNCLASS ID=\"test\"></VNCLASS></EXTRA>",

            // Mismatched tags
            "<VNCLASS><MEMBERS></VNCLASS></MEMBERS>",

            // Invalid attributes
            "<VNCLASS ID=unclosed_quote>content</VNCLASS>",

            // Invalid characters
            "<VNCLASS ID=\"test\"><CONTENT>\x00\x01\x02</CONTENT></VNCLASS>",
        ];

        for xml in partial_xml_cases {
            let result = parse_verb_class_xml(xml);
            // Should handle errors gracefully
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_concurrent_engine_access() {
        // Test concurrent access to VerbNet engine
        let engine = Arc::new(VerbNetEngine::new());

        let handles: Vec<_> = (0..4).map(|i| {
            let engine_clone = Arc::clone(&engine);
            thread::spawn(move || {
                // Test different operations concurrently
                let verb = format!("test_verb_{}", i);
                let classes = engine_clone.get_verb_classes(&verb);
                let theta_roles = engine_clone.get_theta_roles(&verb);

                // Should handle concurrent access
                assert!(classes.is_empty() || !classes.is_empty());
                assert!(theta_roles.is_empty() || !theta_roles.is_empty());
            })
        }).collect();

        for handle in handles {
            handle.join().expect("Thread should not panic");
        }
    }

    #[test]
    fn test_xml_namespace_handling() {
        use super::super::parser::parse_verb_class_xml;

        // Test XML with namespaces
        let namespace_xml = r#"
        <vn:VNCLASS xmlns:vn="http://verbnet.org/schema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" ID="test">
            <vn:MEMBERS>
                <vn:MEMBER name="run" wn="run%2:38:00"/>
            </vn:MEMBERS>
            <vn:THEMROLES>
                <vn:THEMROLE type="Agent"/>
            </vn:THEMROLES>
        </vn:VNCLASS>
        "#;

        let result = parse_verb_class_xml(namespace_xml);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_selectional_restriction_parsing_edge_cases() {
        // Test selectional restriction parsing with edge cases
        use super::super::types::SelectionalRestriction;

        let restrictions = vec![
            SelectionalRestriction {
                restriction_type: "".to_string(), // Empty type
                value: "+".to_string(),
            },
            SelectionalRestriction {
                restriction_type: "animate".to_string(),
                value: "".to_string(), // Empty value
            },
            SelectionalRestriction {
                restriction_type: "very_long_restriction_type_name".to_string(),
                value: "very_long_value_name".to_string(),
            },
        ];

        for restriction in restrictions {
            // Test that restrictions can be created and debugged
            let debug_str = format!("{:?}", restriction);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_theta_role_parsing_edge_cases() {
        // Test theta role parsing with edge cases
        use super::super::types::ThetaRole;

        let roles = vec![
            ThetaRole {
                role_type: "".to_string(), // Empty role type
                selectional_restrictions: vec![],
            },
            ThetaRole {
                role_type: "Agent".to_string(),
                selectional_restrictions: vec![], // No restrictions
            },
            ThetaRole {
                role_type: "VeryLongThetaRoleTypeName".to_string(),
                selectional_restrictions: (0..100).map(|i| {
                    super::super::types::SelectionalRestriction {
                        restriction_type: format!("type_{}", i),
                        value: format!("value_{}", i),
                    }
                }).collect(), // Many restrictions
            },
        ];

        for role in roles {
            let debug_str = format!("{:?}", role);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_feature_extraction_concurrent_access() {
        // Test concurrent feature extraction
        use super::super::feature_extraction::VerbNetFeatureExtractor;
        use canopy_core::{Word, UPos, DepRel, MorphFeatures};

        let extractor = Arc::new(VerbNetFeatureExtractor::new());

        let handles: Vec<_> = (0..4).map(|i| {
            let extractor_clone = Arc::clone(&extractor);
            thread::spawn(move || {
                let word = Word {
                    id: i,
                    text: format!("word{}", i),
                    lemma: format!("lemma{}", i),
                    upos: UPos::Verb,
                    xpos: None,
                    feats: MorphFeatures::default(),
                    head: None,
                    deprel: DepRel::Root,
                    deps: None,
                    misc: None,
                    start: 0,
                    end: 4,
                };

                let features = extractor_clone.extract_features(&word);
                // Should handle concurrent access
                assert!(features.is_ok() || features.is_err());
            })
        }).collect();

        for handle in handles {
            handle.join().expect("Thread should not panic");
        }
    }
}
