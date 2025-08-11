fn main() {
  println!("cargo:rustc-cfg=dev");
  tauri_build::build()
}
