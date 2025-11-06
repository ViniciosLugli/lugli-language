use crate::{
    Bytecode, Instruction,
    module::{ModuleCache, ModuleResolver},
};
use hashbrown::HashMap;
use lugli_common::{LugliError, StringId, StringPool, Value};
use lugli_stdlib::get_global_functions;
use std::{cell::RefCell, path::PathBuf, rc::Rc, time::Instant};

// Instruction handler modules
mod stack_ops;
mod arithmetic_ops;
mod variable_ops;
mod object_ops;
mod control_flow;
mod utility_ops;

const MAX_STACK_SIZE: usize = 10_000;
const MAX_CALL_DEPTH: usize = 1000;

#[derive(Debug, Clone)]
struct CallFrame {
    function_name: Rc<str>,
    return_ip: usize,
    stack_base: usize,
    closure_upvalues: Option<Vec<Rc<RefCell<Value>>>>,
}

impl CallFrame {
    fn new(function_name: impl Into<Rc<str>>, return_ip: usize, stack_base: usize, upvalues: Option<Vec<Rc<RefCell<Value>>>>) -> Self {
        Self {
            function_name: function_name.into(),
            return_ip,
            stack_base,
            closure_upvalues: upvalues,
        }
    }
}

#[derive(Debug)]
pub struct DebugContext {
    pub trace_execution: bool,
    pub trace_stack: bool,
    pub trace_calls: bool,
    pub breakpoints: Vec<usize>,
    pub step_mode: bool,
    pub instruction_count: usize,
    pub start_time: Option<Instant>,
}

impl Default for DebugContext {
    fn default() -> Self {
        Self {
            trace_execution: std::env::var("LUGLI_TRACE_EXEC").is_ok(),
            trace_stack: std::env::var("LUGLI_TRACE_STACK").is_ok(),
            trace_calls: std::env::var("LUGLI_TRACE_CALLS").is_ok(),
            breakpoints: Vec::new(),
            step_mode: false,
            instruction_count: 0,
            start_time: None,
        }
    }
}

pub struct Machine {
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
    pub ip: usize,
    call_stack: Vec<CallFrame>,
    pub debug: DebugContext,
    module_cache: ModuleCache,
    module_resolver: ModuleResolver,
    current_file: Option<PathBuf>,
    string_pool: Rc<RefCell<StringPool>>,
    bytecode_registry: HashMap<usize, Rc<Bytecode>>,
    module_globals: HashMap<usize, Rc<HashMap<String, Value>>>,
    next_bytecode_id: usize,
    open_upvalues: HashMap<usize, Rc<RefCell<Value>>>,
    method_registry: lugli_stdlib::MethodRegistry,
    method_cache: HashMap<(usize, usize), u32>,
    gc: lugli_common::GarbageCollector,
}

