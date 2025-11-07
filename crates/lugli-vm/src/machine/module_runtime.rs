use crate::{
    Bytecode,
    module::{ModuleCache, ModuleResolver},
};
use hashbrown::HashMap;
use lugli_common::{LugliError, Value};
use std::{path::{Path, PathBuf}, rc::Rc};

/// Manages module loading, caching, and bytecode registry
pub struct ModuleRuntime {
    /// Cache of loaded modules (path → module)
    module_cache: ModuleCache,

    /// Resolves module paths
    module_resolver: ModuleResolver,

    /// Registry of bytecode by ID
    bytecode_registry: HashMap<usize, Rc<Bytecode>>,

    /// Module-specific globals (bytecode_id → globals)
    module_globals: HashMap<usize, Rc<HashMap<String, Value>>>,

    /// Next bytecode ID to assign
    next_bytecode_id: usize,

    /// Current file being executed
    current_file: Option<PathBuf>,
}

impl ModuleRuntime {
    pub fn new() -> Self {
        Self {
            module_cache: ModuleCache::new(),
            module_resolver: ModuleResolver::new(),
            bytecode_registry: HashMap::with_capacity(16),
            module_globals: HashMap::with_capacity(16),
            next_bytecode_id: 1, // ID 0 reserved for main script
            current_file: None,
        }
    }

    // Bytecode registry operations
    pub fn register_bytecode(&mut self, bytecode: Bytecode) -> usize {
        let id = self.next_bytecode_id;
        self.next_bytecode_id += 1;
        self.bytecode_registry.insert(id, Rc::new(bytecode));
        id
    }

    pub fn register_main_bytecode(&mut self, bytecode: Bytecode) {
        self.bytecode_registry.insert(0, Rc::new(bytecode));
    }

    pub fn get_bytecode(&self, id: usize) -> Option<&Rc<Bytecode>> {
        self.bytecode_registry.get(&id)
    }

    pub fn clear_bytecode_registry(&mut self) {
        self.bytecode_registry.clear();
        self.next_bytecode_id = 1;
    }

    pub fn retain_bytecodes<F>(&mut self, f: F)
    where
        F: FnMut(&usize, &mut Rc<Bytecode>) -> bool,
    {
        self.bytecode_registry.retain(f);
    }

    // Module cache operations
    pub fn module_cache(&self) -> &ModuleCache {
        &self.module_cache
    }

    pub fn module_cache_mut(&mut self) -> &mut ModuleCache {
        &mut self.module_cache
    }

    pub fn clear_module_cache(&mut self) {
        self.module_cache.clear();
    }

    // Module resolver operations
    pub fn module_resolver(&self) -> &ModuleResolver {
        &self.module_resolver
    }

    pub fn module_resolver_mut(&mut self) -> &mut ModuleResolver {
        &mut self.module_resolver
    }

    // Module globals operations
    pub fn save_module_globals(&mut self, bytecode_id: usize, globals: HashMap<String, Value>) {
        self.module_globals.insert(bytecode_id, Rc::new(globals));
    }

    pub fn get_module_globals(&self, bytecode_id: usize) -> Option<&Rc<HashMap<String, Value>>> {
        self.module_globals.get(&bytecode_id)
    }

    pub fn clear_module_globals(&mut self) {
        self.module_globals.clear();
    }

    // Current file operations
    pub fn set_current_file(&mut self, path: Option<PathBuf>) {
        self.current_file = path;
    }

    pub fn current_file(&self) -> Option<&Path> {
        self.current_file.as_deref()
    }

    // Full reset
    pub fn reset(&mut self) {
        self.module_cache.clear();
        self.bytecode_registry.clear();
        self.module_globals.clear();
        self.next_bytecode_id = 1;
        self.current_file = None;
    }

    // Module loading helper (resolves path)
    pub fn resolve_module_path(
        &self,
        module_path: &str,
        base_path: Option<&Path>,
    ) -> Result<PathBuf, LugliError> {
        self.module_resolver.resolve(module_path, base_path)
    }

    // Check if module is already cached
    pub fn is_module_cached(&self, path: &Path) -> bool {
        self.module_cache.get(path).is_some()
    }

    // Check if module is currently being loaded (circular import detection)
    pub fn is_module_loading(&self, path: &Path) -> bool {
        self.module_cache.is_loading(path)
    }

    pub fn mark_module_loading(&mut self, path: PathBuf) {
        self.module_cache.mark_loading(path);
    }

    pub fn unmark_module_loading(&mut self, path: &Path) {
        self.module_cache.unmark_loading(path);
    }
}

impl Default for ModuleRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_runtime_creation() {
        let runtime = ModuleRuntime::new();
        assert_eq!(runtime.next_bytecode_id, 1);
        assert!(runtime.current_file.is_none());
    }

    #[test]
    fn test_bytecode_registration() {
        let mut runtime = ModuleRuntime::new();

        let bytecode1 = Bytecode::new();
        let bytecode2 = Bytecode::new();

        let id1 = runtime.register_bytecode(bytecode1);
        let id2 = runtime.register_bytecode(bytecode2);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert!(runtime.get_bytecode(id1).is_some());
        assert!(runtime.get_bytecode(id2).is_some());
        assert!(runtime.get_bytecode(99).is_none());
    }

    #[test]
    fn test_module_globals() {
        let mut runtime = ModuleRuntime::new();

        let mut globals = HashMap::new();
        globals.insert("x".to_string(), Value::Number(42.0));

        runtime.save_module_globals(1, globals);

        let retrieved = runtime.get_module_globals(1).unwrap();
        assert_eq!(retrieved.get("x"), Some(&Value::Number(42.0)));
        assert!(runtime.get_module_globals(2).is_none());
    }

    #[test]
    fn test_current_file() {
        let mut runtime = ModuleRuntime::new();
        assert!(runtime.current_file().is_none());

        let path = PathBuf::from("/test/module.lg");
        runtime.set_current_file(Some(path.clone()));
        assert_eq!(runtime.current_file(), Some(path.as_path()));

        runtime.set_current_file(None);
        assert!(runtime.current_file().is_none());
    }

    #[test]
    fn test_module_runtime_reset() {
        let mut runtime = ModuleRuntime::new();

        // Add some state
        let bytecode = Bytecode::new();
        let id = runtime.register_bytecode(bytecode);
        let mut globals = HashMap::new();
        globals.insert("x".to_string(), Value::Number(10.0));
        runtime.save_module_globals(id, globals);
        runtime.set_current_file(Some(PathBuf::from("/test.lg")));

        // Reset
        runtime.reset();

        // Verify everything cleared
        assert_eq!(runtime.next_bytecode_id, 1);
        assert!(runtime.current_file().is_none());
        assert!(runtime.get_bytecode(id).is_none());
        assert!(runtime.get_module_globals(id).is_none());
    }

    #[test]
    fn test_module_loading_tracking() {
        let mut runtime = ModuleRuntime::new();
        let path = PathBuf::from("/test/module.lg");

        assert!(!runtime.is_module_loading(&path));

        runtime.mark_module_loading(path.clone());
        assert!(runtime.is_module_loading(&path));

        runtime.unmark_module_loading(&path);
        assert!(!runtime.is_module_loading(&path));
    }
}
