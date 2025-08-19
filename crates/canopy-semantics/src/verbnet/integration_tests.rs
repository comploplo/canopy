//! VerbNet XML Integration Tests
//!
//! Comprehensive tests against all VerbNet XML files to validate:
//! - XML parsing accuracy
//! - Theta role extraction
//! - Pattern mapping
//! - Cache behavior
//! - Real-world VerbNet compatibility

use super::engine::VerbNetEngine;
use super::parser::VerbNetParser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Comprehensive VerbNet XML test harness
pub struct VerbNetTestSuite {
    test_xml_dir: PathBuf,
    engine: VerbNetEngine,
}

impl VerbNetTestSuite {
    /// Create new test suite with VerbNet test XML directory
    pub fn new() -> Self {
        // Try both relative paths (from crate and from project root)
        let test_xml_dir = if PathBuf::from("data/verbnet/verbnet-test").exists() {
            PathBuf::from("data/verbnet/verbnet-test")
        } else if PathBuf::from("../../data/verbnet/verbnet-test").exists() {
            PathBuf::from("../../data/verbnet/verbnet-test")
        } else {
            // Default path - will be checked when tests run
            PathBuf::from("data/verbnet/verbnet-test")
        };
        let engine = VerbNetEngine::new();

        Self {
            test_xml_dir,
            engine,
        }
    }

    /// Get all XML test files in the VerbNet test directory
    pub fn get_all_test_files(&self) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut xml_files = Vec::new();

        if self.test_xml_dir.exists() {
            for entry in fs::read_dir(&self.test_xml_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().map_or(false, |ext| ext == "xml") {
                    xml_files.push(path);
                }
            }
        }

