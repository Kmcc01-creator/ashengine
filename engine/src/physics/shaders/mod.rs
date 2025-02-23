use ash::vk;
use shaderc;
use std::sync::Arc;

pub struct ShaderModule {
    device: Arc<ash::Device>,
    module: vk::ShaderModule,
}

impl ShaderModule {
    pub fn new(
        device: Arc<ash::Device>,
        spirv_code: &[u32],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(spirv_code)
            .build();

        let module = unsafe { device.create_shader_module(&create_info, None)? };

        Ok(Self { device, module })
    }

    pub fn get_module(&self) -> vk::ShaderModule {
        self.module
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.module, None);
        }
    }
}

use shaderc;
use std::sync::Arc;

pub struct ShaderModule {
    device: Arc<ash::Device>,
    module: vk::ShaderModule,
}

impl ShaderModule {
    pub fn new(
        device: Arc<ash::Device>,
        spirv_code: &[u32],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(spirv_code)
            .build();

        let module = unsafe { device.create_shader_module(&create_info, None)? };

        Ok(Self { device, module })
    }

    pub fn get_module(&self) -> vk::ShaderModule {
        self.module
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.module, None);
        }
    }
}

pub fn compile_shader(
    source: &str,
    shader_kind: shaderc::ShaderKind,
    entry_point: &str,
    options: Option<&shaderc::CompileOptions>,
) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let compiler = shaderc::Compiler::new().ok_or("Failed to create shader compiler")?;

    let binary_result = compiler.compile_into_spirv(
        source,
        shader_kind,
        "shader.comp", // Arbitrary filename for error messages
        entry_point,
        options,
    )?;

    Ok(binary_result.as_binary().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_shader_no_options() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
            #version 450
            void main() {}
        "#;

        let spirv_code = compile_shader(source, shaderc::ShaderKind::Compute, "main", None)?;
        assert!(!spirv_code.is_empty());
        Ok(())
    }

    #[test]
    fn test_compile_shader_with_options() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
            #version 450
            void main() {}
        "#;

        let mut options = shaderc::CompileOptions::new().unwrap();
        options.add_macro_definition("TEST_MACRO", Some("1"));
        options.set_optimization_level(shaderc::OptimizationLevel::Performance);
        options.set_generate_debug_info();

        let spirv_code =
            compile_shader(source, shaderc::ShaderKind::Compute, "main", Some(&options))?;
        assert!(!spirv_code.is_empty());
        Ok(())
    }
    #[test]
    fn test_compile_shader_error() {
        let source = r#"
            #version 450
            void main() {
                invalid_syntax;
            }
        "#;

        let result = compile_shader(source, shaderc::ShaderKind::Compute, "main", None);
        assert!(result.is_err()); // Expecting compilation error
    }
}
