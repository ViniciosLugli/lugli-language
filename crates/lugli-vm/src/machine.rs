use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;
use hashbrown::HashMap;
use lugli_common::{Value, LugliError};
use lugli_stdlib::get_global_functions;
use crate::{Bytecode, Instruction};

pub type NativeFunction = fn(&[Value]) -> Result<Value, LugliError>;

const MAX_STACK_SIZE: usize = 10_000;
const MAX_CALL_DEPTH: usize = 1000;

#[derive(Debug, Clone)]
struct CallFrame {
    function_name: String,
    return_ip: usize,
    stack_base: usize,
    closure_upvalues: Option<Rc<RefCell<Vec<Value>>>>, // If this is a closure call, store its upvalues
}

impl CallFrame {
    fn new(function_name: String, return_ip: usize, stack_base: usize) -> Self {
        Self { function_name, return_ip, stack_base, closure_upvalues: None }
    }

    fn new_closure(function_name: String, return_ip: usize, stack_base: usize, upvalues: Rc<RefCell<Vec<Value>>>) -> Self {
        Self { function_name, return_ip, stack_base, closure_upvalues: Some(upvalues) }
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
}

impl Machine {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        for (name, func) in get_global_functions() {
            globals.insert(name.to_string(), Value::NativeFunction { name: name.to_string(), callback: func, arity: 0 }); // Arity is a placeholder here
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
    }

    fn peek(&self) -> Result<&Value, LugliError> {
        self.stack.last().ok_or_else(|| LugliError::runtime("Stack underflow"))
    }

    fn peek_n(&self, n: usize) -> Result<&Value, LugliError> {
        let index = self.stack.len().checked_sub(1 + n)
            .ok_or_else(|| LugliError::runtime("Stack underflow in peek_n"))?;
        self.stack.get(index)
            .ok_or_else(|| LugliError::runtime("Invalid stack access"))
    }

    fn pop(&mut self) -> Result<Value, LugliError> {
        self.stack.pop().ok_or_else(|| LugliError::runtime("Stack underflow"))
    }

