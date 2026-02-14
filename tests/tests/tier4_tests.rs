// use std::path::PathBuf;
// use tyrus_common::fs::FilePath;

#[test]
fn test_tier4_nestjs_extraction() {
    let current_dir = std::env::current_dir().expect("Failed to get CWD");
    println!("CWD: {:?}", current_dir);
    // CWD is crates/tests (integration_tests), so fixtures are in fixtures/
    let input_path = current_dir.join("fixtures/tier4_nestjs_di/input.ts");

    // Ensure file exists before parsing
    assert!(
        input_path.exists(),
        "Input file does not exist at: {:?}",
        input_path
    );

    let program = tyrus_parser::parse(&input_path).expect("Failed to parse");
    let source_code = std::fs::read_to_string(&input_path).expect("Failed to read source");

    let analysis = tyrus_analyzer::Analyzer::analyze(
        &program,
        source_code,
        input_path.to_string_lossy().to_string(),
    );

    let graph = analysis.graph;

    // Verify Module Extraction
    let cats_module = graph
        .get_module("CatsModule")
        .expect("CatsModule not found in graph");

    assert_eq!(cats_module.name, "CatsModule");

    // Verify Providers
    let provider = cats_module
        .providers
        .iter()
        .find(|p| p.token == "CatsService")
        .expect("CatsService provider not found");

    assert_eq!(provider.token, "CatsService");
    assert_eq!(provider.implementation, "CatsService");

    // Verify Controllers
    assert!(cats_module
        .controllers
        .contains(&"CatsController".to_string()));

    // Verify Injectable Definitions (Implicitly tested by resolve)
    // Let's resolve the graph
    let init_order = graph.resolve().expect("Failed to resolve graph");

    // CatsController depends on CatsService.
    // So CatsService must come before CatsController.
    let service_idx = init_order
        .iter()
        .position(|r| r == "CatsService")
        .expect("CatsService missing");
    let controller_idx = init_order
        .iter()
        .position(|r| r == "CatsController")
        .expect("CatsController missing");

    assert!(
        service_idx < controller_idx,
        "CatsService should be initialized before CatsController"
    );
}

#[test]
fn test_tier4_full_build() {
    let current_dir = std::env::current_dir().expect("Failed to get CWD");
    let fixture_path = current_dir.join("fixtures/tier4_nestjs_di");
    let output_dir = current_dir.join("target/test_output/tier4_full_build");

    // Clean up output dir
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("Failed to clean output dir");
    }
    std::fs::create_dir_all(&output_dir).expect("Failed to create output dir");

    // Run build_project
    // We need to use tyrus_orchestrator::build_project
    // But tyrus_orchestrator is an external crate to integration_tests?
    // integration_tests/Cargo.toml has tyrus_orchestrator dependency.

    tyrus_orchestrator::build_project(fixture_path, output_dir.clone())
        .expect("build_project failed");

    // Verify main.rs content
    let main_rs_path = output_dir.join("src/main.rs");
    assert!(main_rs_path.exists(), "main.rs not generated");

    let main_content = std::fs::read_to_string(&main_rs_path).expect("Failed to read main.rs");

    // Check for Service Instantiation
    // allowed both qualified and unqualified depending on implementation
    let has_service = main_content
        .contains("let cats_service = Arc::new(tyrus_app::input::CatsService::new_di());")
        || main_content.contains(
            "let cats_service = std::sync::Arc::new(tyrus_app::input::CatsService::new_di());",
        );

    assert!(
        has_service,
        "CatsService instantiation missing or incorrect: {}",
        main_content
    );

    // Check for Controller Instantiation with Dependency
    let has_controller = main_content.contains("let cats_controller = Arc::new(tyrus_app::input::CatsController::new_di(cats_service.clone()));") ||
                         main_content.contains("let cats_controller = std::sync::Arc::new(tyrus_app::input::CatsController::new_di(cats_service.clone()));");

    assert!(
        has_controller,
        "CatsController instantiation missing or incorrect: {}",
        main_content
    );

    // Check for Router Merge
    assert!(
        main_content.contains(".merge(tyrus_app::input::CatsController::router())"),
        "Router merge missing"
    );

    // Check for Extension Layer
    assert!(
        main_content.contains(".layer(Extension(cats_controller.clone()))")
            || main_content.contains(".layer(axum::Extension(cats_controller.clone()))"),
        "Controller extension layer missing"
    );
}
