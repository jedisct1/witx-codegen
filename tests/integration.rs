

use witx_codegen::{Config, OutputType, generate};

const WITX_SOURCES: &[&str] = &[
    "test_module.witx",
    "wasi_ephemeral_crypto_common.witx",
    "wasi_ephemeral_crypto_symmetric.witx",
    "wasi_experimental_http.witx",
];

const WITX_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn witx_parse() {
    for s in WITX_SOURCES {
        let p = format!("{}/tests/{}", WITX_DIR, s);

        println!("Parsing {}", p);

        let _witx = witx::load(p).unwrap();
    }
}

#[test]
fn generate_rust() {
    let mut c = Config{
        output_type: OutputType::Rust,
        output_file: Some("/dev/null".to_string()),
        ..Default::default()
    };

    for s in WITX_SOURCES {
        println!("Generate {}", s);

        let p = format!("{}/tests/{}", WITX_DIR, s);
        c.witx_files = vec![p];

        generate(&c).unwrap();
    }
}

#[test]
fn generate_zig() {
    let mut c = Config{
        output_type: OutputType::Zig,
        output_file: Some("/dev/null".to_string()),
        ..Default::default()
    };

    for s in WITX_SOURCES {
        println!("Generate {}", s);

        let p = format!("{}/tests/{}", WITX_DIR, s);
        c.witx_files = vec![p];

        generate(&c).unwrap();
    }
}

#[test]
fn generate_doc() {
    let mut c = Config{
        output_type: OutputType::Doc,
        output_file: Some("/dev/null".to_string()),
        ..Default::default()
    };

    for s in WITX_SOURCES {
        println!("Generate {}", s);

        let p = format!("{}/tests/{}", WITX_DIR, s);
        c.witx_files = vec![p];

        generate(&c).unwrap();
    }
}

#[test]
fn generate_assemblyscript() {
    let mut c = Config{
        output_type: OutputType::AssemblyScript,
        output_file: Some("/dev/null".to_string()),
        ..Default::default()
    };

    for s in WITX_SOURCES {
        println!("Generate {}", s);

        let p = format!("{}/tests/{}", WITX_DIR, s);
        c.witx_files = vec![p];

        generate(&c).unwrap();
    }
}