    fn generate_stack_trace(&self, bytecode: &Bytecode) -> Vec<String> {
        let mut traces = Vec::new();

        for (i, frame) in self.call_stack.iter().rev().enumerate() {
            let ip = if i == 0 { self.ip } else { frame.return_ip };

            let trace = if let Some(location) = bytecode.get_location(ip) {
                if location.line > 0 {
                    format!("  at {} in {}:{}:{}",
                        frame.function_name,
                        location.file_path,
                        location.line,
                        location.column
                    )
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

    fn get_method_args(&self, arg_count: usize) -> Result<&[Value], LugliError> {
        let args_start = self.stack.len()
            .checked_sub(arg_count)
            .ok_or_else(|| LugliError::runtime(format!(
                "Stack underflow: method expects {} args but stack has {}",
                arg_count, self.stack.len()
            )))?;
        Ok(&self.stack[args_start..])
    }

    fn get_constant<'a>(&self, bytecode: &'a Bytecode, index: usize) -> Result<&'a Value, LugliError> {
        bytecode.constants.get(index)
            .ok_or_else(|| LugliError::runtime(format!(
                "Invalid constant index {} (total: {})",
                index, bytecode.constants.len()
            )))
    }

    #[allow(dead_code)]
    fn get_source_context(&self, bytecode: &Bytecode) -> Option<lugli_common::SourceContext> {
        let location = bytecode.get_location(self.ip)?;
        if location.line == 0 {
            return None;
        }

        let mut context = lugli_common::SourceContext::new(
            location.file_path.clone(),
            location.line,
            location.column,
            location.span,
        );

        if let Some(ref source_code) = bytecode.source_code {
            let lines: Vec<&str> = source_code.lines().collect();
            if location.line > 0 && location.line <= lines.len() {
                context = context.with_source(lines[location.line - 1].to_string());
            }
        }

        Some(context)
    }

    fn call_string_method(&mut self, method: &str, s: String, arg_count: usize) -> Result<Value, LugliError> {
        let args = self.get_method_args(arg_count)?;

        

        match method {
            "len" => Ok(Value::Number(s.len() as f64)),
            "trim" => Ok(Value::String(s.trim().to_string())),
            "lower" => Ok(Value::String(s.to_lowercase())),
            "upper" => Ok(Value::String(s.to_uppercase())),
            "chars" => {
                let chars: Vec<Value> = s.chars()
                    .map(|c| Value::String(c.to_string()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(chars))))
            }
            "split" => {
                if arg_count != 1 {
                    return Err(LugliError::runtime("split expects 1 argument (separator)"));
                }
                let separator = match &args[0] {
                    Value::String(sep) => sep.as_str(),
                    _ => return Err(LugliError::runtime("split separator must be a string")),
                };
                let parts: Vec<Value> = s.split(separator)
                    .map(|part| Value::String(part.to_string()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(parts))))
            }
            "is_alphabetic" => Ok(Value::Bool(s.chars().all(|c| c.is_alphabetic()))),
            "starts_with" => {
                if arg_count != 1 {
                    return Err(LugliError::runtime("starts_with expects 1 argument"));
                }
                match &args[0] {
                    Value::String(prefix) => Ok(Value::Bool(s.starts_with(prefix.as_str()))),
                    _ => Err(LugliError::runtime("starts_with argument must be a string")),
                }
            }
            "ends_with" => {
                if arg_count != 1 {
                    return Err(LugliError::runtime("ends_with expects 1 argument"));
                }
                match &args[0] {
                    Value::String(suffix) => Ok(Value::Bool(s.ends_with(suffix.as_str()))),
                    _ => Err(LugliError::runtime("ends_with argument must be a string")),
                }
            }
            "contains" => {
                if arg_count != 1 {
                    return Err(LugliError::runtime("contains expects 1 argument"));
                }
                match &args[0] {
                    Value::String(substring) => Ok(Value::Bool(s.contains(substring.as_str()))),
                    _ => Err(LugliError::runtime("contains argument must be a string")),
                }
            }
            _ => Err(LugliError::runtime(format!("String has no method '{}'", method))),
        }
    }

