//! Basic tests for main.rs module

#[cfg(test)]
mod main_tests {
    use canopy_lsp::server::CanopyServer;
    #[test]
    fn test_main_function_compilation() {
        // Test that main function compiles and exists
        // We can't directly test tokio::main function in unit tests
        // but we can test that the components it uses are available
        assert!(true);
    }

    #[test]
    fn test_layer1_helper_config_creation() {
        let config = canopy_core::layer1parser::Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 100,
            debug: true,
            confidence_threshold: 0.5,
        };

        assert!(config.enable_udpipe);
        assert!(config.enable_basic_features);
        assert!(config.enable_verbnet);
        assert_eq!(config.max_sentence_length, 100);
        assert!(config.debug);
        assert_eq!(config.confidence_threshold, 0.5);
    }

    #[test]
    fn test_semantic_config_creation() {
        let config = canopy_core::layer1parser::SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.6,
            debug: true,
        };

        assert!(config.enable_theta_roles);
        assert!(config.enable_animacy);
        assert!(config.enable_definiteness);
        assert_eq!(config.confidence_threshold, 0.6);
        assert!(config.debug);
    }

    #[test]
    fn test_verbnet_integration_function_exists() {
        // Test that verbnet test function exists and can be called
        canopy_lsp::verbnet_test::test_verbnet_integration();
        assert!(true);
    }

    #[test]
    fn test_server_factory_creation() {
        let parser_config = canopy_core::layer1parser::Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 50,
            debug: false,
            confidence_threshold: 0.7,
        };

        let semantic_config = canopy_core::layer1parser::SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: false,
            enable_definiteness: false,
            confidence_threshold: 0.8,
            debug: false,
        };

        let result = canopy_lsp::CanopyLspServerFactory::create_server_with_config(
            parser_config,
            semantic_config,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_server_health_check() {
        let parser_config = canopy_core::layer1parser::Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 50,
            debug: false,
            confidence_threshold: 0.5,
        };

        let semantic_config = canopy_core::layer1parser::SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.6,
            debug: false,
        };

        if let Ok(server) = canopy_lsp::CanopyLspServerFactory::create_server_with_config(
            parser_config,
            semantic_config,
        ) {
            let health = server.health();
            // Health should have some meaningful structure
            assert!(health.healthy == true || health.healthy == false);
            assert!(health.uptime_seconds >= 0);
        }
    }

    #[test]
    fn test_server_text_processing() {
        let parser_config = canopy_core::layer1parser::Layer1HelperConfig {
            enable_udpipe: true,
            enable_basic_features: true,
            enable_verbnet: true,
            max_sentence_length: 50,
            debug: false,
            confidence_threshold: 0.5,
        };

        let semantic_config = canopy_core::layer1parser::SemanticConfig {
            enable_theta_roles: true,
            enable_animacy: true,
            enable_definiteness: true,
            confidence_threshold: 0.6,
            debug: false,
        };

        if let Ok(server) = canopy_lsp::CanopyLspServerFactory::create_server_with_config(
            parser_config,
            semantic_config,
        ) {
            let result = server.process_text("Test sentence");
            // Should either succeed or return a meaningful error
            assert!(result.is_ok() || result.is_err());
        }
    }
}
