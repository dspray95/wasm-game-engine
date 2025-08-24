use anyhow::*;
use fs_extra::dir::{ copy, CopyOptions };
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=res/*");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let res_dir = manifest_dir.join("res");

    if !res_dir.exists() {
        eprintln!("Warning: res/ directory not found at {:?}", res_dir);
        return Ok(()); // don’t error if missing
    }

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.copy_inside = true;

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    if target_arch == "wasm32" {
        // For wasm builds → copy into ./pkg/res
        let target = manifest_dir.join("pkg").join("res");
        copy(&res_dir, target, &copy_options)?;
    } else {
        // For native builds → copy into OUT_DIR (Cargo build dir)
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        let target = out_dir.join("res");
        copy(&res_dir, target, &copy_options)?;
    }

    Ok(())
}