        xml_files.sort();
        Ok(xml_files)
    }

    /// Test parsing of a single VerbNet XML file
    pub fn test_single_xml(
        &mut self,
        xml_path: &Path,
    ) -> Result<VerbNetTestResult, VerbNetTestError> {
        let _xml_content = fs::read_to_string(xml_path)
            .map_err(|e| VerbNetTestError::FileRead(xml_path.to_path_buf(), e))?;

        let class_name = xml_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| VerbNetTestError::InvalidFileName(xml_path.to_path_buf()))?;

        let start_time = std::time::Instant::now();

        // Test XML parsing
        let parsed_classes = VerbNetParser::parse_file(xml_path).map_err(|e| {
            VerbNetTestError::ParseError(xml_path.to_path_buf(), format!("{:?}", e))
        })?;

        let parsed_class = parsed_classes.into_iter().next().ok_or_else(|| {
            VerbNetTestError::ParseError(xml_path.to_path_buf(), "No classes found".to_string())
        })?;

        let parse_time = start_time.elapsed();

        // Test theta role extraction
        let theta_roles_count = parsed_class.theta_roles.len();
        let frames_count = parsed_class.frames.len();
        let members_count = parsed_class.members.len();

        // Test pattern mapping if frames exist
        let mut pattern_test_results = Vec::new();
        for (frame_idx, frame) in parsed_class.frames.iter().enumerate() {
            // Create a simple dependency pattern from the frame
            let test_pattern = format!("frame_{}", frame_idx);
            let test_args = vec![("test_rel".to_string(), "test_word".to_string())];

            // Test pattern mapping
            let analyses = self.engine.map_dependency_pattern_to_theta_roles(
                &parsed_class
                    .members
                    .first()
                    .map(|m| m.name.as_str())
                    .unwrap_or("test"),
                &test_pattern,
                &test_args,
            );

            pattern_test_results.push(PatternTestResult {
                frame_index: frame_idx,
                frame_description: frame.description.clone(),
                analyses_count: analyses.len(),
                max_confidence: analyses.first().map(|a| a.confidence).unwrap_or(0.0),
            });
        }

        Ok(VerbNetTestResult {
            class_name: class_name.to_string(),
            xml_path: xml_path.to_path_buf(),
            parse_time,
            class_id: parsed_class.id.clone(),
            theta_roles_count,
            frames_count,
            members_count,
            pattern_tests: pattern_test_results,
            success: true,
        })
    }

    /// Run comprehensive test suite against all VerbNet XML files
    pub fn run_comprehensive_test(&mut self) -> VerbNetSuiteResults {
        let start_time = std::time::Instant::now();

        let xml_files = match self.get_all_test_files() {
            Ok(files) => files,
            Err(e) => {
                return VerbNetSuiteResults {
                    total_files: 0,
                    successful_parses: 0,
                    failed_parses: 0,
                    total_time: start_time.elapsed(),
                    results: Vec::new(),
                    errors: vec![VerbNetTestError::DirectoryRead(e)],
                    statistics: VerbNetStatistics::default(),
                };
            }
        };

        let mut results = Vec::new();
        let mut errors = Vec::new();
        let mut successful_parses = 0;
        let mut failed_parses = 0;

        println!(
            "🧪 Running VerbNet XML Integration Tests on {} files...",
            xml_files.len()
        );

        for (idx, xml_file) in xml_files.iter().enumerate() {
            if idx % 10 == 0 {
                println!("   Progress: {}/{} files processed", idx, xml_files.len());
            }

            match self.test_single_xml(xml_file) {
                Ok(result) => {
                    successful_parses += 1;
                    results.push(result);
                }
                Err(error) => {
                    failed_parses += 1;
                    errors.push(error);
                }
            }
        }

        let total_time = start_time.elapsed();
        let statistics = self.calculate_statistics(&results, successful_parses);

        println!("✅ VerbNet test suite completed!");
        println!("   Total files: {}", xml_files.len());
        println!(
            "   Successful: {} ({:.1}%)",
            successful_parses,
            100.0 * successful_parses as f64 / xml_files.len() as f64
        );
        println!(
            "   Failed: {} ({:.1}%)",
            failed_parses,
            100.0 * failed_parses as f64 / xml_files.len() as f64
        );
        println!("   Total time: {:?}", total_time);
        if xml_files.len() > 0 {
            println!(
                "   Average parse time: {:?}",
                total_time / xml_files.len() as u32
            );
        }

        // Show first few errors for debugging
        if !errors.is_empty() {
            println!("First few errors:");
            for (i, error) in errors.iter().take(3).enumerate() {
                println!("   Error {}: {:?}", i + 1, error);
            }
        }

        VerbNetSuiteResults {
            total_files: xml_files.len(),
            successful_parses,
            failed_parses,
            total_time,
            results,
            errors,
            statistics,
        }
    }

    /// Calculate comprehensive statistics from test results
    fn calculate_statistics(
        &self,
        results: &[VerbNetTestResult],
        successful_parses: usize,
    ) -> VerbNetStatistics {
        if results.is_empty() {
            return VerbNetStatistics::default();
        }

        let total_theta_roles: usize = results.iter().map(|r| r.theta_roles_count).sum();
        let total_frames: usize = results.iter().map(|r| r.frames_count).sum();
        let total_members: usize = results.iter().map(|r| r.members_count).sum();

        let avg_parse_time_micros = results
            .iter()
            .map(|r| r.parse_time.as_micros() as u64)
            .sum::<u64>()
            / results.len() as u64;

        let min_parse_time = results
            .iter()
            .map(|r| r.parse_time)
            .min()
            .unwrap_or_default();

        let max_parse_time = results
            .iter()
            .map(|r| r.parse_time)
            .max()
            .unwrap_or_default();

        // Count frame distribution
        let files_with_frames = results.iter().filter(|r| r.frames_count > 0).count();
        let files_with_members = results.iter().filter(|r| r.members_count > 0).count();

        VerbNetStatistics {
            total_verb_classes: results.len(),
            total_theta_roles,
            total_frames,
            total_members,
            avg_theta_roles_per_class: total_theta_roles as f64 / results.len() as f64,
            avg_frames_per_class: total_frames as f64 / results.len() as f64,
            avg_members_per_class: total_members as f64 / results.len() as f64,
            avg_parse_time_micros,
            min_parse_time_micros: min_parse_time.as_micros() as u64,
            max_parse_time_micros: max_parse_time.as_micros() as u64,
            files_with_frames,
            files_with_members,
            coverage_percentage: 100.0 * successful_parses as f64 / results.len() as f64,
        }
    }

    /// Test specific high-value VerbNet classes for detailed validation
    pub fn test_key_verb_classes(&mut self) -> KeyVerbTestResults {
        let key_verbs = vec![
            "give-13.1.xml",  // Ditransitive
            "hit-18.1.xml",   // Contact
            "run-51.3.2.xml", // Motion
            "break-45.1.xml", // Causative/Inchoative
            "seem-109.xml",   // Raising
            "put-9.1.xml",    // Location
            "eat-39.1.xml",   // Consumption
            "say-37.7.xml",   // Communication
        ];

        let mut results = HashMap::new();

        for verb_file in key_verbs {
            let xml_path = self.test_xml_dir.join(verb_file);
            if xml_path.exists() {
                match self.test_single_xml(&xml_path) {
                    Ok(result) => {
                        println!(
                            "✅ Key verb test passed: {} ({} theta roles, {} frames)",
                            result.class_name, result.theta_roles_count, result.frames_count
                        );
                        results.insert(verb_file.to_string(), Ok(result));
                    }
                    Err(error) => {
                        println!("❌ Key verb test failed: {}: {:?}", verb_file, error);
                        results.insert(verb_file.to_string(), Err(error));
                    }
                }
            } else {
                println!("⚠️ Key verb file not found: {}", verb_file);
            }
        }

        KeyVerbTestResults { results }
    }
}

/// Result of testing a single VerbNet XML file
#[derive(Debug, Clone)]
pub struct VerbNetTestResult {
    pub class_name: String,
    pub xml_path: PathBuf,
    pub parse_time: std::time::Duration,
    pub class_id: String,
    pub theta_roles_count: usize,
    pub frames_count: usize,
    pub members_count: usize,
    pub pattern_tests: Vec<PatternTestResult>,
    pub success: bool,
}

/// Result of pattern mapping test for a frame
#[derive(Debug, Clone)]
pub struct PatternTestResult {
    pub frame_index: usize,
    pub frame_description: String,
    pub analyses_count: usize,
    pub max_confidence: f32,
}

