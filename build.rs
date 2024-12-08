fn main() {
    if std::env::var("CARGO_FEATURE_STATIC_LINK").is_ok() {
        if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
            // println!("cargo:rustc-link-lib=static=windows.0.52.0");
        }

        //println!("cargo:rustc-link-lib=static=windows.0.52.0");
        //println!("cargo:rustc-link-lib=static=skparagraph");
        //println!("cargo:rustc-link-lib=static=skshaper");
        //println!("cargo:rustc-link-lib=static=skunicode_core");
        //println!("cargo:rustc-link-lib=static=skunicode_icu");
        //println!("cargo:rustc-link-lib=static=svg");
        //println!("cargo:rustc-link-lib=static=skresources");
    }
}
