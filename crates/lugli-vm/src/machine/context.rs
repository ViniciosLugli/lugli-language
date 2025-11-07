use hashbrown::HashMap;
use lugli_common::{LugliError, Value};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: Rc<str>,
    pub return_ip: usize,
    pub stack_base: usize,
    pub closure_upvalues: Option<Vec<Rc<RefCell<Value>>>>,
}

impl CallFrame {
    pub fn new(function_name: impl Into<Rc<str>>, return_ip: usize, stack_base: usize, upvalues: Option<Vec<Rc<RefCell<Value>>>>) -> Self {
        Self {
            function_name: function_name.into(),
            return_ip,
            stack_base,
            closure_upvalues: upvalues,
        }
    }
}

/// Execution context holds runtime state for VM execution
pub struct ExecutionContext {
    /// The value stack
    stack: Vec<Value>,

    /// Global variables with copy-on-write semantics
    /// Wrapped in Rc<RefCell<>> to enable O(1) snapshots for module isolation
    globals: Rc<RefCell<HashMap<String, Value>>>,

    /// Call stack frames
    call_stack: Vec<CallFrame>,

    /// Open upvalues for closures (stack_index -> upvalue)
    open_upvalues: HashMap<usize, Rc<RefCell<Value>>>,

    /// Current instruction pointer
    ip: usize,
}

impl ExecutionContext {
    pub fn new() -> Self {
        let mut call_stack = Vec::with_capacity(64);
        call_stack.push(CallFrame::new("<script>".to_string(), 0, 0, None));

        Self {
            stack: Vec::with_capacity(256),
            globals: Rc::new(RefCell::new(HashMap::new())),
            call_stack,
            open_upvalues: HashMap::with_capacity(32),
            ip: 0,
        }
    }

    pub fn with_globals(globals: HashMap<String, Value>) -> Self {
        let mut call_stack = Vec::with_capacity(64);
        call_stack.push(CallFrame::new("<script>".to_string(), 0, 0, None));

        Self {
            stack: Vec::with_capacity(256),
            globals: Rc::new(RefCell::new(globals)),
            call_stack,
            open_upvalues: HashMap::with_capacity(32),
            ip: 0,
        }
    }

    // Stack operations
    pub fn push(&mut self, value: Value) { self.stack.push(value); }

    pub fn pop(&mut self) -> Option<Value> { self.stack.pop() }

    pub fn peek(&self, distance: usize) -> Option<&Value> {
        if distance >= self.stack.len() {
            return None;
        }
        self.stack.get(self.stack.len() - 1 - distance)
    }

    pub fn peek_mut(&mut self, distance: usize) -> Option<&mut Value> {
        if distance >= self.stack.len() {
            return None;
        }
        let index = self.stack.len() - 1 - distance;
        self.stack.get_mut(index)
    }

    pub fn stack_len(&self) -> usize { self.stack.len() }

    pub fn truncate_stack(&mut self, len: usize) { self.stack.truncate(len); }

    pub fn clear_stack(&mut self) { self.stack.clear(); }

    pub fn stack(&self) -> &[Value] { &self.stack }

    pub fn stack_mut(&mut self) -> &mut Vec<Value> { &mut self.stack }

    // Global variable operations

    /// Define a new global variable (copy-on-write)
    pub fn define_global(&mut self, name: String, value: Value) {
        // If Rc has multiple owners, make a copy before mutating (COW)
        if Rc::strong_count(&self.globals) > 1 {
            let new_globals = self.globals.borrow().clone();
            self.globals = Rc::new(RefCell::new(new_globals));
        }

        self.globals.borrow_mut().insert(name, value);
    }

    /// Get a global variable (clones the value)
    pub fn get_global(&self, name: &str) -> Option<Value> { self.globals.borrow().get(name).cloned() }

    /// Set an existing global variable (copy-on-write)
    pub fn set_global(&mut self, name: &str, value: Value) -> Result<(), LugliError> {
        if !self.globals.borrow().contains_key(name) {
            return Err(LugliError::runtime(format!("Undefined variable '{}'", name)));
        }

        // If Rc has multiple owners, make a copy before mutating (COW)
        if Rc::strong_count(&self.globals) > 1 {
            let new_globals = self.globals.borrow().clone();
            self.globals = Rc::new(RefCell::new(new_globals));
        }

        self.globals.borrow_mut().insert(name.to_string(), value);
        Ok(())
    }

    /// Get immutable reference to globals HashMap (requires manual borrow)
    pub fn globals(&self) -> Rc<RefCell<HashMap<String, Value>>> { Rc::clone(&self.globals) }

