//! Comprehensive parser tests for canopy-lexicon
//!
//! Tests to improve parser coverage by testing more XML variations and edge cases

use canopy_engine::{XmlParser, XmlResource};
use canopy_lexicon::parser::LexiconXmlResource;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

#[cfg(test)]
mod comprehensive_parser_tests {
    use super::*;

    fn create_test_file(content: &str) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let mut file = File::create(&file_path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write test data");

        temp_dir
    }

    fn create_lexicon_with_attributes() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="2.0" language="en-US" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Advanced Test Lexicon</title>
    <description>Test lexicon with various attributes</description>
    <created>2024-01-01</created>
    <author>Test Author</author>
    <license>MIT License</license>
  </metadata>
  
  <word-classes>
    <word-class id="attr-stop-words" name="Attributed Stop Words" type="stop-words" priority="5">
      <description>Stop words with various attributes</description>
      <properties>
        <property name="semantic-weight" value="0.05" type="float"/>
        <property name="frequency-threshold" value="10000" type="int"/>
        <property name="enabled" value="true" type="bool"/>
        <property name="category" value="function" type="string"/>
      </properties>
      <words>
        <word pos="DT" confidence="0.95" frequency="50000" context="determiner">the</word>
        <word pos="DT" confidence="0.90" frequency="30000" context="article">a</word>
        <word pos="CC" confidence="0.85" frequency="20000">and</word>
        <word pos="IN" confidence="0.80">of</word>
      </words>
    </word-class>
    
    <word-class id="attr-negation" name="Negation with Patterns" type="negation" priority="8">
      <description>Negation words with patterns</description>
      <words>
        <word pos="RB" confidence="0.99">not</word>
        <word pos="DT" confidence="0.95">no</word>
      </words>
      <patterns>
        <pattern id="neg-prefix-un" type="prefix" confidence="0.8">
          <regex>^un[a-z]+</regex>
          <description>Un- prefix for negation</description>
          <examples>
            <example>unhappy</example>
            <example>unable</example>
            <example>uncertain</example>
          </examples>
        </pattern>
        <pattern id="neg-prefix-in" type="prefix" confidence="0.75">
          <regex>^in[a-z]+</regex>
          <description>In- prefix for negation</description>
          <examples>
            <example>inactive</example>
            <example>incorrect</example>
          </examples>
        </pattern>
        <pattern id="neg-suffix-less" type="suffix" confidence="0.70">
          <regex>[a-z]+less$</regex>
          <description>-less suffix for negation</description>
          <examples>
            <example>hopeless</example>
            <example>careless</example>
          </examples>
        </pattern>
      </patterns>
    </word-class>
  </word-classes>
</lexicon>"#
            .to_string()
    }

    fn create_lexicon_with_all_pattern_types() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.5" language="fr" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Pattern Type Test Lexicon</title>
    <description>Testing all pattern types</description>
    <created>2024-01-15</created>
    <author>Pattern Tester</author>
    <license>Apache 2.0</license>
  </metadata>
  
  <word-classes>
    <word-class id="pattern-test" name="Pattern Test Class" type="functional" priority="1">
      <description>Testing various pattern types</description>
      <patterns>
        <pattern id="prefix-re" type="prefix" confidence="0.9">
          <regex>^re[a-z]+</regex>
          <description>Re- prefix</description>
          <examples>
            <example>restart</example>
            <example>return</example>
          </examples>
        </pattern>
        <pattern id="suffix-ing" type="suffix" confidence="0.85">
          <regex>[a-z]+ing$</regex>
          <description>-ing suffix</description>
          <examples>
            <example>running</example>
            <example>walking</example>
          </examples>
        </pattern>
        <pattern id="infix-test" type="infix" confidence="0.80">
          <regex>[a-z]+ed[a-z]+</regex>
          <description>Contains 'ed'</description>
          <examples>
            <example>rededicate</example>
          </examples>
        </pattern>
        <pattern id="whole-word-test" type="whole-word" confidence="0.95">
          <regex>^complete$</regex>
          <description>Exact word match</description>
          <examples>
            <example>complete</example>
          </examples>
        </pattern>
        <pattern id="phrase-test" type="phrase" confidence="0.75">
          <regex>^in fact$</regex>
          <description>Phrase pattern</description>
          <examples>
            <example>in fact</example>
          </examples>
        </pattern>
      </patterns>
    </word-class>
  </word-classes>
