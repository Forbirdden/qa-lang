use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("zig_bundle");

    if !dest_path.exists() {
        fs::create_dir_all(&dest_path).unwrap();
    }

    if let Some(url) = get_zig_url(&target) {
        if fs::read_dir(&dest_path).map(|mut i| i.next().is_none()).unwrap_or(true) {
             download_and_extract(&url, &dest_path);
        }
    } 

    println!("cargo:rustc-env=ZIG_BUNDLE_PATH={}", dest_path.display());
    println!("cargo:rerun-if-env-changed=TARGET");
}


const ZIG_VER: &str = "0.15.2";

fn get_zig_url(target: &str) -> Option<String> {
    let (arch, os, ext) = if target.contains("x86_64-unknown-linux") {
        ("x86_64", "linux", "tar.xz")
    } else if target.contains("aarch64-unknown-linux") {
        ("aarch64", "linux", "tar.xz")
    } else if target.contains("arm-unknown-linux") {
        ("arm", "linux", "tar.xz")
    } else if target.contains("riscv64gc-unknown-linux") {
        ("riscv64", "linux", "tar.xz")
    } else if target.contains("powerpc64le-unknown-linux") {
        ("powerpc64le", "linux", "tar.xz")
    } else if target.contains("i686-unknown-linux") {
        ("x86", "linux", "tar.xz")
    } else if target.contains("loongarch64-unknown-linux") {
        ("loongarch64", "linux", "tar.xz")
    } else if target.contains("s390x-unknown-linux") {
        ("s390x", "linux", "tar.xz")
    } else if target.contains("aarch64-unknown-freebsd") {
        ("aarch64", "freebsd", "tar.xz")
    } else if target.contains("arm-unknown-freebsd") {
        ("arm", "freebsd", "tar.xz")
    } else if target.contains("riscv64-unknown-freebsd") {
        ("riscv64", "freebsd", "tar.xz")
    } else if target.contains("powerpc64le-unknown-freebsd") {
        ("powerpc64le", "freebsd", "tar.xz")
    } else if target.contains("powerpc64-unknown-freebsd") {
        ("powerpc64", "freebsd", "tar.xz")
    } else if target.contains("x86_64-unknown-freebsd") {
        ("x86_64", "freebsd", "tar.xz")
    } else if target.contains("aarch64-unknown-netbsd") {
        ("aarch64", "netbsd", "tar.xz")
    } else if target.contains("arm-unknown-netbsd") {
        ("arm", "netbsd", "tar.xz")
    } else if target.contains("x86-unknown-netbsd") {
        ("x86", "netbsd", "tar.xz")
    } else if target.contains("x86_64-unknown-netbsd") {
        ("x86_64", "netbsd", "tar.xz")
    }  else if target.contains("x86_64-pc-windows") {
        ("x86_64", "windows", "zip")
    } else if target.contains("i686-pc-windows") {
        ("x86", "windows", "zip")
    } else if target.contains("x86_64-apple-darwin") {
        ("x86_64", "macos", "tar.xz")
    } else if target.contains("aarch64-apple-darwin") {
        ("aarch64", "macos", "tar.xz")
    } else if target.contains("freebsd") {
        ("x86_64", "freebsd", "tar.xz")
    } else {
        return None;
    };

    Some(format!(
        "https://ziglang.org/download/{v}/zig-{a}-{o}-{v}.{e}",
        v = ZIG_VER, a = arch, o = os, e = ext
    ))
}

fn download_and_extract(url: &str, dest_dir: &Path) {
    let mut response = reqwest::blocking::get(url).expect("Failed to download Zig");
    let mut data = Vec::new();
    response.copy_to(&mut data).expect("Failed to read Zig data");

    if url.ends_with(".zip") {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data)).unwrap();
        archive.extract(dest_dir).unwrap();
    } else {
        let tar_data = xz2::read::XzDecoder::new(std::io::Cursor::new(data));
        let mut archive = tar::Archive::new(tar_data);
        archive.unpack(dest_dir).unwrap(); 
    }
}