    fn call_list_method(&mut self, method: &str, list: Rc<RefCell<Vec<Value>>>, arg_count: usize) -> Result<Value, LugliError> {
        let args = self.get_method_args(arg_count)?;

        match method {
            "len" => Ok(Value::Number(list.borrow().len() as f64)),
            "push" | "push!" => {
                if arg_count != 1 {
                    return Err(LugliError::runtime("push expects 1 argument"));
                }
                list.try_borrow_mut()
                    .map_err(|_| LugliError::runtime("Cannot modify list while it's being used"))?
                    .push(args[0].clone());
                Ok(Value::Null)
            }
            "pop" | "pop!" => {
                let val = list.try_borrow_mut()
                    .map_err(|_| LugliError::runtime("Cannot modify list while it's being used"))?
                    .pop();
                Ok(val.unwrap_or(Value::Null))
            }
            "join" => {
                let separator = if arg_count > 0 {
                    match &args[0] {
                        Value::String(s) => s.as_str(),
                        _ => return Err(LugliError::runtime("join separator must be a string")),
                    }
                } else {
                    ""
                };
                let strings: Vec<String> = list.borrow()
                    .iter()
                    .map(|v| v.to_string())
                    .collect();
                Ok(Value::String(strings.join(separator)))
            }
            "contains" => {
                if arg_count != 1 {
                    return Err(LugliError::runtime("contains expects 1 argument"));
                }
                let contains = list.borrow().iter().any(|v| v.equals(&args[0]));
                Ok(Value::Bool(contains))
            }
            "is_empty" => Ok(Value::Bool(list.borrow().is_empty())),
            "clear" => {
                list.try_borrow_mut()
                    .map_err(|_| LugliError::runtime("Cannot modify list while it's being used"))?
                    .clear();
                Ok(Value::Null)
            }
            "reverse" => {
                list.try_borrow_mut()
                    .map_err(|_| LugliError::runtime("Cannot modify list while it's being used"))?
                    .reverse();
                Ok(Value::Null)
            }
            "filter" => {
                if arg_count != 1 {
                    return Err(LugliError::runtime("filter expects 1 argument (predicate function)"));
                }
                // Return error - higher-order functions need bytecode access
                Err(LugliError::runtime("filter requires bytecode context (use call_list_method_with_bytecode)"))
            }
            "map" | "map!" => {
                if arg_count != 1 {
                    return Err(LugliError::runtime("map expects 1 argument (mapper function)"));
                }
                // Return error - higher-order functions need bytecode access
                Err(LugliError::runtime("map requires bytecode context (use call_list_method_with_bytecode)"))
            }
            _ => Err(LugliError::runtime(format!("List has no method '{}'", method))),
        }
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
            Value::NativeFunction { callback, arity, .. } => {
                if args.len() != *arity {
                    return Err(LugliError::runtime(format!(
                        "Function expects {} arguments, got {}",
                        arity, args.len()
                    )));
                }
                callback(args)
            }
            Value::Function { name, params, body_start } | Value::Closure { name, params, body_start, .. } => {
                if args.len() != params.len() {
                    return Err(LugliError::runtime(format!(
                        "Function expects {} arguments, got {}",
                        params.len(), args.len()
                    )));
                }

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

                // Execute function body and handle result/errors
                let execution_result = self.execute_function_until_return(bytecode, saved_call_stack_len);

                // Always restore IP, even on error
                self.ip = saved_ip;

                // Handle execution result
                match execution_result {
                    Ok(()) => {
                        // Function returned successfully, get return value
                        if self.stack.len() > saved_stack_len {
                            Ok(self.stack.pop()
                                .expect("BUG: Stack should have return value after len check"))
                        } else {
                            // Stack underflow - function didn't leave return value
                            Err(LugliError::runtime(format!(
                                "Function '{}' returned without value on stack",
                                name
                            )))
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
                return Err(LugliError::runtime(
                    "Function execution ran past end of bytecode"
                ));
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

    fn call_dict_method(&mut self, method: &str, dict: Rc<RefCell<HashMap<String, Value>>>, arg_count: usize) -> Result<Value, LugliError> {
        let args = self.get_method_args(arg_count)?;

        match method {
            "len" => Ok(Value::Number(dict.borrow().len() as f64)),
            "keys" => {
                let keys: Vec<Value> = dict.borrow()
                    .keys()
                    .map(|k| Value::String(k.clone()))
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(keys))))
            }
            "values" => {
                let values: Vec<Value> = dict.borrow()
                    .values()
                    .cloned()
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(values))))
            }
            "get" => {
                if !(1..=2).contains(&arg_count) {
                    return Err(LugliError::runtime("get expects 1 or 2 arguments"));
                }
                match &args[0] {
                    Value::String(key) => {
                        let default = if arg_count == 2 {
                            args[1].clone()
                        } else {
                            Value::Null
                        };
                        Ok(dict.borrow().get(key).cloned().unwrap_or(default))
                    }
                    _ => Err(LugliError::runtime("Dictionary key must be a string")),
                }
            }
            "contains" => {
                if arg_count != 1 {
                    return Err(LugliError::runtime("contains expects 1 argument"));
                }
                match &args[0] {
                    Value::String(key) => {
                        Ok(Value::Bool(dict.borrow().contains_key(key)))
                    }
                    _ => Err(LugliError::runtime("Dictionary key must be a string")),
                }
            }
            "clear" => {
                dict.try_borrow_mut()
                    .map_err(|_| LugliError::runtime("Cannot modify dict while it's being used"))?
                    .clear();
                Ok(Value::Null)
            }
            _ => Err(LugliError::runtime(format!("Dict has no method '{}'", method))),
        }
    }

    pub fn run(&mut self, bytecode: &Bytecode) -> Result<Value, LugliError> {
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
            return Err(LugliError::runtime(format!(
                "Stack overflow: exceeded maximum stack size of {} values",
                MAX_STACK_SIZE
            )));
        }

