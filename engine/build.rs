use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/physics/shaders/particle_update.comp");

    let mut compiler = shaderc::Compiler::new().unwrap();
    let shader_source = PathBuf::from("src/physics/shaders/particle_update.comp");

    let artifact = compiler
        .compile_into_spirv(
            &std::fs::read_to_string(&shader_source).unwrap(),
            shaderc::ShaderKind::Compute,
            "particle_update.comp",
            "main",
            None,
        )
        .unwrap();

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(out_dir.join("particle_update.spv"), artifact.as_binary_u8()).unwrap();
}
