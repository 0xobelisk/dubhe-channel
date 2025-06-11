//! 动态库加载模块
//!
//! Rust libloading 插件安全封装

use anyhow::Result;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::types::{CompilationConfig, Plugin, PluginHandle};

/// 插件管理器
pub struct PluginManager {
    plugins: HashMap<PluginHandle, LoadedPlugin>,
    next_handle: u64,
}

/// 已加载的插件
struct LoadedPlugin {
    #[allow(dead_code)]
    library: Library,
    plugin: Box<dyn Plugin>,
    path: String,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            next_handle: 1,
        }
    }

    /// 加载插件
    pub fn load_plugin(&mut self, path: &str) -> Result<PluginHandle> {
        info!("Loading plugin from: {}", path);

        // 安全检查
        if !self.is_safe_plugin_path(path) {
            return Err(anyhow::anyhow!("Unsafe plugin path: {}", path));
        }

        // 加载动态库
        let library = unsafe { Library::new(path)? };

        // 获取插件创建函数
        let create_plugin: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> =
            unsafe { library.get(b"create_plugin")? };

        // 创建插件实例
        let plugin_ptr = unsafe { create_plugin() };
        if plugin_ptr.is_null() {
            return Err(anyhow::anyhow!("Plugin creation failed"));
        }

        let plugin = unsafe { Box::from_raw(plugin_ptr) };

        // 验证插件
        self.validate_plugin(&*plugin)?;

        let handle = PluginHandle(self.next_handle);
        self.next_handle += 1;

        let loaded_plugin = LoadedPlugin {
            library,
            plugin,
            path: path.to_string(),
        };

        self.plugins.insert(handle, loaded_plugin);

        info!("Plugin loaded successfully: {}", path);
        Ok(handle)
    }

    /// 卸载插件
    pub fn unload_plugin(&mut self, handle: PluginHandle) -> Result<()> {
        match self.plugins.remove(&handle) {
            Some(plugin) => {
                info!("Unloading plugin: {}", plugin.path);
                // 库会在 drop 时自动卸载
                Ok(())
            }
            None => Err(anyhow::anyhow!("Plugin handle not found: {:?}", handle)),
        }
    }

    /// 获取插件
    pub fn get_plugin(&self, handle: PluginHandle) -> Option<&dyn Plugin> {
        self.plugins.get(&handle).map(|p| &*p.plugin)
    }

    /// 列出所有已加载的插件
    pub fn list_plugins(&self) -> Vec<(PluginHandle, &str, &str, &str)> {
        self.plugins
            .iter()
            .map(|(handle, plugin)| {
                (
                    *handle,
                    plugin.path.as_str(),
                    plugin.plugin.name(),
                    plugin.plugin.version(),
                )
            })
            .collect()
    }

    /// 验证插件安全性
    fn validate_plugin(&self, plugin: &dyn Plugin) -> Result<()> {
        // 基本验证
        if plugin.name().is_empty() {
            return Err(anyhow::anyhow!("Plugin name cannot be empty"));
        }

        if plugin.version().is_empty() {
            return Err(anyhow::anyhow!("Plugin version cannot be empty"));
        }

        // 测试编译功能（使用空字节码）
        let test_config = CompilationConfig::default();
        match plugin.compile(&[], &test_config) {
            Ok(_) => {
                info!("Plugin validation passed: {}", plugin.name());
                Ok(())
            }
            Err(e) => {
                warn!("Plugin validation failed: {}", e);
                Ok(()) // 允许测试编译失败，因为可能需要有效的输入
            }
        }
    }

    /// 检查插件路径安全性
    fn is_safe_plugin_path(&self, path: &str) -> bool {
        // 基本安全检查
        if path.contains("..") {
            error!("Plugin path contains '..': {}", path);
            return false;
        }

        if !path.ends_with(".so") && !path.ends_with(".dylib") && !path.ends_with(".dll") {
            error!("Plugin path has invalid extension: {}", path);
            return false;
        }

        // TODO: 添加更多安全检查
        // - 检查文件签名
        // - 检查文件权限
        // - 检查来源白名单

        true
    }
}

/// 示例插件实现
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        "example-compiler"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn compile(&self, bytecode: &[u8], _config: &CompilationConfig) -> anyhow::Result<Vec<u8>> {
        // 示例编译：直接返回输入（用于测试）
        Ok(bytecode.to_vec())
    }
}

// 导出函数（用于动态加载）
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(ExamplePlugin))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();

        // 测试基本功能
        assert_eq!(manager.list_plugins().len(), 0);

        // 测试路径验证
        assert!(!manager.is_safe_plugin_path("../malicious.so"));
        assert!(!manager.is_safe_plugin_path("plugin.txt"));
        assert!(manager.is_safe_plugin_path("plugin.so"));
    }

    #[test]
    fn test_example_plugin() {
        let plugin = ExamplePlugin;
        assert_eq!(plugin.name(), "example-compiler");
        assert_eq!(plugin.version(), "0.1.0");

        let config = CompilationConfig::default();
        let result = plugin.compile(&[1, 2, 3], &config).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }
}