        if self.call_stack.len() > MAX_CALL_DEPTH {
            return Err(LugliError::runtime(format!(
                "Maximum recursion depth exceeded: {} nested calls",
                MAX_CALL_DEPTH
            )));
        }

        // Debug tracing
        if self.debug.trace_execution {
            eprintln!("[TRACE] IP:{:04} | {:?}", self.ip, instruction);
            self.debug.instruction_count += 1;
        }

        if self.debug.trace_stack && !self.stack.is_empty() {
            eprintln!("[STACK] depth:{} top:{:?}", self.stack.len(), self.stack.last());
        }

        let inst_start = if self.debug.trace_execution {
            Some(Instant::now())
        } else {
            None
        };

        match instruction {
            Instruction::Constant(index) => {
                let value = self.get_constant(bytecode, *index)?.clone();
                self.stack.push(value);
            }
            Instruction::Pop => {
                self.pop()?;
            }
            Instruction::Dup => {
                let val = self.peek()?.clone();
                self.stack.push(val);
            }
            Instruction::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.add(&b)?);
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
                let callee = self.peek_n(0)?.clone(); // Callee is at the top

                if self.debug.trace_calls {
                    let func_name = match &callee {
                        Value::NativeFunction { name, .. } => format!("<native:{}>", name),
                        Value::Function { name, .. } | Value::Closure { name, .. } => name.clone(),
                        _ => "<unknown>".to_string(),
                    };
                    eprintln!("[CALL] {} with {} arguments", func_name, arg_count);
                }

