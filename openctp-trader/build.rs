const SO_FILENAME: &str = "thosttraderapi_se_tts.so";

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let current_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let so_symlink_string = format!("{}/lib{}", out_dir, SO_FILENAME);
    let so_symlink = std::path::Path::new(&so_symlink_string);
    if so_symlink.exists() {
        std::fs::remove_file(so_symlink).expect("symlink exists, but failed to remove it");
    }
    std::os::unix::fs::symlink(
        &format!("{}/../api-openctp/lib/{}", current_dir, SO_FILENAME),
        so_symlink,
    )
    .expect("failed to create new symlink");
    println!("cargo:rustc-link-search=native={}", out_dir);
    let target_so = format!("{}/{}", out_dir, SO_FILENAME);
    println!("{}", target_so);
    std::fs::copy(
        &format!("{}/../api-openctp/lib/{}", current_dir, SO_FILENAME),
        &target_so,
    )
    .expect("failed to copy so to outdir");
    println!("cargo:resource={}", target_so);
    println!("cargo:rerun-if-changed={}", out_dir);
    println!("cargo:rerun-if-changed={}", target_so);
}