    /// Get mutable access to globals (triggers COW if shared)
    pub fn globals_mut(&mut self) -> Rc<RefCell<HashMap<String, Value>>> {
        // If Rc has multiple owners, make a copy before mutating (COW)
        if Rc::strong_count(&self.globals) > 1 {
            let new_globals = self.globals.borrow().clone();
            self.globals = Rc::new(RefCell::new(new_globals));
        }

        Rc::clone(&self.globals)
    }

    /// Clear all global variables
    pub fn clear_globals(&mut self) {
        // If Rc has multiple owners, just replace with new empty map
        if Rc::strong_count(&self.globals) > 1 {
            self.globals = Rc::new(RefCell::new(HashMap::new()));
        } else {
            self.globals.borrow_mut().clear();
        }
    }

    /// Replace globals with a new HashMap
    pub fn set_globals(&mut self, globals: HashMap<String, Value>) { self.globals = Rc::new(RefCell::new(globals)); }

    /// Snapshot globals for module isolation (O(1) operation)
    pub fn snapshot_globals(&self) -> Rc<RefCell<HashMap<String, Value>>> { Rc::clone(&self.globals) }

    /// Restore globals from a snapshot (O(1) operation)
    pub fn restore_globals(&mut self, snapshot: Rc<RefCell<HashMap<String, Value>>>) { self.globals = snapshot; }

    // Call stack operations
    pub fn push_frame(&mut self, frame: CallFrame) { self.call_stack.push(frame); }

    pub fn pop_frame(&mut self) -> Option<CallFrame> { self.call_stack.pop() }

    pub fn current_frame(&self) -> Option<&CallFrame> { self.call_stack.last() }

    pub fn current_frame_mut(&mut self) -> Option<&mut CallFrame> { self.call_stack.last_mut() }

    pub fn call_depth(&self) -> usize { self.call_stack.len() }

    pub fn call_stack(&self) -> &[CallFrame] { &self.call_stack }

    pub fn clear_call_stack(&mut self) {
        self.call_stack.clear();
        self.call_stack.push(CallFrame::new("<script>".to_string(), 0, 0, None));
    }

    // Upvalue operations
    pub fn capture_upvalue(&mut self, stack_index: usize) -> Result<Rc<RefCell<Value>>, LugliError> {
        if let Some(existing) = self.open_upvalues.get(&stack_index) {
            return Ok(Rc::clone(existing));
        }

        let value = self.stack.get(stack_index).ok_or_else(|| LugliError::runtime("Stack index out of bounds"))?.clone();
        let upvalue = Rc::new(RefCell::new(value));
        self.open_upvalues.insert(stack_index, Rc::clone(&upvalue));
        Ok(upvalue)
    }

    pub fn close_upvalues(&mut self, from_index: usize) { self.open_upvalues.retain(|&idx, _| idx < from_index); }

    pub fn clear_upvalues(&mut self) { self.open_upvalues.clear(); }

    // Instruction pointer operations
    pub fn advance_ip(&mut self, offset: usize) { self.ip += offset; }

    pub fn jump_to(&mut self, target: usize) { self.ip = target; }

    pub fn current_ip(&self) -> usize { self.ip }

    pub fn reset_ip(&mut self) { self.ip = 0; }

    // Full reset operations
    pub fn reset(&mut self) {
        self.stack.clear();
        self.clear_globals();
        self.ip = 0;
        self.call_stack.clear();
        self.call_stack.push(CallFrame::new("<script>".to_string(), 0, 0, None));
        self.open_upvalues.clear();
    }

    pub fn reset_for_repl(&mut self) {
        self.stack.clear();
        self.ip = 0;
        self.call_stack.clear();
        self.call_stack.push(CallFrame::new("<script>".to_string(), 0, 0, None));
        self.open_upvalues.clear();
    }
}

impl Default for ExecutionContext {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_stack_operations() {
        let mut ctx = ExecutionContext::new();
        ctx.push(Value::Number(42.0));
        assert_eq!(ctx.stack_len(), 1);
        let val = ctx.pop().unwrap();
        assert_eq!(val, Value::Number(42.0));
        assert_eq!(ctx.stack_len(), 0);
    }

    #[test]
    fn test_context_peek() {
        let mut ctx = ExecutionContext::new();
        ctx.push(Value::Number(1.0));
        ctx.push(Value::Number(2.0));
        ctx.push(Value::Number(3.0));

        assert_eq!(ctx.peek(0), Some(&Value::Number(3.0)));
        assert_eq!(ctx.peek(1), Some(&Value::Number(2.0)));
        assert_eq!(ctx.peek(2), Some(&Value::Number(1.0)));
        assert_eq!(ctx.peek(3), None);
    }

