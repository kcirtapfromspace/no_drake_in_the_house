use std::env;

fn main() {
    // Enable offline mode for SQLx if .sqlx directory has query cache files
    // SQLx 0.8 uses individual query-*.json files
    let sqlx_dir = std::path::Path::new(".sqlx");
    let has_query_cache = sqlx_dir.is_dir()
        && sqlx_dir
            .read_dir()
            .map(|mut entries| {
                entries.any(|e| {
                    e.ok()
                        .map(|e| e.file_name().to_string_lossy().starts_with("query-"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

    if has_query_cache {
        println!("cargo:rustc-env=SQLX_OFFLINE=true");
    }

    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=.env.test");
    println!("cargo:rerun-if-changed=.sqlx/");
}
