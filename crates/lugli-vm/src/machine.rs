use crate::{
    Bytecode, Instruction,
    module::{ModuleCache, ModuleResolver},
};
use hashbrown::HashMap;
use lugli_common::{LugliError, Value};
use lugli_stdlib::get_global_functions;
use std::{cell::RefCell, path::PathBuf, rc::Rc, time::Instant};

const MAX_STACK_SIZE: usize = 10_000;
const MAX_CALL_DEPTH: usize = 1000;

#[derive(Debug, Clone)]
struct CallFrame {
    function_name: String,
    return_ip: usize,
    stack_base: usize,
    closure_upvalues: Option<Vec<Rc<RefCell<Value>>>>, // If this is a closure call, store its upvalues
}

impl CallFrame {
    fn new(function_name: String, return_ip: usize, stack_base: usize) -> Self {
        Self {
            function_name,
            return_ip,
            stack_base,
            closure_upvalues: None,
        }
    }

    fn new_closure(function_name: String, return_ip: usize, stack_base: usize, upvalues: Vec<Rc<RefCell<Value>>>) -> Self {
        Self {
            function_name,
            return_ip,
            stack_base,
            closure_upvalues: Some(upvalues),
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
    bytecode_registry: HashMap<usize, Rc<Bytecode>>,
    next_bytecode_id: usize,
    open_upvalues: HashMap<usize, Rc<RefCell<Value>>>, // Track captured stack slots
    method_registry: lugli_stdlib::MethodRegistry,
}

impl Machine {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        for (name, func) in get_global_functions() {
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

        Self {
            stack: Vec::with_capacity(256),
            globals,
            ip: 0,
            call_stack: vec![CallFrame::new("<script>".to_string(), 0, 0)],
            debug,
            module_cache: ModuleCache::new(),
            module_resolver: ModuleResolver::new(),
            current_file: None,
            bytecode_registry: HashMap::new(),
            next_bytecode_id: 1, // Start at 1, main bytecode uses ID 0
            open_upvalues: HashMap::new(),
            method_registry: lugli_stdlib::MethodRegistry::new(),
        }
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.globals.clear();
        self.ip = 0;
        self.call_stack.clear();
        self.call_stack.push(CallFrame::new("<script>".to_string(), 0, 0));
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
        self.call_stack.push(CallFrame::new("<script>".to_string(), 0, 0));
        self.bytecode_registry.clear();
        self.next_bytecode_id = 1;
        self.open_upvalues.clear();
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

    #[allow(dead_code)]
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
        let mut filtered = Vec::new();
        let items = list.borrow().clone();

        for item in items {
            let result = self.call_user_function(predicate, std::slice::from_ref(&item), bytecode)?;
            if result.is_truthy() {
                filtered.push(item);
            }
        }

        Ok(Value::List(Rc::new(RefCell::new(filtered))))
    }

    fn list_map(&mut self, list: Rc<RefCell<Vec<Value>>>, mapper: &Value, bytecode: &Bytecode) -> Result<Value, LugliError> {
        let mut mapped = Vec::new();
        let items = list.borrow().clone();

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
                let frame = CallFrame::new(name.clone(), usize::MAX, stack_base);
                self.call_stack.push(frame);

                // Jump to function body
                self.ip = *body_start;

                // Execute function body and handle result/errors - use the correct bytecode!
                let execution_result = self.execute_function_until_return(&function_bytecode, saved_call_stack_len);

                // Always restore IP, even on error
                self.ip = saved_ip;

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
        let (ast, span_map) = parser.parse().map_err(|e| LugliError::runtime(format!("Parse error in module '{}': {}", resolved_path.display(), e)))?;

        let mut compiler = crate::Compiler::new();
        let module_bytecode =
            compiler.compile(&ast, span_map).map_err(|e| LugliError::runtime(format!("Compile error in module '{}': {}", resolved_path.display(), e)))?;

        // Assign and register bytecode ID for this module
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

        // Execute module bytecode directly without calling run() to avoid overwriting bytecode ID
        // 0
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

        self.globals = saved_globals;
        self.current_file = saved_file;
        self.stack.truncate(saved_stack_len);
        self.ip = saved_ip;
        self.call_stack = saved_call_stack;

        self.module_cache.unmark_loading(&resolved_path);

        let module = crate::module::Module {
            path: resolved_path.clone(),
            bytecode: module_bytecode,
            exports: module_exports.clone(),
        };

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
                        return Err(LugliError::runtime_with_trace(data.message, self.generate_stack_trace(bytecode)));
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

        // Debug tracing
        if self.debug.trace_execution {
            eprintln!("[TRACE] IP:{:04} | {:?}", self.ip, instruction);
            self.debug.instruction_count += 1;
        }

        if self.debug.trace_stack && !self.stack.is_empty() {
            eprintln!("[STACK] depth:{} top:{:?}", self.stack.len(), self.stack.last());
        }

        let inst_start = if self.debug.trace_execution { Some(Instant::now()) } else { None };

        match instruction {
            Instruction::Constant(index) => {
                let value = self.get_constant(bytecode, *index)?.clone_for_stack();
                self.stack.push(value);
            }
            Instruction::Pop => {
                self.pop()?;
            }
            Instruction::Dup => {
                let val = self.peek()?.clone_for_stack();
                self.stack.push(val);
            }
            Instruction::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = match (&a, &b) {
                    (Value::String(_), Value::String(_)) => {
                        let mut pool = bytecode.string_pool.borrow_mut();
                        a.add_with_pool(&b, &mut pool)?
                    }
                    _ => a.add(&b)?,
                };
                self.stack.push(result);
            }
            Instruction::Subtract => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.subtract(&b)?);
            }
            Instruction::Multiply => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.multiply(&b)?);
            }
            Instruction::Divide => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.divide(&b)?);
            }
            Instruction::IntegerDivide => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.integer_divide(&b)?);
            }
            Instruction::Modulo => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.modulo(&b)?);
            }
            Instruction::Power => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.power(&b)?);
            }
            Instruction::Negate => {
                let val = self.pop()?;
                self.stack.push(val.negate()?);
            }
            Instruction::Not => {
                let val = self.pop()?;
                self.stack.push(Value::Bool(!val.is_truthy()));
            }
            Instruction::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(a.is_truthy() && b.is_truthy()));
            }
            Instruction::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(a.is_truthy() || b.is_truthy()));
            }
            Instruction::Print => {
                let val = self.pop()?;
                println!("{}", val);
            }
            Instruction::Equal => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(a.equals(&b)));
            }
            Instruction::NotEqual => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(Value::Bool(!a.equals(&b)));
            }
            Instruction::Greater => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.greater(&b)?);
            }
            Instruction::GreaterEqual => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.greater_equal(&b)?);
            }
            Instruction::Less => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.less(&b)?);
            }
            Instruction::LessEqual => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.less_equal(&b)?);
            }
            Instruction::Jump(addr) => {
                self.ip = *addr;
                return Ok(true);
            }
            Instruction::JumpIfFalse(addr) => {
                let condition = self.pop()?;
                if !condition.is_truthy() {
                    self.ip = *addr;
                    return Ok(true);
                }
            }
            Instruction::Loop(start) => {
                self.ip = *start;
                return Ok(true);
            }
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
                        let args_start_index = self.stack.len() - arg_count - 1;
                        let args_end_index = self.stack.len() - 1;
                        let args = self.stack.get(args_start_index..args_end_index).ok_or_else(|| {
                            LugliError::runtime(format!(
                                "Stack corruption: invalid argument range [{}, {}) for function '{}'",
                                args_start_index, args_end_index, name
                            ))
                        })?;
                        let result = callback(args, &mut bytecode.string_pool.borrow_mut())?;
                        self.stack.truncate(self.stack.len() - arg_count - 1); // Pop args and function
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
                            let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base);
                            self.call_stack.push(frame);

                            let saved_ip = self.ip;
                            self.ip = body_start;

                            let exec_result = self.execute_function_until_return(&function_bytecode, saved_call_stack_len);
                            self.ip = saved_ip + 1; // Move past Call instruction
                            exec_result?;

                            // Return value is now on stack
                        } else {
                            // Same-bytecode call - normal path
                            let stack_base = self.stack.len() - arg_count - 1;
                            let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base);
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
                            let frame = CallFrame::new_closure(name.clone(), self.ip + 1, stack_base, upvalues.clone());
                            self.call_stack.push(frame);

                            let saved_ip = self.ip;
                            self.ip = body_start;

                            let exec_result = self.execute_function_until_return(&function_bytecode, saved_call_stack_len);
                            self.ip = saved_ip + 1; // Move past Call instruction
                            exec_result?;

                            // Return value is now on stack
                        } else {
                            // Same-bytecode call - normal path
                            let stack_base = self.stack.len() - arg_count - 1;
                            let frame = CallFrame::new_closure(name.clone(), self.ip + 1, stack_base, upvalues.clone());
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
            Instruction::Return => {
                let frame = self.call_stack.pop().ok_or_else(|| LugliError::runtime("Call stack underflow"))?;
                let return_value = self.pop().unwrap_or(Value::Null);

                // Close upvalues for this frame before returning
                let frame_base = frame.stack_base;
                let frame_end = self.stack.len();
                self.open_upvalues.retain(|&index, _| index < frame_base || index >= frame_end);

                if self.call_stack.is_empty() {
                    self.stack.push(return_value);
                    return Ok(false);
                }

                self.ip = frame.return_ip;
                self.stack.truncate(frame.stack_base);
                self.stack.push(return_value);
                return Ok(true);
            }
            Instruction::Load(index) => {
                let frame = self.call_stack.last().ok_or_else(|| LugliError::runtime("Call stack is empty"))?;
                let target_index = frame.stack_base + index;
                let value = self
                    .stack
                    .get(target_index)
                    .ok_or_else(|| {
                        LugliError::runtime(format!(
                            "Stack underflow: attempted to load local {} at index {} (stack size: {})",
                            index,
                            target_index,
                            self.stack.len()
                        ))
                    })?
                    .clone();
                self.stack.push(value);
            }
            Instruction::Store(index) => {
                let value = self.pop()?;
                let frame = self.call_stack.last().ok_or_else(|| LugliError::runtime("Call stack is empty"))?;
                let target_index = frame.stack_base + index;

                // Grow stack if necessary
                while self.stack.len() <= target_index {
                    self.stack.push(Value::Null);
                }

                self.stack[target_index] = value;
            }
            Instruction::LoadUpvalue(index) => {
                let frame = self.call_stack.last().ok_or_else(|| LugliError::runtime("Call stack is empty"))?;

                if let Some(upvalues) = &frame.closure_upvalues {
                    let upvalue_ref = upvalues
                        .get(*index)
                        .ok_or_else(|| LugliError::runtime(format!("Upvalue index {} out of bounds (have {} upvalues)", index, upvalues.len())))?;

                    let value = upvalue_ref.try_borrow().map_err(|_| LugliError::runtime("Cannot access upvalue while it's being modified"))?.clone();

                    self.stack.push(value);
                } else {
                    return Err(LugliError::runtime("LoadUpvalue used in non-closure context"));
                }
            }
            Instruction::StoreUpvalue(index) => {
                let value = self.pop()?;
                let frame = self.call_stack.last().ok_or_else(|| LugliError::runtime("Call stack is empty"))?;

                if let Some(upvalues) = &frame.closure_upvalues {
                    let upvalue_ref = upvalues
                        .get(*index)
                        .ok_or_else(|| LugliError::runtime(format!("Upvalue index {} out of bounds (have {} upvalues)", index, upvalues.len())))?;

                    *upvalue_ref.try_borrow_mut().map_err(|_| LugliError::runtime("Cannot modify upvalue while it's being used"))? = value;
                } else {
                    return Err(LugliError::runtime("StoreUpvalue used in non-closure context"));
                }
            }
            Instruction::LoadGlobal(name_index) => {
                let var_name = self.get_constant(bytecode, *name_index)?;
                if let Value::String(name_id) = var_name {
                    let name = bytecode.string_pool.borrow().resolve(*name_id).to_string();
                    if let Some(value) = self.globals.get(&name) {
                        self.stack.push(value.clone());
                    } else {
                        // Generate helpful error with suggestions
                        if let Some(context) = self.get_source_context(bytecode) {
                            let available_names: Vec<&str> = self.globals.keys().map(|s| s.as_str()).collect();
                            let suggestion = crate::error_formatter::suggest_similar_name(&name, &available_names);
                            return Err(LugliError::undefined_variable_with_context(name, context, suggestion));
                        } else {
                            return Err(LugliError::undefined_variable(name));
                        }
                    }
                } else {
                    return Err(LugliError::runtime("Variable name must be a string"));
                }
            }
            Instruction::StoreGlobal(name_index) => {
                let value = self.peek()?.clone_for_stack(); // Don't pop - leave value on stack like Store does
                let var_name = self.get_constant(bytecode, *name_index)?;
                if let Value::String(name_id) = var_name {
                    let name = bytecode.string_pool.borrow().resolve(*name_id).to_string();
                    self.globals.insert(name, value);
                } else {
                    return Err(LugliError::runtime("Variable name must be a string"));
                }
            }
            Instruction::GetProperty(name_index) => {
                let object = self.pop()?;
                let prop_name = self.get_constant(bytecode, *name_index)?;
                if let Value::String(name_id) = prop_name {
                    match object {
                        Value::Dict(dict_ref) => {
                            let dict = dict_ref.borrow();
                            if let Some(value) = dict.get(name_id) {
                                self.stack.push(value.clone());
                            } else {
                                self.stack.push(Value::Null);
                            }
                        }
                        Value::Module {
                            exports, ..
                        } => {
                            let name = bytecode.string_pool.borrow().resolve(*name_id).to_string();
                            if let Some(value) = exports.get(&name) {
                                self.stack.push(value.clone());
                            } else {
                                return Err(LugliError::runtime(format!("Module has no export '{}'", name)));
                            }
                        }
                        _ => {
                            let name = bytecode.string_pool.borrow().resolve(*name_id).to_string();
                            return Err(LugliError::runtime(format!("Cannot access property '{}' on a value of type {}", name, object.type_name())));
                        }
                    }
                } else {
                    return Err(LugliError::runtime("Property name must be a string"));
                }
            }
            Instruction::SetProperty(name_index) => {
                let value = self.pop()?;
                let object = self.pop()?;
                let prop_name = self.get_constant(bytecode, *name_index)?;

                if let Value::String(name_id) = prop_name {
                    if let Value::Dict(dict_ref) = &object {
                        dict_ref
                            .try_borrow_mut()
                            .map_err(|_| LugliError::runtime("Cannot modify struct while it's being used"))?
                            .insert(*name_id, value.clone());
                        // Push the value back as the expression result
                        self.stack.push(value);
                    } else {
                        let name = bytecode.string_pool.borrow().resolve(*name_id).to_string();
                        return Err(LugliError::runtime(format!("Cannot set property '{}' on a value of type {}", name, object.type_name())));
                    }
                } else {
                    return Err(LugliError::runtime("Property name must be a string"));
                }
            }
            Instruction::MakeList(count) => {
                let mut list = Vec::with_capacity(*count);
                for _ in 0..*count {
                    list.push(self.pop()?);
                }
                list.reverse();
                self.stack.push(Value::List(Rc::new(RefCell::new(list))));
            }
            Instruction::MakeDict(count) => {
                let mut dict = HashMap::new();
                for _ in 0..*count {
                    let value = self.pop()?;
                    let key = self.pop()?;
                    if let Value::String(key_str) = key {
                        dict.insert(key_str, value);
                    } else {
                        return Err(LugliError::runtime("Dictionary keys must be strings"));
                    }
                }

                // Check if this is a struct instantiation (has __struct_type__ field)
                let struct_type_key = bytecode.string_pool.borrow_mut().intern("__struct_type__");
                if let Some(Value::String(struct_type_id)) = dict.get(&struct_type_key) {
                    let struct_type = bytecode.string_pool.borrow().resolve(*struct_type_id).to_string();
                    // Look up struct metadata from globals
                    if let Some(Value::Dict(meta)) = self.globals.get(&struct_type) {
                        let meta_ref = meta.borrow();

                        // Get default values from struct metadata
                        let defaults_key = bytecode.string_pool.borrow_mut().intern("__defaults__");
                        if let Some(Value::Dict(defaults)) = meta_ref.get(&defaults_key) {
                            let defaults_ref = defaults.borrow();

                            // Apply defaults for missing fields
                            for (field_name, default_value) in defaults_ref.iter() {
                                if !dict.contains_key(field_name) {
                                    dict.insert(*field_name, default_value.clone());
                                }
                            }
                        }
                    }
                }

                self.stack.push(Value::Dict(Rc::new(RefCell::new(dict))));
            }
            Instruction::DefineFunction(function_index) => {
                let function_value = self.get_constant(bytecode, *function_index)?.clone();
                self.stack.push(function_value);
            }
            Instruction::MakeClosure {
                function_index,
                capture_indices,
            } => {
                // Get the base function template
                let function_value = self.get_constant(bytecode, *function_index)?;

                if let Value::Function {
                    name,
                    params,
                    body_start,
                    bytecode_id,
                } = function_value
                {
                    // Get current stack frame to calculate absolute positions
                    let frame = self.call_stack.last().ok_or_else(|| LugliError::runtime("Call stack is empty during closure creation"))?;

                    // Capture values from the stack using open upvalues pattern
                    // Convert local indices to absolute stack positions
                    let mut upvalues: Vec<Rc<RefCell<Value>>> = Vec::new();

                    for &local_index in capture_indices {
                        let absolute_index = frame.stack_base + local_index;

                        // Check if this stack slot already has a shared reference
                        let upvalue = if let Some(existing) = self.open_upvalues.get(&absolute_index) {
                            // Reuse existing shared reference
                            Rc::clone(existing)
                        } else {
                            // Create new shared reference
                            let value = self.stack.get(absolute_index).cloned().ok_or_else(|| {
                                LugliError::runtime(format!(
                                    "Invalid upvalue capture: local {} (absolute {}) out of bounds (stack size: {})",
                                    local_index,
                                    absolute_index,
                                    self.stack.len()
                                ))
                            })?;
                            let shared = Rc::new(RefCell::new(value));
                            self.open_upvalues.insert(absolute_index, Rc::clone(&shared));
                            shared
                        };

                        upvalues.push(upvalue);
                    }

                    // Create closure with captured upvalues
                    let closure = Value::Closure {
                        name: name.clone(),
                        params: params.clone(),
                        body_start: *body_start,
                        bytecode_id: *bytecode_id,
                        upvalues,
                    };

                    self.stack.push(closure);
                } else {
                    return Err(LugliError::runtime("MakeClosure requires a Function value"));
                }
            }
            Instruction::CallMethod(method_name_index, arg_count) => {
                let arg_count = *arg_count as usize;
                let method_name_id = match self.get_constant(bytecode, *method_name_index)? {
                    Value::String(name_id) => *name_id,
                    _ => return Err(LugliError::runtime("Method name must be a string")),
                };
                let method_name = bytecode.string_pool.borrow().resolve(method_name_id).to_string();

                // Get the object (it's below the arguments on the stack)
                let object_index = self.stack.len() - arg_count - 1;
                let object = &self.stack[object_index];

                // Check for struct method calls first
                if let Value::Dict(d) = object {
                    let dict_ref = d.borrow();
                    let struct_type_key = bytecode.string_pool.borrow_mut().intern("__struct_type__");
                    if let Some(Value::String(struct_type_id)) = dict_ref.get(&struct_type_key) {
                        // Clone the struct type name before dropping the borrow
                        let struct_type_id = *struct_type_id;
                        let struct_type = bytecode.string_pool.borrow().resolve(struct_type_id).to_string();

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
                                        let frame = CallFrame::new("".to_string(), self.ip + 1, stack_base);
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
                                        let frame = CallFrame::new("".to_string(), self.ip + 1, stack_base);
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

                        // Look up the global function
                        if let Some(func) = self.globals.get(&struct_method_name) {
                            match func {
                                Value::Function {
                                    name,
                                    params,
                                    body_start,
                                    ..
                                }
                                | Value::Closure {
                                    name,
                                    params,
                                    body_start,
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

                                    // Set up call frame and invoke function
                                    let stack_base = self.stack.len() - arg_count - 1; // Include the object
                                    let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base);
                                    self.call_stack.push(frame);
                                    self.ip = *body_start;
                                    return Ok(true); // Function call will handle stack management
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
                if let Value::List(l) = object
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

                        let type_name = object.type_name();
                        let mut pool = bytecode.string_pool.borrow_mut();
                        self.method_registry.call(type_name, &method_name, &args, &mut pool)?
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
            Instruction::GetIndex => {
                let index = self.pop()?;
                let object = self.pop()?;

                match (&object, &index) {
                    (Value::List(list), Value::Number(idx)) => {
                        // Validate index is finite and in valid range
                        if !idx.is_finite() {
                            return Err(LugliError::runtime(format!("Index must be a finite number, got {}", idx)));
                        }
                        if *idx < i64::MIN as f64 || *idx > i64::MAX as f64 {
                            return Err(LugliError::runtime(format!("Index {} out of valid range", idx)));
                        }

                        let list_ref = list.borrow();
                        let len = list_ref.len() as i64;
                        let idx_i64 = *idx as i64;

                        let actual_idx = if idx_i64 < 0 {
                            let positive_offset = len + idx_i64;
                            if positive_offset < 0 {
                                return Err(LugliError::runtime(format!(
                                    "Negative index {} out of range for list of length {} (minimum is -{})",
                                    idx_i64, len, len
                                )));
                            }
                            positive_offset as usize
                        } else {
                            if idx_i64 >= len {
                                return Err(LugliError::runtime(format!("Index {} out of range for list of length {}", idx_i64, len)));
                            }
                            idx_i64 as usize
                        };

                        self.stack.push(list_ref[actual_idx].clone());
                    }
                    (Value::Dict(dict), Value::String(key)) => {
                        let dict_ref = dict.borrow();
                        if let Some(value) = dict_ref.get(key) {
                            self.stack.push(value.clone());
                        } else {
                            self.stack.push(Value::Null);
                        }
                    }
                    (Value::Dict(dict), Value::Number(num)) => {
                        // Convert number to string key
                        let key_str = if num.fract() == 0.0 && num.abs() < 1e15 {
                            format!("{:.0}", num) // Format as integer
                        } else {
                            num.to_string()
                        };
                        let key_id = bytecode.string_pool.borrow_mut().intern(&key_str);
                        let dict_ref = dict.borrow();
                        if let Some(value) = dict_ref.get(&key_id) {
                            self.stack.push(value.clone());
                        } else {
                            self.stack.push(Value::Null);
                        }
                    }
                    (Value::String(s), Value::Number(idx)) => {
                        // Validate index is finite and in valid range
                        if !idx.is_finite() {
                            return Err(LugliError::runtime(format!("Index must be a finite number, got {}", idx)));
                        }
                        if *idx < i64::MIN as f64 || *idx > i64::MAX as f64 {
                            return Err(LugliError::runtime(format!("Index {} out of valid range", idx)));
                        }

                        let string = bytecode.string_pool.borrow().resolve(*s).to_string();
                        let chars: Vec<char> = string.chars().collect();
                        let len = chars.len() as i64;
                        let idx_i64 = *idx as i64;

                        let actual_idx = if idx_i64 < 0 {
                            let positive_offset = len + idx_i64;
                            if positive_offset < 0 {
                                return Err(LugliError::runtime(format!(
                                    "Negative index {} out of range for string of length {} (minimum is -{})",
                                    idx_i64, len, len
                                )));
                            }
                            positive_offset as usize
                        } else {
                            if idx_i64 >= len {
                                return Err(LugliError::runtime(format!("Index {} out of range for string of length {}", idx_i64, len)));
                            }
                            idx_i64 as usize
                        };

                        let char_str = chars[actual_idx].to_string();
                        let char_id = bytecode.string_pool.borrow_mut().intern(&char_str);
                        self.stack.push(Value::String(char_id));
                    }
                    _ => {
                        return Err(LugliError::runtime(format!("Cannot index {} with {}", object.type_name(), index.type_name())));
                    }
                }
            }
            Instruction::SetIndex => {
                let value = self.pop()?;
                let index = self.pop()?;
                let object = self.pop()?;

                match (&object, &index) {
                    (Value::List(list), Value::Number(idx)) => {
                        // Validate index is finite and in valid range
                        if !idx.is_finite() {
                            return Err(LugliError::runtime(format!("Index must be a finite number, got {}", idx)));
                        }
                        if *idx < i64::MIN as f64 || *idx > i64::MAX as f64 {
                            return Err(LugliError::runtime(format!("Index {} out of valid range", idx)));
                        }

                        let mut list_ref = list.try_borrow_mut().map_err(|_| LugliError::runtime("Cannot modify list while it's being used"))?;
                        let len = list_ref.len() as i64;
                        let idx_i64 = *idx as i64;

                        let actual_idx = if idx_i64 < 0 {
                            let positive_offset = len + idx_i64;
                            if positive_offset < 0 {
                                return Err(LugliError::runtime(format!(
                                    "Negative index {} out of range for list assignment (length {}, minimum is -{})",
                                    idx_i64, len, len
                                )));
                            }
                            positive_offset as usize
                        } else {
                            if idx_i64 >= len {
                                return Err(LugliError::runtime(format!("Index {} out of range for list assignment (length {})", idx_i64, len)));
                            }
                            idx_i64 as usize
                        };

                        list_ref[actual_idx] = value.clone();
                        self.stack.push(value); // Return the assigned value
                    }
                    (Value::Dict(dict), Value::String(key)) => {
                        dict.try_borrow_mut()
                            .map_err(|_| LugliError::runtime("Cannot modify dict while it's being used"))?
                            .insert(key.clone(), value.clone());
                        self.stack.push(value); // Return the assigned value
                    }
                    (Value::Dict(dict), Value::Number(num)) => {
                        // Convert number to string key
                        let key_str = if num.fract() == 0.0 && num.abs() < 1e15 {
                            format!("{:.0}", num) // Format as integer
                        } else {
                            num.to_string()
                        };
                        let key_id = bytecode.string_pool.borrow_mut().intern(&key_str);
                        dict.try_borrow_mut()
                            .map_err(|_| LugliError::runtime("Cannot modify dict while it's being used"))?
                            .insert(key_id, value.clone());
                        self.stack.push(value); // Return the assigned value
                    }
                    _ => {
                        return Err(LugliError::runtime(format!("Cannot set index on {} with {}", object.type_name(), index.type_name())));
                    }
                }
            }
            Instruction::ToString => {
                let value = self.pop()?;
                let pool = bytecode.string_pool.borrow();
                let string_value = match &value {
                    Value::String(s) => pool.resolve(*s).to_string(),
                    Value::Number(n) => {
                        if n.is_nan() {
                            "NaN".to_string()
                        } else if n.is_infinite() {
                            if n.is_sign_positive() { "Infinity".to_string() } else { "-Infinity".to_string() }
                        } else if n.fract() == 0.0 && n.abs() < 1e15 {
                            // Format as integer if whole number and not too large
                            format!("{:.0}", n)
                        } else {
                            n.to_string()
                        }
                    }
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    Value::List(list) => {
                        let list_ref = list.borrow();
                        let elements: Vec<String> = list_ref
                            .iter()
                            .map(|v| match v {
                                Value::String(s) => format!("\"{}\"", pool.resolve(*s)),
                                _ => v.display_with_pool(&pool),
                            })
                            .collect();
                        format!("[{}]", elements.join(", "))
                    }
                    Value::Dict(dict) => {
                        let dict_ref = dict.borrow();
                        let pairs: Vec<String> = dict_ref
                            .iter()
                            .map(|(k, v)| match v {
                                Value::String(s) => format!("\"{}\": \"{}\"", pool.resolve(*k), pool.resolve(*s)),
                                _ => format!("\"{}\": {}", pool.resolve(*k), v.display_with_pool(&pool)),
                            })
                            .collect();
                        format!("{{{}}}", pairs.join(", "))
                    }
                    Value::Function {
                        name, ..
                    }
                    | Value::Closure {
                        name, ..
                    } => format!("<function {}>", name),
                    Value::NativeFunction {
                        name, ..
                    } => format!("<native function {}>", name),
                    Value::StructInstance {
                        name, ..
                    } => format!("<{} instance>", name),
                    Value::DateTime(dt) => dt.to_string(),
                    Value::Module {
                        path, ..
                    } => format!("<module {}>", path),
                };
                drop(pool);
                let id = bytecode.string_pool.borrow_mut().intern(&string_value);
                self.stack.push(Value::String(id));
            }
            Instruction::ImportModule {
                module_idx,
                bind_name,
            } => {
                let module_path = match &bytecode.constants[*module_idx] {
                    Value::String(s) => bytecode.string_pool.borrow().resolve(*s).to_string(),
                    _ => return Err(LugliError::runtime("ImportModule: expected string constant")),
                };

                let exports = self.load_module(&module_path)?;

                let module_value = Value::Module {
                    path: module_path,
                    exports,
                };

                // Store directly in globals - no stack effect
                self.globals.insert(bind_name.clone(), module_value);
            }
            Instruction::ImportFrom {
                module_idx,
                names,
            } => {
                let module_path = match &bytecode.constants[*module_idx] {
                    Value::String(s) => bytecode.string_pool.borrow().resolve(*s).to_string(),
                    _ => return Err(LugliError::runtime("ImportFrom: expected string constant")),
                };

                let exports = self.load_module(&module_path)?;

                for name in names {
                    let value =
                        exports.get(name).ok_or_else(|| LugliError::runtime(format!("Module '{}' does not export '{}'", module_path, name)))?.clone();

                    self.globals.insert(name.clone(), value);
                }
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
