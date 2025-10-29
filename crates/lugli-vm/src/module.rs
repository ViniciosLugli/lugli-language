use crate::Bytecode;
use hashbrown::{HashMap, HashSet};
use lugli_common::{LugliError, Value};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

/// Represents a compiled module with its exports
#[derive(Debug, Clone)]
pub struct Module {
    pub path: PathBuf,
    pub bytecode: Bytecode,
    pub exports: HashMap<String, Value>,
}

impl Module {
    pub fn new(path: PathBuf, bytecode: Bytecode) -> Self {
        Self {
            path,
            bytecode,
            exports: HashMap::new(),
        }
    }

    pub fn export(&mut self, name: String, value: Value) { self.exports.insert(name, value); }

    pub fn get_export(&self, name: &str) -> Option<&Value> { self.exports.get(name) }
}

/// Caches compiled modules to avoid recompilation
pub struct ModuleCache {
    modules: HashMap<PathBuf, Rc<Module>>,
    loading: HashSet<PathBuf>,
}

impl ModuleCache {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            loading: HashSet::new(),
        }
    }

    pub fn get(&self, path: &Path) -> Option<Rc<Module>> { self.modules.get(path).cloned() }

    pub fn insert(&mut self, path: PathBuf, module: Module) -> Rc<Module> {
        let rc_module = Rc::new(module);
        self.modules.insert(path, rc_module.clone());
        rc_module
    }

    pub fn is_loading(&self, path: &Path) -> bool { self.loading.contains(path) }

    pub fn mark_loading(&mut self, path: PathBuf) { self.loading.insert(path); }

    pub fn unmark_loading(&mut self, path: &Path) { self.loading.remove(path); }

    pub fn clear(&mut self) {
        self.modules.clear();
        self.loading.clear();
    }
}

impl Default for ModuleCache {
    fn default() -> Self { Self::new() }
}

/// Resolves module import paths to filesystem paths
pub struct ModuleResolver {
    search_paths: Vec<PathBuf>,
    stdlib_path: Option<PathBuf>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self {
            search_paths: vec![PathBuf::from(".")],
            stdlib_path: Self::find_stdlib_path(),
        }
    }

    pub fn with_search_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.search_paths = paths;
        self
    }

    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Resolve an import path to an absolute filesystem path
    pub fn resolve(&self, import_path: &str, from_file: Option<&Path>) -> Result<PathBuf, LugliError> {
        // Security: Prevent path traversal attacks
        if import_path.contains("..") || import_path.starts_with('/') || import_path.starts_with('\\') {
            return Err(LugliError::runtime(format!("Invalid module path '{}': absolute paths and path traversal not allowed", import_path)));
        }

        // Normalize import path (convert dots to slashes if needed)
        let import_path = import_path.replace('.', "/");

        // Check if this is a stdlib import
        if import_path.starts_with("stdlib/") || import_path == "stdlib" {
            if let Some(ref stdlib) = self.stdlib_path {
                let module_path = stdlib.join(&import_path[7..]).with_extension("lg");
                if module_path.exists() {
                    return Ok(module_path);
                }
            }
            return Err(LugliError::runtime(format!("Stdlib module '{}' not found", import_path)));
        }

        // Try relative to current file first
        if let Some(current_file) = from_file
            && let Some(parent) = current_file.parent() {
                let relative_path = parent.join(&import_path).with_extension("lg");
                if relative_path.exists() {
                    return relative_path
                        .canonicalize()
                        .map_err(|e| LugliError::runtime(format!("Failed to resolve module path '{}': {}", relative_path.display(), e)));
                }
            }

        // Try search paths
        for search_path in &self.search_paths {
            let candidate = search_path.join(&import_path).with_extension("lg");
            if candidate.exists() {
                return candidate
                    .canonicalize()
                    .map_err(|e| LugliError::runtime(format!("Failed to resolve module path '{}': {}", candidate.display(), e)));
            }
        }

        // Try with explicit .lg extension (if user provided it)
        if import_path.ends_with(".lg") {
            if let Some(current_file) = from_file
                && let Some(parent) = current_file.parent() {
                    let relative_path = parent.join(&import_path);
                    if relative_path.exists() {
                        return relative_path
                            .canonicalize()
                            .map_err(|e| LugliError::runtime(format!("Failed to resolve module path '{}': {}", relative_path.display(), e)));
                    }
                }

            for search_path in &self.search_paths {
                let candidate = search_path.join(&import_path);
                if candidate.exists() {
                    return candidate
                        .canonicalize()
                        .map_err(|e| LugliError::runtime(format!("Failed to resolve module path '{}': {}", candidate.display(), e)));
                }
            }
        }

        Err(LugliError::runtime(format!("Module '{}' not found in search paths: {:?}", import_path, self.search_paths)))
    }

    fn find_stdlib_path() -> Option<PathBuf> {
        // Try environment variable first
        if let Ok(path) = std::env::var("LUGLI_STDLIB_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        // Try relative to executable
        if let Ok(exe_path) = std::env::current_exe()
            && let Some(exe_dir) = exe_path.parent() {
                let stdlib = exe_dir.join("stdlib");
                if stdlib.exists() {
                    return Some(stdlib);
                }
            }

        // Try current directory
        let local_stdlib = PathBuf::from("./stdlib");
        if local_stdlib.exists() {
            return Some(local_stdlib);
        }

        None
    }
}

impl Default for ModuleResolver {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let bytecode = Bytecode::new();
        let module = Module::new(PathBuf::from("test.lg"), bytecode);
        assert_eq!(module.path, PathBuf::from("test.lg"));
        assert!(module.exports.is_empty());
    }

    #[test]
    fn test_module_export() {
        let bytecode = Bytecode::new();
        let mut module = Module::new(PathBuf::from("test.lg"), bytecode);

        module.export("foo".to_string(), Value::Number(42.0));
        assert_eq!(module.get_export("foo"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_module_cache() {
        let mut cache = ModuleCache::new();
        let bytecode = Bytecode::new();
        let module = Module::new(PathBuf::from("test.lg"), bytecode);

        let path = PathBuf::from("test.lg");
        let rc_module = cache.insert(path.clone(), module);

        assert!(cache.get(&path).is_some());
        assert_eq!(Rc::strong_count(&rc_module), 2); // cache + our reference
    }

    #[test]
    fn test_circular_import_detection() {
        let mut cache = ModuleCache::new();
        let path = PathBuf::from("circular.lg");

        assert!(!cache.is_loading(&path));
        cache.mark_loading(path.clone());
        assert!(cache.is_loading(&path));

        cache.unmark_loading(&path);
        assert!(!cache.is_loading(&path));
    }

    #[test]
    fn test_module_resolver_creation() {
        let resolver = ModuleResolver::new();
        assert!(!resolver.search_paths.is_empty());
    }

    #[test]
    fn test_add_search_path() {
        let mut resolver = ModuleResolver::new();
        let new_path = PathBuf::from("/custom/path");

        resolver.add_search_path(new_path.clone());
        assert!(resolver.search_paths.contains(&new_path));

        // Should not add duplicates
        let len_before = resolver.search_paths.len();
        resolver.add_search_path(new_path);
        assert_eq!(resolver.search_paths.len(), len_before);
    }
}
