use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=MNN_LIB_DIR");
    println!("cargo:rerun-if-env-changed=MNN_INCLUDE_DIR");
    println!("cargo:rustc-check-cfg=cfg(mnn_linked)");
    println!("cargo:rustc-check-cfg=cfg(mnn_llm_linked)");

    if cfg!(feature = "mnn") {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mnn_wrapper_dir = PathBuf::from(&manifest_dir).join("mnn-wrapper");

        println!("cargo:rerun-if-changed=mnn-wrapper/mnn_c_api.h");
        println!("cargo:rerun-if-changed=mnn-wrapper/mnn_c_api.cpp");

        let mut found_mnn = false;
        let mut found_llm = false;
        let mut found_wrapper = false;

        let mnn_source_dir = PathBuf::from(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.join("MNN"))
            .unwrap_or_default();

        let mnn_llm_build_dir = mnn_source_dir.join("build_llm").join("Release");
        let wrapper_build_dir = mnn_wrapper_dir.join("build").join("Release");

        if let Ok(mnn_lib_dir) = env::var("MNN_LIB_DIR") {
            let lib_path = PathBuf::from(&mnn_lib_dir);
            if lib_path.exists() {
                println!("cargo:rustc-link-search=native={}", mnn_lib_dir);
                println!("cargo:rustc-link-lib=dylib=MNN");
                found_mnn = true;
            }
        }

        if !found_mnn {
            let search_paths = vec![
                mnn_llm_build_dir.clone(),
                PathBuf::from(&manifest_dir)
                    .parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.parent())
                    .map(|p| p.join("MNN").join("build").join("Release"))
                    .unwrap_or_default(),
                PathBuf::from(&manifest_dir)
                    .parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.parent())
                    .map(|p| p.join("MNN").join("build"))
                    .unwrap_or_default(),
                PathBuf::from(
                    "D:\\mnn_lib\\mnn_3.4.1_windows_x64_cpu_opencl\\lib\\x64\\Release\\Dynamic\\MD",
                ),
                PathBuf::from(
                    "D:\\mnn_lib\\mnn_3.4.1_windows_x64_cpu_opencl\\lib\\x64\\Release\\Dynamic\\MT",
                ),
                wrapper_build_dir.clone(),
                mnn_wrapper_dir.join("build"),
                PathBuf::from(&manifest_dir).join("lib"),
            ];

            for path in search_paths {
                if path.exists() {
                    let has_mnn = path.join("MNN.dll").exists()
                        || path.join("MNN.lib").exists()
                        || path.join("libMNN.so").exists()
                        || path.join("libMNN.dylib").exists();

                    if has_mnn {
                        println!("cargo:rustc-link-search=native={}", path.display());
                        println!("cargo:rustc-link-lib=dylib=MNN");
                        found_mnn = true;

                        let llm_header = mnn_source_dir
                            .join("transformers")
                            .join("llm")
                            .join("engine")
                            .join("include")
                            .join("llm")
                            .join("llm.hpp");
                        if llm_header.exists() {
                            found_llm = true;
                        }
                        break;
                    }
                }
            }
        }

        if found_llm && cfg!(feature = "mnn-llm") {
            if wrapper_build_dir.exists() {
                println!(
                    "cargo:rustc-link-search=native={}",
                    wrapper_build_dir.display()
                );

                let has_wrapper = wrapper_build_dir.join("mnn_llm_wrapper.dll").exists()
                    || wrapper_build_dir.join("mnn_llm_wrapper.lib").exists()
                    || wrapper_build_dir.join("libmnn_llm_wrapper.so").exists()
                    || wrapper_build_dir.join("libmnn_llm_wrapper.dylib").exists();

                if has_wrapper {
                    println!("cargo:rustc-link-lib=dylib=mnn_llm_wrapper");
                    found_wrapper = true;
                }
            }

            if !found_wrapper {
                let wrapper_search_paths = vec![
                    mnn_wrapper_dir.join("build"),
                    PathBuf::from(&manifest_dir).join("lib"),
                ];

                for path in wrapper_search_paths {
                    if path.exists() {
                        let has_wrapper = path.join("mnn_llm_wrapper.dll").exists()
                            || path.join("mnn_llm_wrapper.lib").exists();

                        if has_wrapper {
                            println!("cargo:rustc-link-search=native={}", path.display());
                            println!("cargo:rustc-link-lib=dylib=mnn_llm_wrapper");
                            found_wrapper = true;
                            break;
                        }
                    }
                }
            }
        }

        if found_mnn {
            println!("cargo:rustc-cfg=mnn_linked");
        }

        if found_llm && cfg!(feature = "mnn-llm") && found_wrapper {
            println!("cargo:rustc-cfg=mnn_llm_linked");
        }

        if !found_mnn {
            println!("cargo:warning=========================================");
            println!("cargo:warning=MNN library not found!");
            println!("cargo:warning=========================================");
            println!("cargo:warning=To build with MNN support, you need to:");
            println!("cargo:warning=1. Install Visual Studio 2017/2019/2022 or MinGW-w64");
            println!("cargo:warning=2. Run the build script:");
            println!("cargo:warning=   cd crates/mobile-claw/mnn-wrapper");
            println!("cargo:warning=   ./build_windows.bat");
            println!("cargo:warning=");
            println!("cargo:warning=Or set environment variables:");
            println!("cargo:warning=   MNN_LIB_DIR=/path/to/mnn/lib");
            println!("cargo:warning=   MNN_INCLUDE_DIR=/path/to/mnn/include");
            println!("cargo:warning=");
            println!("cargo:warning=Pre-built MNN libraries can be downloaded from:");
            println!("cargo:warning=https://github.com/alibaba/MNN/releases");
            println!("cargo:warning=========================================");
        }

        if cfg!(feature = "mnn-llm") && found_llm && !found_wrapper {
            println!("cargo:warning=========================================");
            println!("cargo:warning=MNN LLM wrapper library not found!");
            println!("cargo:warning=========================================");
            println!("cargo:warning=To build with MNN LLM support, you need to:");
            println!("cargo:warning=1. Build MNN with LLM support first:");
            println!("cargo:warning=   cd MNN");
            println!("cargo:warning=   ./build_llm.bat");
            println!("cargo:warning=2. Build the LLM wrapper:");
            println!("cargo:warning=   cd crates/mobile-claw/mnn-wrapper");
            println!("cargo:warning=   mkdir build && cd build");
            println!("cargo:warning=   cmake .. -DMNN_DIR=../../../MNN -DMNN_BUILD_DIR=../../../MNN/build_llm");
            println!("cargo:warning=   cmake --build . --config Release");
            println!("cargo:warning=========================================");
        }

        if cfg!(target_os = "windows") {
            println!("cargo:rustc-link-lib=dylib=user32");
            println!("cargo:rustc-link-lib=dylib=ole32");
            println!("cargo:rustc-link-lib=dylib=shell32");
        }

        if cfg!(target_os = "linux") {
            println!("cargo:rustc-link-lib=dylib=pthread");
            println!("cargo:rustc-link-lib=dylib=dl");
        }

        if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=Metal");
        }
    }
}
