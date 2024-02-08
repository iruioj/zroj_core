use std::{env, path::PathBuf};

fn main() {
    #[cfg(unix)]
    {
        // Tell Cargo that if the given file changes, to rerun this build script.
        println!("cargo:rerun-if-changed=src/unix/utils.c");
        println!("cargo:rerun-if-changed=src/unix/sigutils.h");
        println!("cargo:rerun-if-changed=src/unix/share.h");
        println!("cargo:rerun-if-changed=src/unix/sio.h");

        cc::Build::new()
            .file("src/unix/utils.c")
            .compile("sandboxunixutils");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

        bindgen::Builder::default()
            .header("src/unix/sigutils.h")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            // Finish the builder and generate the bindings.
            .generate()
            .expect("Unable to generate sigutils.h bindings")
            .write_to_file(out_path.join("sigutilscc.rs"))
            .expect("Couldn't write bindings!");

        bindgen::Builder::default()
            .header("src/unix/share.h")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            // Finish the builder and generate the bindings.
            .generate()
            .expect("Unable to generate share.h bindings")
            .write_to_file(out_path.join("sharecc.rs"))
            .expect("Couldn't write bindings!");
    }
}
