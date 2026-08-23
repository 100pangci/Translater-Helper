fn main() {
    tauri_build::build();

    // 版本号逻辑（与 VNDB-GUI-Rs 一致）：
    // 1. 优先使用构建环境变量 TRANSLATER_HELPER_VERSION（CI 打标签时由 workflow 注入，如 v0.1.0）
    // 2. 其次读取仓库根目录 version.txt（本地开发构建，如 0.1.0）
    // 3. 兜底为 dev
    println!("cargo:rerun-if-env-changed=TRANSLATER_HELPER_VERSION");
    println!("cargo:rerun-if-changed=../version.txt");

    let version = std::env::var("TRANSLATER_HELPER_VERSION")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::fs::read_to_string("../version.txt")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "dev".to_string());

    println!("cargo:rustc-env=TRANSLATER_HELPER_VERSION={version}");
}