                match callee {
                    Value::NativeFunction { name, callback, .. } => {
                        if self.debug.trace_calls {
                            eprintln!("[CALL] Executing native function: {}", name);
                        }
                        let args_start_index = self.stack.len() - arg_count - 1;
                        let args_end_index = self.stack.len() - 1;
                        let args = &self.stack[args_start_index..args_end_index];
                        let result = callback(args)?;
                        self.stack.truncate(self.stack.len() - arg_count - 1); // Pop args and function
                        self.stack.push(result);
                    }
                    Value::Function { name, params, body_start } => {
                        let arity = params.len();
                        if arg_count != arity {
                            return Err(LugliError::runtime(format!(
                                "Function expects {} arguments, got {}",
                                arity, arg_count
                            )));
                        }
                        // The new frame starts where the callee is, which is `len - arg_count - 1`
                        let stack_base = self.stack.len() - arg_count - 1;
                        let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base);
                        self.call_stack.push(frame);
                        self.ip = body_start;
                        return Ok(true);
                    }
                    Value::Closure { name, params, body_start, upvalues } => {
                        let arity = params.len();
                        if arg_count != arity {
                            return Err(LugliError::runtime(format!(
                                "Closure expects {} arguments, got {}",
                                arity, arg_count
                            )));
                        }
                        // The new frame starts where the callee is, which is `len - arg_count - 1`
                        let stack_base = self.stack.len() - arg_count - 1;
                        let frame = CallFrame::new_closure(name.clone(), self.ip + 1, stack_base, upvalues.clone());
                        self.call_stack.push(frame);
                        self.ip = body_start;
                        return Ok(true);
                    }
                    _ => {
                        return Err(LugliError::runtime(format!("Not callable: {}", callee.type_name())));
                    }
                }
            }
            Instruction::Return => {
                let frame = self.call_stack.pop()
                    .ok_or_else(|| LugliError::runtime("Call stack underflow"))?;
                let return_value = self.pop().unwrap_or(Value::Null);

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
                let frame = self.call_stack.last()
                    .ok_or_else(|| LugliError::runtime("Call stack is empty"))?;
                let target_index = frame.stack_base + index;
                let value = self.stack.get(target_index)
                    .ok_or_else(|| LugliError::runtime(format!(
                        "Stack underflow: attempted to load local {} at index {} (stack size: {})",
                        index, target_index, self.stack.len()
                    )))?
                    .clone();
                self.stack.push(value);
            }
            Instruction::Store(index) => {
                let value = self.pop()?;
                let frame = self.call_stack.last()
                    .ok_or_else(|| LugliError::runtime("Call stack is empty"))?;
                let target_index = frame.stack_base + index;

                // Grow stack if necessary
                while self.stack.len() <= target_index {
                    self.stack.push(Value::Null);
                }

                self.stack[target_index] = value;
            }
            Instruction::LoadUpvalue(index) => {
                let frame = self.call_stack.last()
                    .ok_or_else(|| LugliError::runtime("Call stack is empty"))?;

                if let Some(upvalues) = &frame.closure_upvalues {
                    let upvalue = upvalues.borrow().get(*index).cloned()
                        .ok_or_else(|| LugliError::runtime(format!(
                            "Upvalue index {} out of bounds (have {} upvalues)",
                            index, upvalues.borrow().len()
                        )))?;
                    self.stack.push(upvalue);
                } else {
                    return Err(LugliError::runtime("LoadUpvalue used in non-closure context"));
                }
            }
            Instruction::StoreUpvalue(index) => {
                let value = self.pop()?;
                let frame = self.call_stack.last()
                    .ok_or_else(|| LugliError::runtime("Call stack is empty"))?;

                if let Some(upvalues) = &frame.closure_upvalues {
                    let mut upvalues_mut = upvalues.try_borrow_mut()
                        .map_err(|_| LugliError::runtime("Cannot modify closure upvalues while they're being used"))?;
                    if *index >= upvalues_mut.len() {
                        return Err(LugliError::runtime(format!(
                            "Upvalue index {} out of bounds (have {} upvalues)",
                            index, upvalues_mut.len()
                        )));
                    }
                    upvalues_mut[*index] = value;
                } else {
                    return Err(LugliError::runtime("StoreUpvalue used in non-closure context"));
                }
            }
            Instruction::LoadGlobal(name_index) => {
                let var_name = self.get_constant(bytecode, *name_index)?;
                if let Value::String(name) = var_name {
                    if let Some(value) = self.globals.get(name) {
                        self.stack.push(value.clone());
                    } else {
                        // Generate helpful error with suggestions
                        if let Some(context) = self.get_source_context(bytecode) {
                            let available_names: Vec<&str> = self.globals.keys().map(|s| s.as_str()).collect();
                            let suggestion = crate::error_formatter::suggest_similar_name(name, &available_names);
                            return Err(LugliError::undefined_variable_with_context(name.clone(), context, suggestion));
                        } else {
                            return Err(LugliError::undefined_variable(name.clone()));
                        }
                    }
                } else {
                    return Err(LugliError::runtime("Variable name must be a string"));
                }
            }
            Instruction::StoreGlobal(name_index) => {
                let value = self.peek()?.clone(); // Don't pop - leave value on stack like Store does
                let var_name = self.get_constant(bytecode, *name_index)?;
                if let Value::String(name) = var_name {
                    self.globals.insert(name.clone(), value);
                } else {
                    return Err(LugliError::runtime("Variable name must be a string"));
                }
            }
            Instruction::GetProperty(name_index) => {
                let object = self.pop()?;
                let prop_name = self.get_constant(bytecode, *name_index)?;
                if let Value::String(name) = prop_name {
                    if let Value::Dict(dict_ref) = object {
                        let dict = dict_ref.borrow();
                        if let Some(value) = dict.get(name) {
                            self.stack.push(value.clone());
                        } else {
                            self.stack.push(Value::Null);
                        }
                    } else {
                        return Err(LugliError::runtime(format!("Cannot access property '{}' on a value of type {}", name, object.type_name())));
                    }
                } else {
                    return Err(LugliError::runtime("Property name must be a string"));
                }
            }
            Instruction::SetProperty(name_index) => {
                let value = self.pop()?;
                let object = self.pop()?;
                let prop_name = self.get_constant(bytecode, *name_index)?;

                if let Value::String(name) = prop_name {
                    if let Value::Dict(dict_ref) = &object {
                        dict_ref.try_borrow_mut()
                            .map_err(|_| LugliError::runtime("Cannot modify struct while it's being used"))?
                            .insert(name.clone(), value.clone());
                        // Push the value back as the expression result
                        self.stack.push(value);
                    } else {
                        return Err(LugliError::runtime(format!("Cannot set property '{}' on a value of type {}", name, object.type_name())));
                    }
                } else {
                    return Err(LugliError::runtime("Property name must be a string"));
                }
            },
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
                if let Some(Value::String(struct_type)) = dict.get("__struct_type__") {
                    // Look up struct metadata from globals
                    if let Some(Value::Dict(meta)) = self.globals.get(struct_type) {
                        let meta_ref = meta.borrow();

                        // Get default values from struct metadata
                        if let Some(Value::Dict(defaults)) = meta_ref.get("__defaults__") {
                            let defaults_ref = defaults.borrow();

                            // Apply defaults for missing fields
                            for (field_name, default_value) in defaults_ref.iter() {
                                if !dict.contains_key(field_name) {
                                    dict.insert(field_name.clone(), default_value.clone());
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
            Instruction::MakeClosure { function_index, capture_indices } => {
                // Get the base function template
                let function_value = self.get_constant(bytecode, *function_index)?;

                if let Value::Function { name, params, body_start } = function_value {
                    // Get current stack frame to calculate absolute positions
                    let frame = self.call_stack.last()
                        .ok_or_else(|| LugliError::runtime("Call stack is empty during closure creation"))?;

                    // Capture values from the stack based on local indices
                    // Convert local indices to absolute stack positions
                    let upvalues: Vec<Value> = capture_indices.iter()
                        .map(|&local_index| {
                            let absolute_index = frame.stack_base + local_index;
                            self.stack.get(absolute_index).cloned()
                                .ok_or_else(|| LugliError::runtime(format!(
                                    "Invalid upvalue capture: local {} (absolute {}) out of bounds (stack size: {})",
                                    local_index, absolute_index, self.stack.len()
                                )))
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    // Create closure with captured upvalues
                    let closure = Value::Closure {
                        name: name.clone(),
                        params: params.clone(),
                        body_start: *body_start,
                        upvalues: Rc::new(RefCell::new(upvalues)),
                    };

                    self.stack.push(closure);
                } else {
                    return Err(LugliError::runtime("MakeClosure requires a Function value"));
                }
            }
            Instruction::CallMethod(method_name_index, arg_count) => {
                let arg_count = *arg_count as usize;
                let method_name = match self.get_constant(bytecode, *method_name_index)? {
                    Value::String(name) => name.clone(),
                    _ => return Err(LugliError::runtime("Method name must be a string")),
                };

                // Get the object (it's below the arguments on the stack)
                let object_index = self.stack.len() - arg_count - 1;
                let object = &self.stack[object_index];

                // Check for struct method calls first
                if let Value::Dict(d) = object {
                    let dict_ref = d.borrow();
                    if let Some(Value::String(struct_type)) = dict_ref.get("__struct_type__") {
                        // Clone the struct type name before dropping the borrow
                        let struct_type = struct_type.clone();
                        drop(dict_ref); // Release the borrow before method call

                        // This is a struct instance - look up the method as StructType_methodName
                        let struct_method_name = format!("{}_{}", struct_type, method_name);

                        // Look up the global function
                        if let Some(func) = self.globals.get(&struct_method_name) {
                            match func {
                                Value::Function { name, params, body_start } | Value::Closure { name, params, body_start, .. } => {
                                    // Check arity - should be args + 1 for self parameter
                                    let arity = params.len();
                                    if arg_count + 1 != arity {
                                        return Err(LugliError::runtime(format!(
                                            "Method expects {} arguments (including self), got {}",
                                            arity, arg_count + 1
                                        )));
                                    }

                                    // Set up call frame and invoke function
                                    let stack_base = self.stack.len() - arg_count - 1; // Include the object
                                    let frame = CallFrame::new(name.clone(), self.ip + 1, stack_base);
                                    self.call_stack.push(frame);
                                    self.ip = *body_start;
                                    return Ok(true); // Function call will handle stack management
                                }
                                Value::NativeFunction { callback, .. } => {
                                    // For native functions, collect args including self
                                    let args_start = self.stack.len() - arg_count - 1;
                                    let args = &self.stack[args_start..];
                                    let result = callback(args)?;

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
                            return Err(LugliError::runtime(format!(
                                "Struct '{}' has no method '{}'",
                                struct_type,
                                method_name
                            )));
                        }
                    }
                }

                // Special handling for higher-order list methods that need bytecode access
                if let Value::List(l) = object
                    && (method_name == "filter" || method_name == "map" || method_name == "map!") && arg_count == 1 {
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

                // Dispatch based on object type for built-in types
                let result = match object {
                    Value::String(s) => {
                        self.call_string_method(&method_name, s.clone(), arg_count)?
                    }
                    Value::List(l) => {
                        self.call_list_method(&method_name, l.clone(), arg_count)?
                    }
                    Value::Dict(d) => {
                        // Regular dict method call (not a struct)
                        self.call_dict_method(&method_name, d.clone(), arg_count)?
                    }
                    _ => {
                        return Err(LugliError::runtime(format!(
                            "Object of type {} has no method '{}'",
                            object.type_name(),
                            method_name
                        )));
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
                                return Err(LugliError::runtime(format!(
                                    "Index {} out of range for list of length {}",
                                    idx_i64, len
                                )));
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
                    (Value::String(s), Value::Number(idx)) => {
                        let chars: Vec<char> = s.chars().collect();
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
                                return Err(LugliError::runtime(format!(
                                    "Index {} out of range for string of length {}",
                                    idx_i64, len
                                )));
                            }
                            idx_i64 as usize
                        };

                        self.stack.push(Value::String(chars[actual_idx].to_string()));
                    }
                    _ => {
                        return Err(LugliError::runtime(format!(
                            "Cannot index {} with {}",
                            object.type_name(),
                            index.type_name()
                        )));
                    }
                }
            }
            Instruction::SetIndex => {
                let value = self.pop()?;
                let index = self.pop()?;
                let object = self.pop()?;

                match (&object, &index) {
                    (Value::List(list), Value::Number(idx)) => {
                        let mut list_ref = list.try_borrow_mut()
                            .map_err(|_| LugliError::runtime("Cannot modify list while it's being used"))?;
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
                                return Err(LugliError::runtime(format!(
                                    "Index {} out of range for list assignment (length {})",
                                    idx_i64, len
                                )));
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
                    _ => {
                        return Err(LugliError::runtime(format!(
                            "Cannot set index on {} with {}",
                            object.type_name(),
                            index.type_name()
                        )));
                    }
                }
            }
            Instruction::ToString => {
                let value = self.pop()?;
                let string_value = match &value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => {
                        if n.is_nan() {
                            "NaN".to_string()
                        } else if n.is_infinite() {
                            if n.is_sign_positive() {
                                "Infinity".to_string()
                            } else {
                                "-Infinity".to_string()
                            }
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
                        let elements: Vec<String> = list_ref.iter()
                            .map(|v| match v {
                                Value::String(s) => format!("\"{}\"", s),
                                _ => v.to_string(),
                            })
                            .collect();
                        format!("[{}]", elements.join(", "))
                    }
                    Value::Dict(dict) => {
                        let dict_ref = dict.borrow();
                        let pairs: Vec<String> = dict_ref.iter()
                            .map(|(k, v)| match v {
                                Value::String(s) => format!("\"{}\": \"{}\"", k, s),
                                _ => format!("\"{}\": {}", k, v),
                            })
                            .collect();
                        format!("{{{}}}", pairs.join(", "))
                    }
                    Value::Function { name, .. } | Value::Closure { name, .. } => format!("<function {}>", name),
                    Value::NativeFunction { name, .. } => format!("<native function {}>", name),
                    Value::StructInstance { name, .. } => format!("<{} instance>", name),
                    Value::DateTime(dt) => dt.to_string(),
                };
                self.stack.push(Value::String(string_value));
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
    fn default() -> Self {
        Self::new()
    }
}