</lexicon>"#
            .to_string()
    }

    fn create_complex_nested_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Complex Nested Lexicon</title>
    <description>Testing complex nesting and structures</description>
    <created>2024-02-01</created>
    <author>Complex Tester</author>
    <license>BSD</license>
  </metadata>
  
  <word-classes>
    <word-class id="complex-class1" name="Complex Class One" type="pronouns" priority="10">
      <description>First complex class</description>
      <properties>
        <property name="complexity" value="high" type="string"/>
        <property name="nested-level" value="3" type="int"/>
        <property name="active" value="true" type="bool"/>
        <property name="weight" value="0.75" type="float"/>
      </properties>
      <words>
        <word pos="PRP" confidence="0.98" frequency="100000" context="personal">I</word>
        <word pos="PRP" confidence="0.97" frequency="95000" context="personal">you</word>
        <word pos="PRP" confidence="0.96" frequency="90000" context="personal">he</word>
        <word pos="PRP" confidence="0.95" frequency="85000" context="personal">she</word>
      </words>
    </word-class>

    <word-class id="complex-class2" name="Complex Class Two" type="prepositions" priority="9">
      <description>Second complex class with patterns</description>
      <properties>
        <property name="spatial" value="true" type="bool"/>
        <property name="temporal" value="false" type="bool"/>
      </properties>
      <words>
        <word pos="IN">in</word>
        <word pos="IN">on</word>
        <word pos="IN">at</word>
        <word pos="IN">under</word>
      </words>
      <patterns>
        <pattern id="prep-compound" type="phrase" confidence="0.8">
          <regex>^(in|on|at) (the|a|an) [a-z]+$</regex>
          <description>Prepositional phrases</description>
          <examples>
            <example>in the house</example>
            <example>on a table</example>
            <example>at the store</example>
          </examples>
        </pattern>
      </patterns>
    </word-class>

    <word-class id="complex-class3" name="Complex Class Three" type="conjunctions" priority="7">
      <description>Third complex class</description>
      <words>
        <word pos="CC" confidence="0.99" frequency="75000">and</word>
        <word pos="CC" confidence="0.95" frequency="45000">but</word>
        <word pos="CC" confidence="0.90" frequency="25000">or</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#
            .to_string()
    }

    #[test]
    fn test_parse_lexicon_with_attributes() {
        let xml_content = create_lexicon_with_attributes();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(
            result.is_ok(),
            "Should parse lexicon with attributes successfully"
        );

        let resource = result.unwrap();
        let validation = resource.validate();
        assert!(
            validation.is_ok(),
            "Resource with attributes should validate"
        );

        let stats = resource.database.stats();
        assert!(
            stats.total_words >= 4,
            "Should have parsed at least 4 words with attributes"
        );
        assert!(
            stats.total_word_classes >= 2,
            "Should have parsed at least 2 word classes"
        );
        assert!(
            stats.total_patterns >= 3,
            "Should have parsed at least 3 patterns"
        );
    }

    #[test]
    fn test_parse_all_pattern_types() {
        let xml_content = create_lexicon_with_all_pattern_types();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(
            result.is_ok(),
            "Should parse lexicon with all pattern types successfully"
        );

        let resource = result.unwrap();
        let stats = resource.database.stats();
        assert!(
            stats.total_patterns >= 5,
            "Should have parsed all 5 pattern types"
        );
    }

    #[test]
    fn test_parse_complex_nested_xml() {
        let xml_content = create_complex_nested_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(
            result.is_ok(),
            "Should parse complex nested XML successfully"
        );

        let resource = result.unwrap();
        let validation = resource.validate();
        assert!(
            validation.is_ok(),
            "Complex nested resource should validate"
        );

        let stats = resource.database.stats();
        assert!(
            stats.total_word_classes >= 3,
            "Should have parsed all 3 complex classes"
        );
        assert!(
            stats.total_words >= 11,
            "Should have parsed all nested words"
        );
    }

    #[test]
    fn test_xml_with_different_versions() {
        let versions = vec!["1.0", "1.5", "2.0"];

        for version in versions {
            let xml_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="{}" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Version {} Test</title>
    <description>Testing version {}</description>
    <created>2024-01-01</created>
    <author>Version Tester</author>
    <license>MIT</license>
  </metadata>
  
  <word-classes>
    <word-class id="version-test" name="Version Test" type="stop-words" priority="10">
      <description>Testing version handling</description>
      <words>
        <word pos="DT">the</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#,
                version, version, version
            );

            let temp_dir = create_test_file(&xml_content);
            let file_path = temp_dir.path().join("test-lexicon.xml");

            let parser = XmlParser::new();
            let result = parser.parse_file::<LexiconXmlResource>(&file_path);

            assert!(
                result.is_ok(),
                "Should parse version {} successfully",
                version
            );

            let resource = result.unwrap();
            assert_eq!(resource.database.version, version);
        }
    }

    #[test]
    fn test_xml_with_different_languages() {
        let languages = vec!["en", "en-US", "fr", "es", "de"];

        for lang in languages {
            let xml_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="{}" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Language {} Test</title>
    <description>Testing language {}</description>
    <created>2024-01-01</created>
    <author>Language Tester</author>
    <license>MIT</license>
  </metadata>
  
  <word-classes>
    <word-class id="lang-test" name="Language Test" type="functional" priority="1">
      <description>Testing language handling</description>
      <words>
        <word pos="X">test</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#,
                lang, lang, lang
            );

            let temp_dir = create_test_file(&xml_content);
            let file_path = temp_dir.path().join("test-lexicon.xml");

            let parser = XmlParser::new();
            let result = parser.parse_file::<LexiconXmlResource>(&file_path);

            assert!(
                result.is_ok(),
                "Should parse language {} successfully",
                lang
            );

            let resource = result.unwrap();
            assert_eq!(resource.database.language, lang);
        }
    }

    #[test]
    fn test_xml_with_property_variations() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Property Variations Test</title>
    <description>Testing different property types</description>
    <created>2024-01-01</created>
    <author>Property Tester</author>
    <license>MIT</license>
  </metadata>
  
  <word-classes>
    <word-class id="prop-test" name="Property Test" type="intensifiers" priority="6">
      <description>Testing property variations</description>
      <properties>
        <property name="string-prop" value="test-string" type="string"/>
        <property name="int-prop" value="42" type="int"/>
        <property name="float-prop" value="3.14" type="float"/>
        <property name="bool-true" value="true" type="bool"/>
        <property name="bool-false" value="false" type="bool"/>
        <property name="negative-int" value="-10" type="int"/>
        <property name="negative-float" value="-2.5" type="float"/>
        <property name="zero-int" value="0" type="int"/>
        <property name="zero-float" value="0.0" type="float"/>
      </properties>
      <words>
        <word pos="RB">very</word>
        <word pos="RB">quite</word>
        <word pos="RB">extremely</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#;

        let temp_dir = create_test_file(xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(
            result.is_ok(),
            "Should parse property variations successfully"
        );

        let resource = result.unwrap();
        let stats = resource.database.stats();
        assert!(
            stats.total_words >= 3,
            "Should have parsed intensifier words"
        );
    }

    #[test]
    fn test_xml_with_word_attribute_variations() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Word Attribute Test</title>
    <description>Testing word attribute variations</description>
    <created>2024-01-01</created>
    <author>Attribute Tester</author>
    <license>MIT</license>
  </metadata>
  
  <word-classes>
    <word-class id="word-attr-test" name="Word Attribute Test" type="hedge-words" priority="4">
      <description>Testing various word attributes</description>
      <words>
        <word pos="RB" confidence="0.95" frequency="1000" context="uncertainty">maybe</word>
        <word pos="RB" confidence="0.90" frequency="500">perhaps</word>
        <word pos="RB" frequency="750" context="probability">probably</word>
        <word confidence="0.80" context="possibility">possibly</word>
        <word pos="RB" confidence="0.85">likely</word>
        <word>surely</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#;

        let temp_dir = create_test_file(xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(
            result.is_ok(),
            "Should parse word attribute variations successfully"
        );

        let resource = result.unwrap();
        let stats = resource.database.stats();
        assert!(
            stats.total_words >= 6,
            "Should have parsed all words with various attributes"
        );
    }

    #[test]
    fn test_xml_with_empty_elements() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Empty Elements Test</title>
    <description>Testing empty element handling</description>
    <created>2024-01-01</created>
    <author>Empty Tester</author>
    <license>MIT</license>
  </metadata>
  
  <word-classes>
    <word-class id="empty-test1" name="Empty Test One" type="sentiment" priority="3">
      <description>Class with empty words</description>
      <words>
      </words>
    </word-class>
    
    <word-class id="empty-test2" name="Empty Test Two" type="modal" priority="2">
      <description>Class with empty patterns</description>
      <words>
        <word pos="MD">can</word>
        <word pos="MD">will</word>
      </words>
      <patterns>
      </patterns>
    </word-class>
  </word-classes>