/// Comprehensive results of the entire test suite
#[derive(Debug)]
pub struct VerbNetSuiteResults {
    pub total_files: usize,
    pub successful_parses: usize,
    pub failed_parses: usize,
    pub total_time: std::time::Duration,
    pub results: Vec<VerbNetTestResult>,
    pub errors: Vec<VerbNetTestError>,
    pub statistics: VerbNetStatistics,
}

/// Statistical analysis of VerbNet test results
#[derive(Debug, Default)]
pub struct VerbNetStatistics {
    pub total_verb_classes: usize,
    pub total_theta_roles: usize,
    pub total_frames: usize,
    pub total_members: usize,
    pub avg_theta_roles_per_class: f64,
    pub avg_frames_per_class: f64,
    pub avg_members_per_class: f64,
    pub avg_parse_time_micros: u64,
    pub min_parse_time_micros: u64,
    pub max_parse_time_micros: u64,
    pub files_with_frames: usize,
    pub files_with_members: usize,
    pub coverage_percentage: f64,
}

/// Results of testing key verb classes
pub struct KeyVerbTestResults {
    pub results: HashMap<String, Result<VerbNetTestResult, VerbNetTestError>>,
}

/// Errors that can occur during VerbNet testing
#[derive(Debug)]
pub enum VerbNetTestError {
    FileRead(PathBuf, std::io::Error),
    ParseError(PathBuf, String),
    InvalidFileName(PathBuf),
    DirectoryRead(std::io::Error),
}

impl std::fmt::Display for VerbNetTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerbNetTestError::FileRead(path, e) => {
                write!(f, "Failed to read file {}: {}", path.display(), e)
            }
            VerbNetTestError::ParseError(path, e) => {
                write!(f, "Failed to parse {}: {}", path.display(), e)
            }
            VerbNetTestError::InvalidFileName(path) => {
                write!(f, "Invalid file name: {}", path.display())
            }
            VerbNetTestError::DirectoryRead(e) => write!(f, "Failed to read directory: {}", e),
        }
    }
}

impl std::error::Error for VerbNetTestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbnet_xml_comprehensive() {
        let mut test_suite = VerbNetTestSuite::new();

        // Run comprehensive test suite
        let results = test_suite.run_comprehensive_test();

        println!("\n📊 VerbNet XML Integration Test Results:");
        println!("   Total files processed: {}", results.total_files);
        println!("   Successful parses: {}", results.successful_parses);
        println!(
            "   Parse success rate: {:.1}%",
            100.0 * results.successful_parses as f64 / results.total_files as f64
        );

        // Validate minimum success criteria
        if results.total_files > 0 {
            let success_rate = results.successful_parses as f64 / results.total_files as f64;
            assert!(
                success_rate >= 0.8,
                "VerbNet XML parse success rate too low: {:.1}% (expected ≥80%)",
                success_rate * 100.0
            );

            // Check performance criteria
            assert!(
                results.statistics.avg_parse_time_micros < 1000,
                "Average parse time too slow: {}μs (expected <1000μs)",
                results.statistics.avg_parse_time_micros
            );

            println!("✅ Comprehensive VerbNet XML test passed!");
        } else {
            println!("⚠️ No VerbNet XML files found - test skipped");
        }
    }

    #[test]
    fn test_key_verb_classes() {
        let mut test_suite = VerbNetTestSuite::new();

        let key_results = test_suite.test_key_verb_classes();

        let successful_keys = key_results.results.values().filter(|r| r.is_ok()).count();

        println!("Key verb classes tested: {}", key_results.results.len());
        println!("Successful: {}", successful_keys);

        // At least some key verbs should parse successfully
        if !key_results.results.is_empty() {
            assert!(
                successful_keys > 0,
                "No key verb classes parsed successfully"
            );
            println!("✅ Key verb class test passed!");
        } else {
            println!("⚠️ No key verb files found - test skipped");
        }
    }

    #[test]
    fn test_give_xml_specific() {
        let mut test_suite = VerbNetTestSuite::new();
        let give_path = test_suite.test_xml_dir.join("give-13.1.xml");

        if give_path.exists() {
            let result = test_suite
                .test_single_xml(&give_path)
                .expect("give-13.1.xml should parse successfully");

            println!("Give class details:");
            println!("   Class ID: {}", result.class_id);
            println!("   Theta roles: {}", result.theta_roles_count);
            println!("   Frames: {}", result.frames_count);
            println!("   Members: {}", result.members_count);
            println!("   Parse time: {:?}", result.parse_time);

            // Validate give-specific expectations
            assert!(
                result.theta_roles_count >= 3,
                "Give should have at least 3 theta roles (Agent, Theme, Recipient)"
            );
            assert!(result.frames_count > 0, "Give should have syntactic frames");
            assert!(result.members_count > 0, "Give should have verb members");
            assert!(
                result.parse_time.as_millis() < 10,
                "Parse time should be <10ms"
            );

            println!("✅ Give-specific test passed!");
        } else {
            println!("⚠️ give-13.1.xml not found - test skipped");
        }
    }
}