    #[test]
    fn test_context_globals() {
        let mut ctx = ExecutionContext::new();
        ctx.define_global("x".to_string(), Value::Number(10.0));
        assert_eq!(ctx.get_global("x"), Some(Value::Number(10.0)));
        assert_eq!(ctx.get_global("y"), None);

        assert!(ctx.set_global("x", Value::Number(20.0)).is_ok());
        assert_eq!(ctx.get_global("x"), Some(Value::Number(20.0)));

        assert!(ctx.set_global("y", Value::Number(30.0)).is_err());
    }

    #[test]
    fn test_context_call_stack() {
        let mut ctx = ExecutionContext::new();
        assert_eq!(ctx.call_depth(), 1); // Script frame

        ctx.push_frame(CallFrame::new("test_fn", 100, 5, None));
        assert_eq!(ctx.call_depth(), 2);

        let frame = ctx.current_frame().unwrap();
        assert_eq!(frame.function_name.as_ref(), "test_fn");
        assert_eq!(frame.return_ip, 100);

        ctx.pop_frame();
        assert_eq!(ctx.call_depth(), 1);
    }

    #[test]
    fn test_context_ip() {
        let mut ctx = ExecutionContext::new();
        assert_eq!(ctx.current_ip(), 0);

        ctx.advance_ip(5);
        assert_eq!(ctx.current_ip(), 5);

        ctx.jump_to(100);
        assert_eq!(ctx.current_ip(), 100);

        ctx.reset_ip();
        assert_eq!(ctx.current_ip(), 0);
    }

    #[test]
    fn test_context_upvalues() {
        let mut ctx = ExecutionContext::new();
        ctx.push(Value::Number(42.0));

        let upvalue1 = ctx.capture_upvalue(0).unwrap();
        assert_eq!(*upvalue1.borrow(), Value::Number(42.0));

        // Capturing same index returns same upvalue
        let upvalue2 = ctx.capture_upvalue(0).unwrap();
        assert!(Rc::ptr_eq(&upvalue1, &upvalue2));

        // Closing upvalues - close all upvalues at index >= 1 (keeps index 0)
        ctx.close_upvalues(1);
        assert_eq!(ctx.open_upvalues.len(), 1);

        // Close all upvalues at index >= 0
        ctx.close_upvalues(0);
        assert!(ctx.open_upvalues.is_empty());
    }

    #[test]
    fn test_context_reset() {
        let mut ctx = ExecutionContext::new();
        ctx.push(Value::Number(1.0));
        ctx.define_global("x".to_string(), Value::Number(10.0));
        ctx.advance_ip(50);

        ctx.reset();

        assert_eq!(ctx.stack_len(), 0);
        assert_eq!(ctx.globals().borrow().len(), 0);
        assert_eq!(ctx.current_ip(), 0);
        assert_eq!(ctx.call_depth(), 1); // Script frame restored
    }

    #[test]
    fn test_cow_globals_isolation() {
        let mut ctx = ExecutionContext::new();
        ctx.define_global("x".to_string(), Value::Number(10.0));

        // Snapshot
        let snapshot = ctx.snapshot_globals();

        // Modify
        ctx.set_global("x", Value::Number(20.0)).unwrap();

        // Snapshot should be unchanged
        assert_eq!(snapshot.borrow().get("x"), Some(&Value::Number(10.0)));
        assert_eq!(ctx.get_global("x"), Some(Value::Number(20.0)));
    }

    #[test]
    fn test_cow_triggers_copy() {
        let mut ctx = ExecutionContext::new();
        ctx.define_global("x".to_string(), Value::Number(10.0));

        let snapshot = ctx.snapshot_globals();

        // This should trigger COW
        ctx.set_global("x", Value::Number(20.0)).unwrap();

        // After COW, snapshot and ctx should have different Rc instances
        assert!(!Rc::ptr_eq(&snapshot, &ctx.globals()));

        // Snapshot should still have the old value
        assert_eq!(snapshot.borrow().get("x"), Some(&Value::Number(10.0)));

        // Context should have the new value
        assert_eq!(ctx.get_global("x"), Some(Value::Number(20.0)));
    }

    #[test]
    fn test_snapshot_restore() {
        let mut ctx = ExecutionContext::new();
        ctx.define_global("x".to_string(), Value::Number(10.0));
        ctx.define_global("y".to_string(), Value::Number(20.0));

        // Snapshot
        let snapshot = ctx.snapshot_globals();

        // Modify
        ctx.define_global("z".to_string(), Value::Number(30.0));
        ctx.set_global("x", Value::Number(100.0)).unwrap();

        // Restore
        ctx.restore_globals(snapshot);

        // Should have original values
        assert_eq!(ctx.get_global("x"), Some(Value::Number(10.0)));
        assert_eq!(ctx.get_global("y"), Some(Value::Number(20.0)));
        assert_eq!(ctx.get_global("z"), None); // z was added after snapshot
    }
}