</lexicon>"#;

        let temp_dir = create_test_file(xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(result.is_ok(), "Should handle empty elements gracefully");

        let resource = result.unwrap();
        let stats = resource.database.stats();
        assert_eq!(stats.total_word_classes, 2);
        assert_eq!(stats.total_words, 2); // Only from the second class
    }

    #[test]
    fn test_xml_error_recovery() {
        // Test XML with some recoverable issues
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Error Recovery Test</title>
    <description>Testing error recovery</description>
    <created>2024-01-01</created>
    <author>Error Tester</author>
    <license>MIT</license>
  </metadata>
  
  <word-classes>
    <word-class id="recovery-test" name="Recovery Test" type="temporal" priority="5">
      <description>Testing recovery from minor issues</description>
      <words>
        <word pos="RB">now</word>
        <word pos="RB">then</word>
        <word pos="RB">today</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#;

        let temp_dir = create_test_file(xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(result.is_ok(), "Should recover from minor XML issues");

        let resource = result.unwrap();
        let validation = resource.validate();
        assert!(
            validation.is_ok(),
            "Resource should validate after recovery"
        );
    }

    #[test]
    fn test_parser_with_minimal_valid_xml() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Minimal</title>
    <description>Minimal valid lexicon</description>
    <created>2024-01-01</created>
    <author>Minimal</author>
    <license>MIT</license>
  </metadata>
  <word-classes>
  </word-classes>
</lexicon>"#;

        let temp_dir = create_test_file(xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(result.is_ok(), "Should parse minimal valid XML");

        let resource = result.unwrap();
        let stats = resource.database.stats();
        assert_eq!(stats.total_word_classes, 0);
        assert_eq!(stats.total_words, 0);
        assert_eq!(stats.total_patterns, 0);
    }
}