impl Machine {
    fn execute_binary_op(&mut self, op: fn(&Value, &Value) -> Result<Value, LugliError>) -> Result<(), LugliError> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(op(&a, &b)?);
        Ok(())
    }

    pub fn new() -> Self {
        let string_pool = Rc::new(RefCell::new(StringPool::with_common_strings()));
        let global_functions = get_global_functions();
        let mut globals = HashMap::with_capacity(global_functions.len());
        for (name, func) in global_functions {
            globals.insert(
                name.to_string(),
                Value::NativeFunction {
                    name: name.to_string(),
                    callback: func,
                    arity: 0,
                },
            ); // Arity is a placeholder here
        }

        let mut debug = DebugContext::default();
        if debug.trace_execution || debug.trace_stack || debug.trace_calls {
            debug.start_time = Some(Instant::now());
        }

        let mut call_stack = Vec::with_capacity(64);
        call_stack.push(CallFrame::new("<script>".to_string(), 0, 0, None));

        Self {
            stack: Vec::with_capacity(256),
            globals,
            ip: 0,
            call_stack,
            debug,
            module_cache: ModuleCache::new(),
            module_resolver: ModuleResolver::new(),
            current_file: None,
            string_pool,
            bytecode_registry: HashMap::with_capacity(16),
            module_globals: HashMap::with_capacity(16),
            next_bytecode_id: 1,
            open_upvalues: HashMap::with_capacity(32),
            method_registry: lugli_stdlib::MethodRegistry::new(),
            method_cache: HashMap::with_capacity(256),
            gc: lugli_common::GarbageCollector::new(),
        }
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.globals.clear();
        self.ip = 0;
        self.call_stack.clear();
        self.call_stack.push(CallFrame::new("<script>".to_string(), 0, 0, None));
        self.debug = DebugContext::default();
        if self.debug.trace_execution || self.debug.trace_stack || self.debug.trace_calls {
            self.debug.start_time = Some(Instant::now());
        }
        self.module_cache.clear();
        self.current_file = None;
        self.bytecode_registry.clear();
        self.next_bytecode_id = 1; // Start at 1, main bytecode uses ID 0
        self.open_upvalues.clear();
    }

    pub fn reset_for_repl(&mut self) {
        self.stack.clear();
        self.ip = 0;
        self.call_stack.clear();
        self.call_stack.push(CallFrame::new("<script>".to_string(), 0, 0, None));

        // GC bytecodes: only keep those referenced by globals
        self.gc_bytecodes();

        self.open_upvalues.clear();
    }

    /// Garbage collect bytecodes that are no longer referenced by globals
    fn gc_bytecodes(&mut self) {
        use std::collections::HashSet;

        // Mark phase: collect all bytecode IDs referenced by globals
        let mut live_bytecodes = HashSet::new();

        for value in self.globals.values() {
            self.mark_bytecode_ids(value, &mut live_bytecodes);
        }

        // Sweep phase: remove unreferenced bytecodes
        self.bytecode_registry.retain(|&id, _| live_bytecodes.contains(&id));
    }

    /// Get current GC statistics
    pub fn gc_stats(&self) -> lugli_common::GCStats { self.gc.stats() }

    /// Recursively mark bytecode IDs referenced by a value
    fn mark_bytecode_ids(&self, value: &Value, live_set: &mut std::collections::HashSet<usize>) {
        match value {
            Value::Function {
                bytecode_id, ..
            } => {
                live_set.insert(*bytecode_id);
            }
            Value::Closure {
                bytecode_id,
                upvalues,
                ..
            } => {
                live_set.insert(*bytecode_id);
                for upvalue in upvalues {
                    self.mark_bytecode_ids(&upvalue.borrow(), live_set);
                }
            }
            Value::List(list) => {
                for item in list.borrow().iter() {
                    self.mark_bytecode_ids(item, live_set);
                }
            }
            Value::Dict(dict) => {
                for val in dict.borrow().values() {
                    self.mark_bytecode_ids(val, live_set);
                }
            }
            Value::StructInstance {
                fields, ..
            } => {
                for val in fields.values() {
                    self.mark_bytecode_ids(val, live_set);
                }
            }
            Value::Module {
                exports, ..
            } => {
                for val in exports.values() {
                    self.mark_bytecode_ids(val, live_set);
                }
            }
            _ => {}
        }
    }

    /// Resolve a StringId using hybrid approach: try shared pool first, then bytecode pools
    fn resolve_string_id(&self, id: StringId) -> Option<String> {
        // Try shared pool first (fast path)
        {
            let pool = self.string_pool.borrow();
            if let Some(s) = pool.try_resolve(id) {
                return Some(s.to_string());
            }
        }

        // Fallback: search bytecode pools
        for bytecode in self.bytecode_registry.values() {
            let pool = bytecode.string_pool.borrow();
            if let Some(s) = pool.try_resolve(id) {
                return Some(s.to_string());
            }
        }

        None
    }

    pub fn format_value(&self, value: &Value) -> String {
        use lugli_common::Value;

        match value {
            Value::String(id) => {
                // Try shared string pool first (fast path for modules)
                let pool = self.string_pool.borrow();
                if let Some(s) = pool.try_resolve(*id) {
                    return format!("\"{}\"", s);
                }
                drop(pool);

                // Fallback: search registered bytecode pools (for REPL/standalone bytecode)
                for bytecode in self.bytecode_registry.values() {
                    let pool = bytecode.string_pool.borrow();
                    if let Some(s) = pool.try_resolve(*id) {
                        return format!("\"{}\"", s);
                    }
                }

                format!("<string#{}>", id.as_u32())
            }
            Value::List(l) => match l.try_borrow() {
                Ok(list_ref) => {
                    let items: Vec<String> = list_ref.iter().map(|v| self.format_value(v)).collect();
                    format!("[{}]", items.join(", "))
                }
                Err(_) => "[<borrowed list>]".to_string(),
            },
            Value::Dict(d) => match d.try_borrow() {
                Ok(dict_ref) => {
                    let mut items = Vec::new();
                    for (k, v) in dict_ref.iter() {
                        // Try shared pool first, then fall back to bytecode pools
                        let key_str = {
                            let pool = self.string_pool.borrow();
                            if let Some(s) = pool.try_resolve(*k) {
                                s.to_string()
                            } else {
                                drop(pool);
                                // Search bytecode pools
                                let mut found = None;
                                for bytecode in self.bytecode_registry.values() {
                                    let pool = bytecode.string_pool.borrow();
                                    if let Some(s) = pool.try_resolve(*k) {
                                        found = Some(s.to_string());
                                        break;
                                    }
                                }
                                found.unwrap_or_else(|| format!("<string#{}>", k.as_u32()))
                            }
                        };
                        items.push(format!("\"{}\": {}", key_str, self.format_value(v)));
                    }
                    format!("{{ {} }}", items.join(", "))
                }
                Err(_) => "{<borrowed dict>}".to_string(),
            },
            _ => value.to_string(),
        }
    }

    fn peek(&self) -> Result<&Value, LugliError> { self.stack.last().ok_or_else(|| LugliError::runtime("Stack underflow")) }

    fn peek_n(&self, n: usize) -> Result<&Value, LugliError> {
        let index = self.stack.len().checked_sub(1 + n).ok_or_else(|| LugliError::runtime("Stack underflow in peek_n"))?;
        self.stack.get(index).ok_or_else(|| LugliError::runtime("Invalid stack access"))
    }

    fn pop(&mut self) -> Result<Value, LugliError> { self.stack.pop().ok_or_else(|| LugliError::runtime("Stack underflow")) }

    fn generate_stack_trace(&self, bytecode: &Bytecode) -> Vec<String> {
        let mut traces = Vec::new();

        for (i, frame) in self.call_stack.iter().rev().enumerate() {
            let ip = if i == 0 { self.ip } else { frame.return_ip };

            let trace = if let Some(location) = bytecode.get_location(ip) {
                if location.line > 0 {
                    format!("  at {} in {}:{}:{}", frame.function_name, location.file_path, location.line, location.column)
                } else {
                    format!("  at {} (ip {:04})", frame.function_name, ip)
                }
            } else {
                format!("  at {} (ip {:04})", frame.function_name, ip)
            };

            traces.push(trace);
        }

        // Add summary info if debug enabled
        if self.debug.trace_execution && self.debug.instruction_count > 0 {
            traces.push(format!("  [Executed {} instructions]", self.debug.instruction_count));
        }

        traces
    }

    fn get_constant<'a>(&self, bytecode: &'a Bytecode, index: usize) -> Result<&'a Value, LugliError> {
        bytecode
            .constants
            .get(index)
            .ok_or_else(|| LugliError::runtime(format!("Invalid constant index {} (total: {})", index, bytecode.constants.len())))
    }

    fn get_source_context(&self, bytecode: &Bytecode) -> Option<lugli_common::SourceContext> {
        let location = bytecode.get_location(self.ip)?;
        if location.line == 0 {
            return None;
        }

        let mut context = lugli_common::SourceContext::new(location.file_path.clone(), location.line, location.column, location.span);

        if let Some(ref source_code) = bytecode.source_code {
            let lines: Vec<&str> = source_code.lines().collect();
            if location.line > 0 && location.line <= lines.len() {
                context = context.with_source(lines[location.line - 1].to_string());
            }
        }

        Some(context)
    }

    fn list_filter(&mut self, list: Rc<RefCell<Vec<Value>>>, predicate: &Value, bytecode: &Bytecode) -> Result<Value, LugliError> {
        let items = {
            let borrowed = list.borrow();
            let mut items_vec = Vec::with_capacity(borrowed.len());
            items_vec.extend(borrowed.iter().cloned());
            items_vec
        };

        let mut filtered = Vec::new();
        for item in items {
            let result = self.call_user_function(predicate, std::slice::from_ref(&item), bytecode)?;
            if result.is_truthy() {
                filtered.push(item);
            }
        }

        Ok(Value::List(Rc::new(RefCell::new(filtered))))
    }

    fn list_map(&mut self, list: Rc<RefCell<Vec<Value>>>, mapper: &Value, bytecode: &Bytecode) -> Result<Value, LugliError> {
        let items = {
            let borrowed = list.borrow();
            let mut items_vec = Vec::with_capacity(borrowed.len());
            items_vec.extend(borrowed.iter().cloned());
            items_vec
        };

        let mut mapped = Vec::new();
        for item in items {
            let result = self.call_user_function(mapper, std::slice::from_ref(&item), bytecode)?;
            mapped.push(result);
        }

        Ok(Value::List(Rc::new(RefCell::new(mapped))))
    }

    fn call_user_function(&mut self, function: &Value, args: &[Value], bytecode: &Bytecode) -> Result<Value, LugliError> {
        match function {
            Value::NativeFunction {
                callback,
                arity,
                ..
            } => {
                if args.len() != *arity {
                    return Err(LugliError::runtime(format!("Function expects {} arguments, got {}", arity, args.len())));
                }
                callback(args, &mut bytecode.string_pool.borrow_mut())
            }
            Value::Function {
                name,
                params,
                body_start,
                bytecode_id,
            }
            | Value::Closure {
                name,
                params,
                body_start,
                bytecode_id,
                ..
            } => {
                if args.len() != params.len() {
                    return Err(LugliError::runtime(format!("Function expects {} arguments, got {}", params.len(), args.len())));
                }

                // Look up the correct bytecode from the registry
                let function_bytecode = self
                    .bytecode_registry
                    .get(bytecode_id)
                    .ok_or_else(|| LugliError::runtime(format!("Internal error: Bytecode ID {} not found in registry", bytecode_id)))?
                    .clone();

                // Save current state for cleanup on error
                let saved_stack_len = self.stack.len();
                let saved_ip = self.ip;
                let saved_call_stack_len = self.call_stack.len();

                // Push function and arguments onto stack (function call protocol)
                self.stack.push(function.clone());
                for arg in args {
                    self.stack.push(arg.clone());
                }

                // Set up call frame - stack_base points to first argument (after function)
                let stack_base = saved_stack_len + 1; // +1 to skip the function itself
                let frame = CallFrame::new(name.clone(), usize::MAX, stack_base, None);
                self.call_stack.push(frame);

                // Save and restore module globals for cross-bytecode execution
                let saved_globals = self.globals.clone();
                if bytecode_id != &0 {
                    if let Some(module_globals) = self.module_globals.get(bytecode_id) {
                        self.globals = (**module_globals).clone();
                    }
                }

                // Jump to function body
                self.ip = *body_start;

                // Execute function body and handle result/errors - use the correct bytecode!
                let execution_result = self.execute_function_until_return(&function_bytecode, saved_call_stack_len);

                // Always restore IP and globals, even on error
                self.ip = saved_ip;
                self.globals = saved_globals;

                // Handle execution result
                match execution_result {
                    Ok(()) => {
                        // Function returned successfully, get return value
                        if self.stack.len() > saved_stack_len {
                            self.stack.pop().ok_or_else(|| LugliError::runtime("Internal error: Function return value missing from stack"))
                        } else {
                            // Stack underflow - function didn't leave return value
                            Err(LugliError::runtime(format!("Function '{}' returned without value on stack", name)))
                        }
                    }
                    Err(e) => {
                        // Error during execution - cleanup before propagating
                        self.stack.truncate(saved_stack_len);
                        self.call_stack.truncate(saved_call_stack_len);
                        Err(e)
                    }
                }
            }
            _ => Err(LugliError::runtime(format!("Value is not callable: {}", function.type_name()))),
        }
    }

    fn execute_function_until_return(&mut self, bytecode: &Bytecode, target_call_depth: usize) -> Result<(), LugliError> {
        loop {
            // Check if function returned (call frame popped) FIRST, before bounds check
            // Return instruction may set IP to usize::MAX as a signal, so check this first
            // Use <= because when Return pops the frame, len becomes equal to target_depth
            if self.call_stack.len() <= target_call_depth {
                return Ok(());
            }

            // Bounds check
            if self.ip >= bytecode.instructions.len() {
                return Err(LugliError::runtime("Function execution ran past end of bytecode"));
            }

            // Execute one instruction
            let instruction = &bytecode.instructions[self.ip];
            let continue_execution = self.execute_instruction(instruction, bytecode)?;

            // Stop if instruction indicated halt
            if !continue_execution {
                return Ok(());
            }
        }
    }

    fn load_module(&mut self, module_path: &str) -> Result<HashMap<String, Value>, LugliError> {
        let resolved_path = self.module_resolver.resolve(module_path, self.current_file.as_deref())?;

        // Prevent self-import
        if let Some(current) = self.current_file.as_ref().filter(|c| resolved_path == **c) {
            return Err(LugliError::runtime(format!("Module cannot import itself: '{}'", current.display())));
        }

        if self.module_cache.is_loading(&resolved_path) {
            return Err(LugliError::runtime(format!("Circular import detected: module '{}' is already being loaded", resolved_path.display())));
        }

        if let Some(cached_module) = self.module_cache.get(&resolved_path) {
            return Ok(cached_module.exports.clone());
        }

        self.module_cache.mark_loading(resolved_path.clone());

        let source = std::fs::read_to_string(&resolved_path)
            .map_err(|e| LugliError::runtime(format!("Failed to read module '{}': {}", resolved_path.display(), e)))?;

        let mut parser = lugli_parser::Parser::new(&source)
            .map_err(|e| LugliError::runtime(format!("Parse error in module '{}': {}", resolved_path.display(), e)))?;
        let (ast, span_map) =
            parser.parse().map_err(|e| LugliError::runtime(format!("Parse error in module '{}': {}", resolved_path.display(), e)))?;

        let shared_pool = if let Some(main_bytecode) = self.bytecode_registry.get(&0) {
            Rc::clone(&main_bytecode.string_pool)
        } else {
            Rc::new(RefCell::new(lugli_common::StringPool::with_common_strings()))
        };

        let mut compiler = crate::Compiler::with_shared_pool(shared_pool);
        let module_bytecode = compiler
            .compile(&ast, span_map)
            .map_err(|e| LugliError::runtime(format!("Compile error in module '{}': {}", resolved_path.display(), e)))?;

        let module_bytecode_id = self.next_bytecode_id;
        self.next_bytecode_id += 1;
        self.bytecode_registry.insert(module_bytecode_id, Rc::new(module_bytecode.clone()));

        let saved_globals = self.globals.clone();
        let saved_file = self.current_file.clone();
        let saved_stack_len = self.stack.len();
        let saved_ip = self.ip;
        let saved_call_stack = self.call_stack.clone();

        self.globals = HashMap::new();
        for (name, func) in get_global_functions() {
            self.globals.insert(
                name.to_string(),
                Value::NativeFunction {
                    name: name.to_string(),
                    callback: func,
                    arity: 0,
                },
            );
        }
        self.current_file = Some(resolved_path.clone());

        self.ip = 0;
        let mut result = Ok(());
        while self.ip < module_bytecode.instructions.len() {
            let instruction = &module_bytecode.instructions[self.ip];
            match self.execute_instruction(instruction, &module_bytecode) {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    result = Err(e);
                    break;
                }
            }
        }

        result?;

        let mut module_exports = self.globals.clone();

        // Update all functions in exports to have correct bytecode_id
        for (_, value) in module_exports.iter_mut() {
            match value {
                Value::Function {
                    bytecode_id, ..
                } => {
                    *bytecode_id = module_bytecode_id;
                }
                Value::Closure {
                    bytecode_id, ..
                } => {
                    *bytecode_id = module_bytecode_id;
                }
                _ => {}
            }
        }

        // Store module globals for cross-bytecode function calls
        self.module_globals.insert(module_bytecode_id, Rc::new(module_exports.clone()));

        self.globals = saved_globals;
        self.current_file = saved_file;
        self.stack.truncate(saved_stack_len);
        self.ip = saved_ip;
        self.call_stack = saved_call_stack;

        self.module_cache.unmark_loading(&resolved_path);

        let module = crate::module::Module::with_globals(
            resolved_path.clone(),
            module_bytecode,
            module_exports.clone(),
        );

        self.module_cache.insert(resolved_path, module);

        Ok(module_exports)
    }

    pub fn run(&mut self, bytecode: &Bytecode) -> Result<Value, LugliError> {
        // Register main bytecode with ID 0
        let bytecode_rc = Rc::new(bytecode.clone());
        self.bytecode_registry.insert(0, bytecode_rc.clone());

        self.ip = 0;
        while self.ip < bytecode.instructions.len() {
            let instruction = &bytecode.instructions[self.ip];
            match self.execute_instruction(instruction, bytecode) {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    if let LugliError::Runtime(data) = e {
                        let context = self.get_source_context(bytecode);
                        return Err(LugliError::runtime_full(
                            data.message,
                            data.code,
                            context.or(data.context),
                            data.suggestion,
                            self.generate_stack_trace(bytecode),
                        ));
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        self.pop().or(Ok(Value::Null))
    }

    fn execute_instruction(&mut self, instruction: &Instruction, bytecode: &Bytecode) -> Result<bool, LugliError> {
        // Stack overflow protection
        if self.stack.len() > MAX_STACK_SIZE {
            return Err(LugliError::runtime(format!("Stack overflow: exceeded maximum stack size of {} values", MAX_STACK_SIZE)));
        }

        if self.call_stack.len() > MAX_CALL_DEPTH {
            return Err(LugliError::runtime(format!("Maximum recursion depth exceeded: {} nested calls", MAX_CALL_DEPTH)));
        }

        // Increment instruction counter for GC and debugging
        self.debug.instruction_count += 1;

        // Debug tracing
        if self.debug.trace_execution {
            eprintln!("[TRACE] IP:{:04} | {:?}", self.ip, instruction);
        }

        // TODO: GC is currently disabled because:
        // 1. It clones entire VM state (expensive)
        // 2. sweep() is a no-op - doesn't actually free memory
        // 3. Rust's Rc<RefCell<>> handles reference counting automatically
        // Need to implement proper tri-color marking GC or remove GC tracking entirely
        // if self.debug.instruction_count % 10_000 == 0 {
        //     let mut roots = Vec::with_capacity(self.globals.len() + self.stack.len());
        //     roots.extend(self.globals.values().cloned());
        //     roots.extend(self.stack.iter().cloned());
        //     self.gc.collect(&roots);
        // }

        if self.debug.trace_stack && !self.stack.is_empty() {
            eprintln!("[STACK] depth:{} top:{:?}", self.stack.len(), self.stack.last());
        }

        let inst_start = if self.debug.trace_execution { Some(Instant::now()) } else { None };

        match instruction {
            // Stack operations
            Instruction::Constant(index) => self.exec_constant(bytecode, *index)?,
            Instruction::LoadSmallInt(n) => self.exec_load_small_int(*n)?,
            Instruction::LoadInt(n) => self.exec_load_int(*n)?,
            Instruction::LoadTrue => self.exec_load_true()?,
            Instruction::LoadFalse => self.exec_load_false()?,
            Instruction::LoadNull => self.exec_load_null()?,
            Instruction::ReserveLocals(count) => self.exec_reserve_locals(*count)?,
            Instruction::Pop => self.exec_pop()?,
            Instruction::Dup => self.exec_dup()?,
            Instruction::Print => self.exec_print()?,

            // Arithmetic and logical operations
            Instruction::Add => self.exec_add(bytecode)?,
            Instruction::Subtract => self.exec_subtract()?,
            Instruction::Multiply => self.exec_multiply()?,
            Instruction::Divide => self.exec_divide()?,
            Instruction::IntegerDivide => self.exec_integer_divide()?,
            Instruction::Modulo => self.exec_modulo()?,
            Instruction::Power => self.exec_power()?,
            Instruction::AddInt(n) => self.exec_add_int(*n)?,
            Instruction::SubInt(n) => self.exec_sub_int(*n)?,
            Instruction::MulInt(n) => self.exec_mul_int(*n)?,
            Instruction::Negate => self.exec_negate()?,
            Instruction::Not => self.exec_not()?,
            Instruction::And => self.exec_and()?,
            Instruction::Or => self.exec_or()?,
            Instruction::Equal => self.exec_equal()?,
            Instruction::NotEqual => self.exec_not_equal()?,
            Instruction::Greater => self.exec_greater()?,
            Instruction::GreaterEqual => self.exec_greater_equal()?,
            Instruction::Less => self.exec_less()?,
            Instruction::LessEqual => self.exec_less_equal()?,
            // Control flow operations
            Instruction::Jump(addr) => return self.exec_jump(*addr),
            Instruction::JumpIfFalse(addr) => {
                if self.exec_jump_if_false(*addr)? {
                    return Ok(true);
                }
            }
            Instruction::Loop(start) => return self.exec_loop(*start),
            Instruction::Call(arg_count) => {
                let arg_count = *arg_count as usize;
                let callee = self.peek_n(0)?.clone_for_stack(); // Callee is at the top

                if self.debug.trace_calls {
                    let func_name = match &callee {
                        Value::NativeFunction {
                            name, ..
                        } => format!("<native:{}>", name),
                        Value::Function {
                            name, ..
                        }
                        | Value::Closure {
                            name, ..
                        } => name.clone(),
                        _ => "<unknown>".to_string(),
                    };
                    eprintln!("[CALL] {} with {} arguments", func_name, arg_count);
                }

                match callee {
                    Value::NativeFunction {
                        name,
                        callback,
                        ..
                    } => {
                        if self.debug.trace_calls {
                            eprintln!("[CALL] Executing native function: {}", name);
                        }
                        let args_start_index = self.stack.len().checked_sub(arg_count + 1).ok_or_else(|| {
                            LugliError::runtime(format!(
                                "Stack underflow: need {} arguments but stack only has {} elements",
                                arg_count,
                                self.stack.len()
                            ))
                        })?;
                        let args_end_index = self.stack.len() - 1;
                        let args = self.stack.get(args_start_index..args_end_index).ok_or_else(|| {
                            LugliError::runtime(format!(
                                "Stack corruption: invalid argument range [{}, {}) for function '{}'",
                                args_start_index, args_end_index, name
                            ))
                        })?;
                        let result = {
                            let mut pool = bytecode.string_pool.borrow_mut();
                            callback(args, &mut pool)?
                        };
                        self.stack.truncate(self.stack.len() - arg_count - 1);
                        self.stack.push(result);
                    }
                    Value::Function {
                        name,
                        params,
                        body_start,
                        bytecode_id,
                    } => {
                        let arity = params.len();
                        if arg_count != arity {
                            return Err(LugliError::runtime(format!("Function expects {} arguments, got {}", arity, arg_count)));
                        }

                        // Check if this is a cross-bytecode call (module function)
                        if bytecode_id != 0 {
                            // Cross-bytecode call - need to execute in function's bytecode
                            let function_bytecode = self
                                .bytecode_registry
                                .get(&bytecode_id)
                                .ok_or_else(|| LugliError::runtime(format!("Internal error: Bytecode ID {} not found", bytecode_id)))?
                                .clone();

                            let saved_call_stack_len = self.call_stack.len();
                            let stack_base = self.stack.len() - arg_count - 1;
                            let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base, None);
                            self.call_stack.push(frame);

                            // Save and restore module globals for cross-bytecode execution
                            let saved_globals = self.globals.clone();
                            if let Some(module_globals) = self.module_globals.get(&bytecode_id) {
                                self.globals = (**module_globals).clone();
                            }

                            let saved_ip = self.ip;
                            self.ip = body_start;

                            let exec_result = self.execute_function_until_return(&function_bytecode, saved_call_stack_len);
                            self.ip = saved_ip; // Restore IP; execute_instruction will increment it
                            self.globals = saved_globals; // Restore original globals
                            exec_result?;

                            // Return value is now on stack
                        } else {
                            // Same-bytecode call - normal path
                            let stack_base = self.stack.len() - arg_count - 1;
                            let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base, None);
                            self.call_stack.push(frame);
                            self.ip = body_start;
                            return Ok(true);
                        }
                    }
                    Value::Closure {
                        name,
                        params,
                        body_start,
                        bytecode_id,
                        upvalues,
                    } => {
                        let arity = params.len();
                        if arg_count != arity {
                            return Err(LugliError::runtime(format!("Closure expects {} arguments, got {}", arity, arg_count)));
                        }

                        // Check if this is a cross-bytecode call (module closure)
                        if bytecode_id != 0 {
                            // Cross-bytecode call - need to execute in closure's bytecode
                            let function_bytecode = self
                                .bytecode_registry
                                .get(&bytecode_id)
                                .ok_or_else(|| LugliError::runtime(format!("Internal error: Bytecode ID {} not found", bytecode_id)))?
                                .clone();

                            let saved_call_stack_len = self.call_stack.len();
                            let stack_base = self.stack.len() - arg_count - 1;
                            let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base, Some(upvalues.clone()));
                            self.call_stack.push(frame);

                            // Save and restore module globals for cross-bytecode execution
                            let saved_globals = self.globals.clone();
                            if let Some(module_globals) = self.module_globals.get(&bytecode_id) {
                                self.globals = (**module_globals).clone();
                            }

                            let saved_ip = self.ip;
                            self.ip = body_start;

                            let exec_result = self.execute_function_until_return(&function_bytecode, saved_call_stack_len);
                            self.ip = saved_ip; // Restore IP; execute_instruction will increment it
                            self.globals = saved_globals; // Restore original globals
                            exec_result?;

                            // Return value is now on stack
                        } else {
                            // Same-bytecode call - normal path
                            let stack_base = self.stack.len() - arg_count - 1;
                            let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base, Some(upvalues.clone()));
                            self.call_stack.push(frame);
                            self.ip = body_start;
                            return Ok(true);
                        }
                    }
                    _ => {
                        return Err(LugliError::runtime(format!("Not callable: {}", callee.type_name())));
                    }
                }
            }
            Instruction::Return => return self.exec_return(),
            // Variable operations
            Instruction::Load(index) => self.exec_load(*index)?,
            Instruction::Store(index) => self.exec_store(*index)?,
            Instruction::LoadUpvalue(index) => self.exec_load_upvalue(*index)?,
            Instruction::StoreUpvalue(index) => self.exec_store_upvalue(*index)?,
            Instruction::LoadGlobal(name_index) => self.exec_load_global(bytecode, *name_index)?,
            Instruction::StoreGlobal(name_index) => self.exec_store_global(bytecode, *name_index)?,
            // Object and collection operations
            Instruction::GetProperty(name_index) => self.exec_get_property(bytecode, *name_index)?,
            Instruction::SetProperty(name_index) => self.exec_set_property(bytecode, *name_index)?,
            Instruction::MakeList(count) => self.exec_make_list(*count)?,
            Instruction::MakeDict(count) => self.exec_make_dict(bytecode, *count)?,
            Instruction::DefineFunction(function_index) => self.exec_define_function(bytecode, *function_index)?,
            Instruction::MakeClosure {
                function_index,
                capture_indices,
            } => self.exec_make_closure(bytecode, *function_index, capture_indices)?,
            Instruction::CallMethod(method_name_index, arg_count) => {
                let arg_count = *arg_count as usize;
                let method_name_id = match self.get_constant(bytecode, *method_name_index)? {
                    Value::String(name_id) => *name_id,
                    _ => return Err(LugliError::runtime("Method name must be a string")),
                };
                let method_name = bytecode.string_pool.borrow().resolve(method_name_id).to_string();

                // Get the object (it's below the arguments on the stack)
                // Validate stack has enough elements (object + args)
                if self.stack.len() < arg_count + 1 {
                    return Err(LugliError::runtime(format!(
                        "Stack underflow in CallMethod: need {} elements (1 object + {} args), have {}",
                        arg_count + 1, arg_count, self.stack.len()
                    )));
                }
                let object_index = self.stack.len() - arg_count - 1;
                let object = self.stack[object_index].clone(); // Clone to avoid long borrow

                // Check for struct method calls first
                if let Value::Dict(ref d) = object {
                    let dict_ref = d.borrow();
                    let struct_type_key = bytecode.string_pool.borrow_mut().intern("__struct_type__");
                    if let Some(Value::String(struct_type_id)) = dict_ref.get(&struct_type_key) {
                        // Clone the struct type name before dropping the borrow
                        let struct_type_id = *struct_type_id;
                        let struct_type = self.resolve_string_id(struct_type_id)
                            .ok_or_else(|| LugliError::runtime(format!("Invalid struct type id: {}", struct_type_id.as_u32())))?;

                        // Check if the property is a function field
                        let field_func = dict_ref.get(&method_name_id).cloned();
                        drop(dict_ref); // Release the borrow before method call

                        // If the field exists and is a function, call it (property call)
                        #[allow(clippy::collapsible_match)]
                        if let Some(func) = field_func {
                            match &func {
                                Value::Function {
                                    params,
                                    body_start,
                                    bytecode_id,
                                    ..
                                }
                                | Value::Closure {
                                    params,
                                    body_start,
                                    bytecode_id,
                                    ..
                                } => {
                                    // Check arity
                                    let arity = params.len();
                                    if arg_count != arity {
                                        return Err(LugliError::runtime(format!("Function expects {} arguments, got {}", arity, arg_count)));
                                    }

                                    // For property function calls, we don't pass self
                                    // Collect arguments, remove object+args from stack, push just args
                                    let args_start = self.stack.len() - arg_count;
                                    let args: Vec<Value> = self.stack.drain(args_start..).collect();

                                    // Remove the object from the stack
                                    self.stack.pop();

                                    // Push arguments back
                                    for arg in args {
                                        self.stack.push(arg);
                                    }

                                    // Now set up the call frame
                                    if *bytecode_id != 0 {
                                        // Cross-bytecode call
                                        let function_bytecode = self
                                            .bytecode_registry
                                            .get(bytecode_id)
                                            .ok_or_else(|| LugliError::runtime(format!("Internal error: Bytecode ID {} not found", bytecode_id)))?
                                            .clone();

                                        let saved_call_stack_len = self.call_stack.len();
                                        let stack_base = self.stack.len() - arg_count;
                                        let frame = CallFrame::new("".to_string(), self.ip + 1, stack_base, None);
                                        self.call_stack.push(frame);

                                        let saved_ip = self.ip;
                                        self.ip = *body_start;

                                        let exec_result = self.execute_function_until_return(&function_bytecode, saved_call_stack_len);
                                        self.ip = saved_ip + 1;
                                        exec_result?;
                                        return Ok(false);
                                    } else {
                                        // Same-bytecode call
                                        let stack_base = self.stack.len() - arg_count;
                                        let frame = CallFrame::new("".to_string(), self.ip + 1, stack_base, None);
                                        self.call_stack.push(frame);
                                        self.ip = *body_start;
                                        return Ok(true);
                                    }
                                }
                                _ => {
                                    // Not a function, fall through to check for struct
                                    // methods
                                }
                            }
                        }

                        // This is a struct instance - look up the method as StructType_methodName
                        let struct_method_name = format!("{}_{}", struct_type, method_name);

                        // Look up the method in globals first, then check all loaded modules
                        let func = self.globals.get(&struct_method_name).cloned()
                            .or_else(|| self.module_cache.find_in_exports(&struct_method_name));

                        if let Some(func) = func {
                            match func {
                                Value::Function {
                                    name,
                                    params,
                                    body_start,
                                    bytecode_id,
                                }
                                | Value::Closure {
                                    name,
                                    params,
                                    body_start,
                                    bytecode_id,
                                    ..
                                } => {
                                    // Check arity - should be args + 1 for self parameter
                                    let arity = params.len();
                                    if arg_count + 1 != arity {
                                        return Err(LugliError::runtime(format!(
                                            "Method expects {} arguments (including self), got {}",
                                            arity,
                                            arg_count + 1
                                        )));
                                    }

                                    // Check if this is a cross-bytecode call (module method)
                                    if bytecode_id != 0 {
                                        // Cross-bytecode call - need to execute in method's bytecode
                                        let method_bytecode = self
                                            .bytecode_registry
                                            .get(&bytecode_id)
                                            .ok_or_else(|| LugliError::runtime(format!("Internal error: Bytecode ID {} not found", bytecode_id)))?
                                            .clone();

                                        let saved_call_stack_len = self.call_stack.len();
                                        let stack_base = self.stack.len() - arg_count - 1; // Include the object
                                        let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base, None);
                                        self.call_stack.push(frame);

                                        // Save and restore module globals for cross-bytecode execution
                                        let saved_globals = self.globals.clone();
                                        if let Some(module_globals) = self.module_globals.get(&bytecode_id) {
                                            self.globals = (**module_globals).clone();
                                        }

                                        let saved_ip = self.ip;
                                        self.ip = body_start;

                                        let exec_result = self.execute_function_until_return(&method_bytecode, saved_call_stack_len);

                                        self.ip = saved_ip; // Restore IP; execute_instruction will increment it
                                        self.globals = saved_globals; // Restore original globals
                                        exec_result?;

                                        // Return value is now on stack, but we need to clean up
                                        // The stack has: [... object, arg1, ..., argN, return_value]
                                        // We need to remove object and args, keep only return_value
                                        let return_value = self.pop().unwrap_or(Value::Null);
                                        self.stack.truncate(object_index);
                                        self.stack.push(return_value);

                                        // Manually increment IP since we're returning early
                                        self.ip += 1;
                                        return Ok(true); // Cross-bytecode method call complete
                                    } else {
                                        // Same-bytecode call - normal path
                                        let stack_base = self.stack.len() - arg_count - 1; // Include the object
                                        let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base, None);
                                        self.call_stack.push(frame);
                                        self.ip = body_start;
                                        return Ok(true); // Function call will handle stack management
                                    }
                                }
                                Value::NativeFunction {
                                    callback, ..
                                } => {
                                    // For native functions, collect args including self
                                    let args_start = self.stack.len() - arg_count - 1;
                                    let args = self.stack.get(args_start..).ok_or_else(|| {
                                        LugliError::runtime(format!("Stack corruption: invalid argument start index {} for method call", args_start))
                                    })?;
                                    let result = callback(args, &mut bytecode.string_pool.borrow_mut())?;

                                    // Pop arguments and object, push result
                                    self.stack.truncate(object_index);
                                    self.stack.push(result);
                                    // Manually increment IP since we're returning early
                                    self.ip += 1;
                                    return Ok(true);
                                }
                                _ => {
                                    return Err(LugliError::runtime(format!("'{}' is not a function", struct_method_name)));
                                }
                            }
                        } else {
                            return Err(LugliError::runtime(format!("Struct '{}' has no method or property '{}'", struct_type, method_name)));
                        }
                    }
                }

                // Special handling for higher-order list methods that need bytecode access
                if let Value::List(ref l) = object
                    && (method_name == "filter" || method_name == "map" || method_name == "map!")
                    && arg_count == 1
                {
                    // Save current IP to restore after function calls
                    let callmethod_ip = self.ip;

                    let args_start = self.stack.len() - arg_count;
                    let function = self.stack[args_start].clone();

                    let result_list = if method_name == "filter" {
                        self.list_filter(l.clone(), &function, bytecode)?
                    } else {
                        self.list_map(l.clone(), &function, bytecode)?
                    };

                    // Pop arguments and object, push result
                    self.stack.truncate(object_index);
                    self.stack.push(result_list);

                    // Set IP to continue after this CallMethod instruction
                    // The main loop will increment IP, so we don't need to add 1
                    self.ip = callmethod_ip + 1;

                    // Return false to indicate we handled IP advancement
                    return Ok(false);
                }

                // Dispatch based on object type for built-in types using method registry
                let result = match object {
                    Value::String(_) | Value::List(_) | Value::Dict(_) => {
                        // Get receiver and arguments for registry call
                        let args_start = object_index;
                        let args_end = self.stack.len();
                        let args: Vec<Value> = self.stack[args_start..args_end].to_vec();

                        // Hash-based method lookup with cache
                        let type_id = object.type_id();
                        let cache_key = (type_id, *method_name_index);

                        let hash = if let Some(&h) = self.method_cache.get(&cache_key) {
                            h
                        } else {
                            let h = lugli_stdlib::methods::hash_method(object.type_name(), &method_name);
                            self.method_cache.insert(cache_key, h);
                            h
                        };

                        let mut pool = bytecode.string_pool.borrow_mut();
                        self.method_registry.call_by_hash(hash, &args, &mut pool)?
                    }
                    Value::Module {
                        exports, ..
                    } => {
                        // Module "method call" is actually accessing an exported function
                        // Clone the function to avoid borrow checker issues
                        let function =
                            exports.get(&method_name).ok_or_else(|| LugliError::runtime(format!("Module has no export '{}'", method_name)))?.clone();

                        // Get the arguments from stack
                        let args_start = self.stack.len() - arg_count;
                        let args: Vec<Value> = self.stack[args_start..].to_vec();

                        // Call the function
                        self.call_user_function(&function, &args, bytecode)?
                    }
                    _ => {
                        return Err(LugliError::runtime(format!("Object of type {} has no method '{}'", object.type_name(), method_name)));
                    }
                };

                // Pop arguments and object, push result
                self.stack.truncate(object_index);
                self.stack.push(result);
            }
            // Index operations
            Instruction::GetIndex => self.exec_get_index(bytecode)?,
            Instruction::SetIndex => self.exec_set_index(bytecode)?,

            // Utility and module operations
            Instruction::ToString => self.exec_to_string(bytecode)?,
            Instruction::ImportModule { module_idx, bind_name } => {
                self.exec_import_module(bytecode, *module_idx, bind_name)?
            }
            Instruction::ImportFrom { module_idx, names } => {
                self.exec_import_from(bytecode, *module_idx, names)?
            }
        };

        // Timing measurement
        if let Some(start_time) = inst_start {
            let elapsed = start_time.elapsed();
            if self.debug.trace_execution && elapsed.as_micros() > 100 {
                eprintln!("[PERF] Instruction took {:?}", elapsed);
            }
        }

        self.ip += 1;
        Ok(true)
    }
}

impl Default for Machine {
    fn default() -> Self { Self::new() }
}
