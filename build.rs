
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use bindgen::CargoCallbacks;

fn main ()
{
    let base_path = PathBuf::from (env::current_dir ().unwrap ()).join ("abPOA")
        .canonicalize ()
        .expect ("cannot canonicalize base path");

    // This is the directory where the `c` library is located.
    let libdir_path = base_path.clone ().join ("lib");
    let srcdir_path = base_path.clone ().join ("src");

    fs::create_dir_all (libdir_path.clone ())
        .expect ("failed to create libdir");

    // Tell Cargo that if the given file changes, to rerun this build script.
    println! ("cargo:rerun-if-changed={}", srcdir_path.join ("abpoa.c").to_str ().unwrap ());
    // Tell Cargo that if the lib does not exist, to run this build script.
    println! ("cargo:rerun-if-changed={}", libdir_path.join ("libabpoa.a").to_str ().unwrap ());

    let sm_output = process::Command::new ("git")
        .args([
            "submodule",
            "update",
            "--init",
            "--depth",
            "1",
            "--recommend-shallow",
        ])
        .output ()
        .expect ("Failed to fetch git submodules!");

    println! ("cargo:warning={}", format! ("git submodule stdout: {}", String::from_utf8 (sm_output.stdout).unwrap ()));
    println! ("cargo:warning={}", format! ("git submodule stderr: {}", String::from_utf8 (sm_output.stderr).unwrap ()));
    println! ("cargo:warning={}", format! ("current_dir: '{}' out_dir: '{}'", env::current_dir ().unwrap ().display (), env::var ("OUT_DIR").unwrap ()));


    let pfile = fs::File::open ("src/main/patch/abPOA.patch")
        .expect ("Failed to open patch file");

    process::Command::new ("patch")
        .arg ("-p1")
        .current_dir (base_path.clone ())
        .stdin (pfile)
        .spawn ()
        .expect ("failed to patch abPOA");

    process::Command::new ("make")
        .arg ("clean")
        .current_dir (base_path.clone ())
        .spawn ()
        .expect ("failed to clean abPOA");

    if let Ok ("debug") = env::var ("PROFILE").as_deref ()
    {
        println! ("cargo:warning={}", "built debug version of libabpoa");
        process::Command::new ("make")
            .arg ("libabpoa")
            .env ("gdb", "1")
            .env ("debug", "1")
            .current_dir (base_path.clone ())
            .spawn ()
            .expect ("failed to build abPOA debug");
    }
    else
    {
        println! ("cargo:warning={}", "built release version of libabpoa");
        process::Command::new ("make")
            .arg ("libabpoa")
            .current_dir (base_path.clone ())
            .spawn ()
            .expect ("failed to build abPOA release");
    }

    // Tell cargo to look for shared libraries in the specified directory
    println! ("cargo:rustc-link-search={}", libdir_path.to_str ().unwrap ());

    // Tell cargo to tell rustc to link our `hello` library. Cargo will
    // automatically know it must look for a `libhello.a` file.
    println! ("cargo:rustc-link-lib=abpoa");
    println! ("cargo:rustc-link-lib=z");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header ("src/main/c/appoa_wrapper.h")
        // Tell clang where the library is
        .clang_arg (format! ("-I{}", srcdir_path.display ()))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks (Box::new(CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate ()
        // Unwrap the Result and panic on failure.
        .expect ("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from (env::var ("OUT_DIR").unwrap ()).join("bindings.rs");
    bindings
        .write_to_file (out_path)
        .expect ("Couldn't write bindings!");
}

