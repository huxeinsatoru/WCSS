use wcss_compiler::{compile, config::CompilerConfig};

#[test]
fn test_tailwind_base_directive() {
    let source = r#"
        @tailwind base;
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    assert!(result.errors.is_empty(), "Compilation errors: {:?}", result.errors);
    assert!(result.css.contains("@tailwind base;"));
}

#[test]
fn test_tailwind_components_directive() {
    let source = r#"
        @tailwind components;
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    assert!(result.errors.is_empty(), "Compilation errors: {:?}", result.errors);
    assert!(result.css.contains("@tailwind components;"));
}

#[test]
fn test_tailwind_utilities_directive() {
    let source = r#"
        @tailwind utilities;
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    assert!(result.errors.is_empty(), "Compilation errors: {:?}", result.errors);
    assert!(result.css.contains("@tailwind utilities;"));
}

#[test]
fn test_apply_directive_basic() {
    let source = r#"
        .btn {
            @apply px-4 py-2 bg-blue-500 text-white rounded;
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    assert!(result.errors.is_empty(), "Compilation errors: {:?}", result.errors);
    assert!(result.css.contains("@apply px-4 py-2 bg-blue-500 text-white rounded;"));
}

#[test]
fn test_apply_with_hover_modifier() {
    let source = r#"
        .btn-primary {
            @apply px-4 py-2 bg-blue-500 hover:bg-blue-600;
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    assert!(result.errors.is_empty(), "Compilation errors: {:?}", result.errors);
    assert!(result.css.contains("@apply px-4 py-2 bg-blue-500 hover:bg-blue-600;"));
}

#[test]
fn test_tailwind_with_layer() {
    let source = r#"
        @tailwind base;
        @tailwind components;
        @tailwind utilities;

        @layer components {
            .card {
                @apply p-4 bg-white shadow rounded;
            }
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    assert!(result.errors.is_empty(), "Compilation errors: {:?}", result.errors);
    assert!(result.css.contains("@tailwind base;"));
    assert!(result.css.contains("@tailwind components;"));
    assert!(result.css.contains("@tailwind utilities;"));
    assert!(result.css.contains("@layer components"));
    assert!(result.css.contains("@apply p-4 bg-white shadow rounded;"));
}

#[test]
fn test_mixed_wcss_and_tailwind() {
    // Test simpler case first
    let source1 = r#"
        .custom-button {
            padding: 1rem;
        }
    "#;
    
    let config = CompilerConfig::default();
    let result1 = compile(source1, &config);
    assert!(result1.errors.is_empty(), "Simple test failed: {:?}", result1.errors);
    
    // Test with @apply only
    let source2 = r#"
        .custom-button {
            @apply rounded;
        }
    "#;
    
    let result2 = compile(source2, &config);
    assert!(result2.errors.is_empty(), "Apply only test failed: {:?}", result2.errors);
    
    // Test mixed
    let source3 = r#"
        .custom-button {
            padding: 1rem;
            @apply rounded;
        }
    "#;
    
    let result3 = compile(source3, &config);
    if !result3.errors.is_empty() {
        eprintln!("Mixed test errors: {:?}", result3.errors);
    }
    assert!(result3.errors.is_empty(), "Mixed test failed: {:?}", result3.errors);
}

#[test]
fn test_apply_with_regular_properties() {
    let source = r#"
        .mixed {
            color: red;
            @apply px-4 py-2;
            font-size: 16px;
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    assert!(result.errors.is_empty(), "Compilation errors: {:?}", result.errors);
    eprintln!("Generated CSS:\n{}", result.css);
    assert!(result.css.contains("color: red;") || result.css.contains("color:red;"));
    assert!(result.css.contains("@apply px-4 py-2;") || result.css.contains("@apply px-4 py-2"));
    assert!(result.css.contains("font-size: 16px;") || result.css.contains("font-size:16px;"));
}

#[test]
fn test_tailwind_all_directives() {
    let source = r#"
        @tailwind base;
        @tailwind components;
        @tailwind utilities;
        @tailwind variants;
        @tailwind screens;
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    assert!(result.errors.is_empty(), "Compilation errors: {:?}", result.errors);
    assert!(result.css.contains("@tailwind base;"));
    assert!(result.css.contains("@tailwind components;"));
    assert!(result.css.contains("@tailwind utilities;"));
    assert!(result.css.contains("@tailwind variants;"));
    assert!(result.css.contains("@tailwind screens;"));
}

#[test]
fn test_apply_in_nested_layer() {
    let source = r#"
        @layer components {
            .btn {
                @apply px-4 py-2 rounded;
            }
            
            .btn-primary {
                @apply bg-blue-500 text-white;
            }
        }
    "#;
    
    let config = CompilerConfig::default();
    let result = compile(source, &config);
    assert!(result.errors.is_empty(), "Compilation errors: {:?}", result.errors);
    assert!(result.css.contains("@layer components"));
    assert!(result.css.contains("@apply px-4 py-2 rounded;"));
    assert!(result.css.contains("@apply bg-blue-500 text-white;"));
}
