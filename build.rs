use std::path::{Path, PathBuf};

fn main() {
    // Get the output directory.
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir);

    // Read the memory map.
    let mem_map_path = Path::new("memory.x");
    let mem_map_contents = std::fs::read(mem_map_path)
        .unwrap_or_else(|e| panic!("Failed to read `{}`: {:?}", mem_map_path.display(), e));

    // Write the MCU specific memory map to `memory.x` in the output directory
    // and ensure the output directory on the linker search path.
    let mem_out_path = out_dir.join("memory.x");
    std::fs::write(&mem_out_path, &mem_map_contents)
        .unwrap_or_else(|e| panic!("Failed to write `{}`: {:?}", mem_out_path.display(), e));
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Ensure the build script is only re-run when the MCU specific memory map
    // is changed.
    println!("cargo:rerun-if-changed={}", mem_map_path.display());

    // Add the cortex-m-rt linker script.
    println!("cargo:rustc-link-arg=-Tlink.x");
}
