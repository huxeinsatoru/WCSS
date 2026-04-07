use wcss_compiler::content_scanner::{scan_content_paths, ContentScanner, ScanConfig};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_scan_html_file() {
    let temp_dir = TempDir::new().unwrap();
    let html_file = temp_dir.path().join("index.html");
    
    fs::write(&html_file, r#"
        <!DOCTYPE html>
        <html>
        <body>
            <div class="container">
                <button class="btn btn-primary">Click me</button>
                <div class="card shadow-lg">Content</div>
            </div>
        </body>
        </html>
    "#).unwrap();

    let pattern = html_file.to_str().unwrap().to_string();
    let classes = scan_content_paths(vec![pattern]).unwrap();

    assert!(classes.contains("container"));
    assert!(classes.contains("btn"));
    assert!(classes.contains("btn-primary"));
    assert!(classes.contains("card"));
    assert!(classes.contains("shadow-lg"));
}

#[test]
fn test_scan_jsx_file() {
    let temp_dir = TempDir::new().unwrap();
    let jsx_file = temp_dir.path().join("Button.jsx");
    
    fs::write(&jsx_file, r#"
        export function Button({ variant }) {
            return (
                <button className="btn btn-large hover:bg-blue">
                    Click me
                </button>
            );
        }
    "#).unwrap();

    let pattern = jsx_file.to_str().unwrap().to_string();
    let classes = scan_content_paths(vec![pattern]).unwrap();

    assert!(classes.contains("btn"));
    assert!(classes.contains("btn-large"));
    assert!(classes.contains("hover:bg-blue"));
}

#[test]
fn test_scan_tsx_file() {
    let temp_dir = TempDir::new().unwrap();
    let tsx_file = temp_dir.path().join("Card.tsx");
    
    fs::write(&tsx_file, r#"
        interface CardProps {
            title: string;
        }

        export const Card: React.FC<CardProps> = ({ title }) => {
            return (
                <div className="card rounded-lg shadow-md p-4">
                    <h2 className="text-xl font-bold">{title}</h2>
                </div>
            );
        };
    "#).unwrap();

    let pattern = tsx_file.to_str().unwrap().to_string();
    let classes = scan_content_paths(vec![pattern]).unwrap();

    assert!(classes.contains("card"));
    assert!(classes.contains("rounded-lg"));
    assert!(classes.contains("shadow-md"));
    assert!(classes.contains("p-4"));
    assert!(classes.contains("text-xl"));
    assert!(classes.contains("font-bold"));
}

#[test]
fn test_scan_vue_file() {
    let temp_dir = TempDir::new().unwrap();
    let vue_file = temp_dir.path().join("Button.vue");
    
    fs::write(&vue_file, r#"
        <template>
            <button :class="['btn', 'btn-primary']">
                Click me
            </button>
        </template>
    "#).unwrap();

    let pattern = vue_file.to_str().unwrap().to_string();
    let classes = scan_content_paths(vec![pattern]).unwrap();

    assert!(classes.contains("btn"));
    assert!(classes.contains("btn-primary"));
}

#[test]
fn test_scan_svelte_file() {
    let temp_dir = TempDir::new().unwrap();
    let svelte_file = temp_dir.path().join("Button.svelte");
    
    fs::write(&svelte_file, r#"
        <script>
            let active = true;
        </script>

        <button class:active class:btn>
            Click me
        </button>
    "#).unwrap();

    let pattern = svelte_file.to_str().unwrap().to_string();
    let classes = scan_content_paths(vec![pattern]).unwrap();

    assert!(classes.contains("active"));
    assert!(classes.contains("btn"));
}

#[test]
fn test_scan_template_literal() {
    let temp_dir = TempDir::new().unwrap();
    let jsx_file = temp_dir.path().join("Dynamic.jsx");
    
    fs::write(&jsx_file, r#"
        export function Dynamic({ variant }) {
            return (
                <div className={`btn btn-${variant}`}>
                    Dynamic class
                </div>
            );
        }
    "#).unwrap();

    let pattern = jsx_file.to_str().unwrap().to_string();
    let classes = scan_content_paths(vec![pattern]).unwrap();

    // Should extract "btn" but not the dynamic part
    assert!(classes.contains("btn"));
    // Dynamic parts like ${variant} are not extracted
}

#[test]
#[ignore] // TODO: Improve clsx regex to handle multiple arguments
fn test_scan_clsx_function() {
    let temp_dir = TempDir::new().unwrap();
    let jsx_file = temp_dir.path().join("Clsx.jsx");
    
    fs::write(&jsx_file, r#"
        import clsx from 'clsx';

        export function Component() {
            const classes = clsx('btn', 'btn-primary');
            return <div className={classes}>Content</div>;
        }
    "#).unwrap();

    let pattern = jsx_file.to_str().unwrap().to_string();
    let classes = scan_content_paths(vec![pattern]).unwrap();

    assert!(classes.contains("btn"));
    assert!(classes.contains("btn-primary"));
}

#[test]
fn test_scan_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    
    let file1 = temp_dir.path().join("file1.html");
    fs::write(&file1, r#"<div class="container">Content</div>"#).unwrap();
    
    let file2 = temp_dir.path().join("file2.jsx");
    fs::write(&file2, r#"<button className="btn">Click</button>"#).unwrap();

    // Use separate patterns for each file type
    let pattern1 = file1.to_str().unwrap().to_string();
    let pattern2 = file2.to_str().unwrap().to_string();
    let classes = scan_content_paths(vec![pattern1, pattern2]).unwrap();

    assert!(classes.contains("container"));
    assert!(classes.contains("btn"));
}

#[test]
fn test_safelist_patterns() {
    use regex::Regex;

    let config = ScanConfig {
        patterns: vec![],
        safelist_patterns: vec![
            Regex::new(r"^btn-").unwrap(),
            Regex::new(r"^text-").unwrap(),
        ],
        recursive: true,
    };

    let scanner = ContentScanner::new(config);

    assert!(scanner.matches_safelist("btn-primary"));
    assert!(scanner.matches_safelist("btn-large"));
    assert!(scanner.matches_safelist("text-xl"));
    assert!(scanner.matches_safelist("text-2xl"));
    assert!(!scanner.matches_safelist("card"));
    assert!(!scanner.matches_safelist("container"));
}

#[test]
fn test_invalid_class_names_filtered() {
    let temp_dir = TempDir::new().unwrap();
    let html_file = temp_dir.path().join("invalid.html");
    
    // Include some invalid class names
    fs::write(&html_file, r#"
        <div class="valid-class 123invalid class with spaces">
            Content
        </div>
    "#).unwrap();

    let pattern = html_file.to_str().unwrap().to_string();
    let classes = scan_content_paths(vec![pattern]).unwrap();

    assert!(classes.contains("valid-class"));
    assert!(!classes.contains("123invalid")); // starts with digit
    assert!(!classes.contains("class with spaces")); // contains spaces
}

#[test]
fn test_glob_pattern_matching() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create nested directory structure
    let src_dir = temp_dir.path().join("src");
    let components_dir = src_dir.join("components");
    fs::create_dir_all(&components_dir).unwrap();
    
    let file1 = components_dir.join("Button.jsx");
    fs::write(&file1, r#"<button className="btn">Click</button>"#).unwrap();
    
    let file2 = components_dir.join("Card.jsx");
    fs::write(&file2, r#"<div className="card">Content</div>"#).unwrap();

    // Use glob pattern to match all JSX files
    let pattern = format!("{}/**/*.jsx", temp_dir.path().display());
    let classes = scan_content_paths(vec![pattern]).unwrap();

    assert!(classes.contains("btn"));
    assert!(classes.contains("card"));
}
