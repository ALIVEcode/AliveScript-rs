use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    rc::Rc,
    str::FromStr,
    sync::{Arc, RwLock},
};

use pest::{
    Parser as PestParser, Span,
    error::{Error as PestError, ErrorVariant as PestErrorVariant},
    iterators::{Pair, Pairs},
};

use crate::{
    AlivescriptParser, Rule,
    compiler::{
        bytecode::{
            BinCompcode, BinLogiccode, BinOpcode, Instructions, JUMP_OFFSET, Opcode, UnaryOpcode,
            instructions_to_string_debug,
        },
        err::{CompilationError, CompilationErrorKind},
        obj::{UpvalueSpec, Value},
        parser::{PRATT_EXPR_PARSER, PRATT_TYPE_PARSER},
        value::{
            ASFieldInfo, ASFunction, ASModule, ASStructure, ArcClosureProto, ArcStructure,
            ClosureProto, FieldProto, ModuleProto, StructType, Type, TypeSpec,
        },
    },
    utils::Invert,
};

mod bitmasks;
pub(crate) mod bytecode;
mod err;
pub mod obj;
mod parser;
mod serializer;
mod utils;
pub mod value;

macro_rules! unpack {
    ($pat:pat = $expr:expr) => {
        let $pat = $expr else { unreachable!() };
    };
}

#[derive(Debug, Clone)]
pub struct Local {
    name: String, // Identifier text, needed for shadowing and error reporting.
    depth: i32,   // Scope depth: -1 = declared but not initialized,
    // 0+ = active scope levels.
    is_captured: bool, // Set to true if an inner function captures this variable.
    var_type: TypeSpec,
    is_const: bool,
    is_inner: bool,
}

#[derive(Debug)]
pub struct LocalType {
    name: String,
    spec: TypeSpec,
    depth: i32,
}

#[derive(Debug)]
pub struct Compiler<'a> {
    pub source_name: String,

    pub input: &'a str,

    // Current function being built
    pub function: Rc<RefCell<ASFunction>>,
    pub code: Instructions,

    // Compiler nesting
    pub parent: Option<Rc<RefCell<Compiler<'a>>>>,

    // Scope & locals
    pub locals: Vec<Local>,
    pub local_types: Vec<LocalType>,
    pub scope_depth: usize,
    pub return_type: TypeSpec,

    // Captured variables
    pub upvalues: Vec<UpvalueSpec>,

    // Errors
    pub had_error: bool,
    pub panic_mode: bool,

    pub jump_stack: Vec<usize>,     // offsets to patch later
    pub continue_stack: Vec<usize>, // offsets to patch later
    pub break_stack: Vec<usize>,    // offsets to patch later
}

impl<'a> Compiler<'a> {
    pub fn new(input: &'a str, source_name: String) -> Self {
        Self {
            input,
            source_name,
            function: Rc::new(RefCell::new(ASFunction::new_anonymous(0))),
            code: Instructions::new(),
            parent: None,
            locals: vec![],
            local_types: vec![],
            scope_depth: 0,
            upvalues: vec![],
            had_error: false,
            panic_mode: false,
            jump_stack: vec![],
            continue_stack: vec![],
            break_stack: vec![],
            return_type: Type::Tout.into(),
        }
    }

    fn new_closure(
        input: &'a str,
        name: Option<String>,
        parent: Rc<RefCell<Compiler<'a>>>,
        nb_params: usize,
        return_type: TypeSpec,
    ) -> Self {
        Self {
            input,
            source_name: { parent.borrow().source_name.clone() },
            function: Rc::new(RefCell::new(ASFunction::new(name, nb_params))),
            code: Instructions::new(),
            parent: Some(parent),
            locals: vec![],
            local_types: vec![],
            scope_depth: 0,
            upvalues: vec![],
            had_error: false,
            panic_mode: false,
            jump_stack: vec![],
            continue_stack: vec![],
            break_stack: vec![],
            return_type,
        }
    }

    pub fn parse_and_compile_to_module(self) -> Result<ModuleProto, CompilationError> {
        let stmts = AlivescriptParser::parse(Rule::script, self.input)?;

        self.compile_to_module(stmts)
    }

    pub fn compile_to_module(
        self,
        pairs: Pairs<'a, Rule>,
    ) -> Result<ModuleProto, CompilationError> {
        let source = self.source_name.clone();

        let mut rc_self = Rc::new(RefCell::new(self));

        rc_self
            .build_ast_stmts(pairs)
            .map_err(|err| err.set_source_if_none(source.clone()))?;

        rc_self.borrow_mut().code.pop_if_op_is(Opcode::Pop);

        rc_self.borrow_mut().code.emit_return();

        rc_self.borrow_mut().finish();

        let closure = ClosureProto::new(Arc::new(rc_self.borrow().function.borrow().clone()));

        let mut exported = HashMap::new();
        for local in rc_self.borrow().locals.iter() {
            if local.is_inner {
                continue;
            }
            exported.insert(
                local.name.clone(),
                FieldProto {
                    value_idx: rc_self
                        .borrow()
                        .resolve_local(&local.name, true)
                        .expect("Valid local variable")
                        .expect("Valid local variable")
                        .0,
                    is_const: local.is_const,
                    field_type: local.var_type.clone().as_base_type().map_err(
                        |compilation_error_kind| {
                            compilation_error_kind.to_error(Span::new("", 0, 0).unwrap())
                        },
                    )?,
                },
            );
        }

        Ok(ModuleProto {
            name: source,
            load_fn: closure,
            exported_members: exported,
        })
    }

    pub fn parse_and_compile(self) -> Result<ClosureProto, CompilationError> {
        let stmts = AlivescriptParser::parse(Rule::script, self.input)?;

        self.compile(stmts)
    }

    pub fn parse_and_compile_debug(self) -> Result<ClosureProto, CompilationError> {
        let stmts = AlivescriptParser::parse(Rule::script, self.input)?;

        println!("{:#?}", &stmts);

        self.compile_debug(stmts)
    }

    pub fn compile(self, pairs: Pairs<'a, Rule>) -> Result<ClosureProto, CompilationError> {
        let source = self.source_name.clone();

        let mut rc_self = Rc::new(RefCell::new(self));

        rc_self
            .build_ast_stmts(pairs)
            .map_err(|err| err.set_source_if_none(source))?;

        rc_self.borrow_mut().code.pop_if_op_is(Opcode::Pop);

        rc_self.borrow_mut().code.emit_return();

        rc_self.borrow_mut().finish();

        let x = ClosureProto::new(Arc::new(rc_self.borrow().function.borrow().clone()));

        Ok(x)
    }

    fn compile_lambda_expr(self, pairs: Pairs<'a, Rule>) -> Result<ClosureProto, CompilationError> {
        let source = self.source_name.clone();

        let mut rc_self = Rc::new(RefCell::new(self));

        rc_self
            .parse_expr(pairs)
            .map_err(|err| err.set_source_if_none(source))?;

        rc_self.borrow_mut().code.emit_return();

        rc_self.borrow_mut().finish();

        // println!("CLOSURE:");
        // println!(
        //     "| ----INSTRUCTIONS----\n| {}",
        //     instructions_to_string_debug(
        //         &rc_self.borrow().function.borrow().code,
        //         Rc::clone(&rc_self)
        //     )
        //     .join("\n| ")
        // );
        // println!(
        //     "| ----LOCALS----\n| {}",
        //     format!("{:#?}", rc_self.borrow().locals).replace("\n", "\n| ")
        // );
        // println!(
        //     "| ----CONSTANTS----\n| {}",
        //     format!("{:#?}", rc_self.borrow().function.borrow().constants).replace("\n", "\n| ")
        // );
        // println!(
        //     "| ----UPVALUES----\n| {}",
        //     format!("{:#?}", rc_self.borrow().function.borrow().upvalue_specs).replace("\n", "\n| ")
        // );
        // println!();

        let x = ClosureProto::new(Arc::new(rc_self.borrow().function.borrow().clone()));

        Ok(x)
    }

    fn compile_empty(self) -> Result<ClosureProto, CompilationError> {
        let source = self.source_name.clone();

        let mut rc_self = Rc::new(RefCell::new(self));

        rc_self.borrow_mut().code.emit_return();

        rc_self.borrow_mut().finish();

        let x = ClosureProto::new(Arc::new(rc_self.borrow().function.borrow().clone()));

        Ok(x)
    }

    pub fn compile_debug(self, pairs: Pairs<'a, Rule>) -> Result<ClosureProto, CompilationError> {
        let source = self.source_name.clone();

        let mut rc_self = Rc::new(RefCell::new(self));

        rc_self
            .build_ast_stmts(pairs)
            .map_err(|err| err.set_source_if_none(source))?;

        rc_self.borrow_mut().code.pop_if_op_is(Opcode::Pop);

        rc_self.borrow_mut().code.emit_return();

        rc_self.borrow_mut().finish();

        println!("FUNCTION:");
        println!(
            "| ----INSTRUCTIONS----\n| {}",
            instructions_to_string_debug(
                &rc_self.borrow().function.borrow().code,
                Rc::clone(&rc_self)
            )
            .join("\n| ")
        );
        println!(
            "| ----LOCALS----\n| {}",
            format!("{:#?}", rc_self.borrow().locals).replace("\n", "\n| ")
        );
        println!(
            "| ----CONSTANTS----\n| {}",
            format!("{:#?}", rc_self.borrow().function.borrow().constants).replace("\n", "\n| ")
        );
        println!(
            "| ----UPVALUES----\n| {}",
            format!("{:#?}", rc_self.borrow().function.borrow().upvalue_specs)
                .replace("\n", "\n| ")
        );
        println!();

        let x = ClosureProto::new(Arc::new(rc_self.borrow().function.borrow().clone()));

        Ok(x)
    }

    fn finish(&mut self) {
        let code = self.code.inner().clone();
        self.function.borrow_mut().code = code;
        self.function.borrow_mut().upvalue_specs = self.upvalues.clone();
        self.function.borrow_mut().upvalue_count = self.upvalues.len();
    }

    fn get_const(&mut self, idx: usize) -> Option<Value> {
        self.function.borrow().constants.get(idx).cloned()
    }

    fn get_struct_const(&self, name: &str) -> Option<ArcStructure> {
        let f = self.function.borrow();
        let s = f.constants.iter().find(|c| match c {
            Value::Structure(s) => s.read().unwrap().name == name,
            _ => false,
        });

        if let Some(s) = s {
            let Value::Structure(s) = s else {
                unreachable!()
            };
            return Some(ArcStructure::clone(s));
        }

        // 2. Try to resolve as an UPVALUE
        if let Some(parent) = &self.parent {
            let p = parent.borrow();
            return p.get_struct_const(name);
        }

        None
    }

    fn get_or_add_const(&mut self, obj: Value) -> u16 {
        let idx = self
            .function
            .borrow()
            .constants
            .iter()
            .enumerate()
            .find(|(_, o)| **o == obj)
            .map(|(i, _)| i);
        if let Some(idx) = idx {
            return idx as u16;
        }

        let mut f = self.function.borrow_mut();
        f.constants.push(obj);
        f.constants.len() as u16 - 1
    }

    /// Like `get_or_add_const`, but emits an Opcode::Const
    fn push_const(&mut self, obj: Value) -> u16 {
        let c_idx = self.get_or_add_const(obj);
        self.code.emit_const(c_idx);

        c_idx
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn nb_local_scope_vars(&mut self) -> usize {
        self.locals
            .iter()
            .filter(|local| local.depth >= self.scope_depth as i32)
            .count()
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        // Pop locals from this scope.
        while let Some(local) = self.locals.last() {
            if local.depth <= self.scope_depth as i32 {
                break;
            }

            if local.is_captured {
                println!("emit: CLOSE_UPVALUE");
            } else {
                self.code.emit_pop();
            }

            self.locals.pop();
        }
        // Pop locals from this scope.
        while let Some(local) = self.local_types.last() {
            if local.depth <= self.scope_depth as i32 {
                break;
            }

            self.local_types.pop();
        }
    }

    // declare a local inner variable (no name)
    fn declare_inner_local(&mut self, debug_name: &str) -> u16 {
        self.locals.push(Local {
            name: format!("(({}))", debug_name),
            // inner locals are automatically initialized
            depth: self.scope_depth as i32,
            is_captured: false,
            var_type: Type::Tout.into(),
            is_const: false,
            is_inner: true,
        });
        self.locals.len() as u16 - 1
    }

    fn declare_local(&mut self, name: &str, var_type: TypeSpec, is_const: bool) -> u16 {
        self.locals.push(Local {
            name: name.to_string(),
            depth: -1, // not initialized yet
            is_captured: false,
            var_type: var_type,
            is_const,
            is_inner: false,
        });
        self.locals.len() as u16 - 1
    }

    fn mark_initialized(&mut self, index: u16) {
        let local = self.locals.get_mut(index as usize).unwrap();
        // if not initialized yet
        if local.depth == -1 {
            local.depth = self.scope_depth as i32;
        }
    }

    fn declare_local_type(&mut self, name: &str, spec: TypeSpec) -> u16 {
        self.local_types.push(LocalType {
            name: name.to_string(),
            spec,
            depth: self.scope_depth as i32,
        });
        self.local_types.len() as u16 - 1
    }

    // Helper to record an upvalue and return its index
    fn add_upvalue(&mut self, is_local: bool, index: usize) -> usize {
        let spec = if is_local {
            UpvalueSpec::Local(index)
        } else {
            UpvalueSpec::Upvalue(index)
        };

        // Check if we already have this exact upvalue recorded
        if let Some(i) = self.upvalues.iter().position(|u| *u == spec) {
            return i;
        }

        self.upvalues.push(spec);
        self.upvalues.len() - 1
    }

    fn resolve_upval(
        &mut self,
        name: &str,
    ) -> Result<Option<(usize, Local)>, CompilationErrorKind> {
        // 1. Check if we have a parent compiler
        let parent_rc = match &self.parent {
            Some(p) => Rc::clone(p),
            None => return Ok(None), // Reached the top-level script, not an upvalue
        };

        // We need mutable access to the parent's state (locals/upvalues)
        let mut parent = parent_rc.borrow_mut();

        // 2. Try to resolve as a LOCAL in the PARENT
        if let Some((local_idx, local)) = parent.resolve_local(name, true)? {
            // FOUND: It's a local in the parent (Direct Capture)

            // Mark the local in the parent as captured.
            parent.mark_captured(local_idx);

            // Record it as a new upvalue in THIS compiler.
            // We capture the stack slot index (local_idx) from the parent's frame.
            let upval_idx = self.add_upvalue(true, local_idx);

            // Return the index of the newly created upvalue in *this* function's upvalues array.
            return Ok(Some((upval_idx, local)));
        }

        // 3. Try to resolve as an UPVALUE in the PARENT (Indirect Capture)
        // Note: This is a recursive call!
        if let Some((upval_idx_in_parent, local)) = parent.resolve_upval(name)? {
            // FOUND: It's already an upvalue in the parent's closure (Inherited Upvalue)

            // Record it as a new upvalue in THIS compiler.
            // We capture the upvalue index (upval_idx_in_parent) from the parent's upvalue array.
            let upval_idx = self.add_upvalue(false, upval_idx_in_parent);

            // Return the index of the newly created upvalue in *this* function's upvalues array.
            return Ok(Some((upval_idx, local)));
        }

        // 4. Not found in the entire ancestry.
        Ok(None)
    }

    fn resolve_local(
        &self,
        name: &str,
        allow_uninit: bool,
    ) -> Result<Option<(usize, Local)>, CompilationErrorKind> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                if local.depth == -1 && !allow_uninit {
                    Err(CompilationErrorKind::generic_error(format!(
                        "Impossible de lire la variable '{}', car elle n'a pas été initialisée",
                        name
                    )))?;
                }
                return Ok(Some((i, local.clone())));
            }
        }
        Ok(None)
    }

    fn mark_captured(&mut self, index: usize) {
        self.locals[index].is_captured = true;
    }

    fn patch_jump(&mut self, jmp_stack_idx: usize) {
        let val = self.code.inner().len() - 1;
        let jump_idx = self.jump_stack[jmp_stack_idx];
        self.code
            .raw_patch(jump_idx, ((val - jump_idx) as i16 + JUMP_OFFSET) as u16);
    }

    fn patch_jump_to(&mut self, jmp_stack_idx: usize, dest: usize) {
        let jump_idx = self.jump_stack[jmp_stack_idx];
        self.code.raw_patch(
            jump_idx,
            (dest as i16 - 1 - jump_idx as i16 + JUMP_OFFSET) as u16,
        );
    }

    fn push_cond_jump(&mut self) -> usize {
        let jump_idx = self.code.inner().len() + 1;
        self.code.emit_jump_if_false(0);
        self.jump_stack.push(jump_idx);
        self.jump_stack.len() - 1
    }

    fn push_jump(&mut self) -> usize {
        let jump_idx = self.code.inner().len() + 1;
        self.code.emit_jump(0);
        self.jump_stack.push(jump_idx);
        self.jump_stack.len() - 1
    }

    fn load_var(&mut self, ident: &str) {
        // 1. Try to resolve as a LOCAL
        if let Ok(Some((local_idx, _))) = self.resolve_local(ident, false) {
            self.code.emit_get_local(local_idx as u16);
            return;
        }

        // 2. Try to resolve as an UPVALUE
        if let Ok(Some((upval_idx, _))) = self.resolve_upval(ident) {
            self.code.emit_get_upvalue(upval_idx as u16);
            return;
        }

        // 3. Load a GLOBAL by setting the variable name as a string constant
        // and emiting a LoadGlobal
        let glob_name_idx = self.get_or_add_const(Value::Texte(ident.to_string()));
        self.code.emit_get_global(glob_name_idx);
    }

    fn get_var_type(&self, ident: &str) -> Option<TypeSpec> {
        // 1. Try to resolve as a LOCAL
        for local in self.local_types.iter().rev() {
            if local.name == ident {
                return Some(local.spec.clone());
            }
        }

        // 2. Try to resolve as an UPVALUE
        if let Some(parent) = &self.parent {
            let p = parent.borrow();
            return p.get_var_type(ident);
        }

        None
    }
}

trait Parser<'a> {
    fn parse_top_expr(&mut self, primary: Pair<'a, Rule>) -> Result<usize, CompilationError>;

    fn parse_expr(
        &mut self,
        pairs: impl Iterator<Item = Pair<'a, Rule>>,
    ) -> Result<usize, CompilationError>;

    fn parse_fn_params(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError>;
    fn parse_fn(
        &mut self,
        inner: Pairs<'a, Rule>,
        fn_name: Option<String>,
    ) -> Result<(), CompilationError>;

    fn parse_methode_def(
        &mut self,
        structure: ArcStructure,
        pair: Pair<'a, Rule>,
    ) -> Result<(), CompilationError>;

    fn parse_assign_vars(
        &mut self,
        pairs: Pairs<Rule>,
        is_const: bool,
        new_local: bool,
    ) -> Result<(), CompilationError>;

    fn parse_cond_jump(
        &mut self,
        rhs_start_idx: usize,
        rhs_len: usize,
        cond: bool,
    ) -> Result<usize, CompilationError>;

    fn parse_assign(&mut self, pair: Pair<'a, Rule>) -> Result<(), CompilationError>;

    fn parse_declare(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError>;

    fn parse_lit(&mut self, pair: Pair<Rule>) -> Result<(), CompilationError>;

    fn parse_type(&mut self, pairs: Pairs<Rule>) -> Result<TypeSpec, CompilationError>;

    fn parse_if(&mut self, pair: Pair<'a, Rule>) -> Result<(), CompilationError>;

    fn build_ast_stmt(&mut self, pair: Pair<'a, Rule>) -> Result<(), CompilationError>;

    fn build_ast_stmts(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError>;
}

impl<'a> Parser<'a> for Rc<RefCell<Compiler<'a>>> {
    fn parse_top_expr(&mut self, primary: Pair<'a, Rule>) -> Result<usize, CompilationError> {
        let before = self.borrow().code.len_inner();
        match primary.as_rule() {
            Rule::AssignMember => {
                self.parse_expr(primary.into_inner())?;
            }
            Rule::List | Rule::ListExpr => {
                let mut nb_el = 0;
                for arg in primary.into_inner() {
                    self.parse_expr(arg.into_inner())?;
                    nb_el += 1;
                }
                self.borrow_mut().code.emit_new_list(nb_el);
            }

            Rule::Expr => {
                self.parse_expr(primary.into_inner())?;
            }

            Rule::Ident => {
                let ident = primary.as_str();
                self.borrow_mut().load_var(ident);
            }

            Rule::Lit => {
                self.parse_lit(primary.into_inner().next().unwrap())?;
            }

            Rule::StructInit => {
                let span = primary.as_span();
                let mut inner = primary.into_inner();

                let struct_pair = inner.next().unwrap();
                let struct_name = struct_pair.as_str();
                self.parse_top_expr(struct_pair)?;

                let Some(s_type) = self.borrow().get_var_type(struct_name) else {
                    panic!("Unknown struct {}", struct_name)
                };

                let s_c = s_type.clone();
                let Type::Struct(..) = s_type
                    .as_base_type()
                    .map_err(|compilation_error_kind| compilation_error_kind.to_error(span))?
                else {
                    return Err(CompilationErrorKind::generic_error(format!(
                        "Impossible de construire un type '{:?}'. Seule une structure peut être construite.",
                        s_c,
                    )).to_error(span));
                };
                // TODO: check the fields are the correct types

                let mut nb_fields = 0;
                while let Some(field) = inner.next() {
                    let mut field_inner = field.into_inner();
                    let field_name = field_inner.next().expect("A name for the field").as_str();
                    self.parse_expr(
                        field_inner
                            .next()
                            .expect("Value for the field")
                            .into_inner(),
                    )?;
                    self.borrow_mut()
                        .push_const(Value::Texte(field_name.to_string()));
                    nb_fields += 1;
                }

                self.borrow_mut().code.emit_new_struct(nb_fields);
            }

            Rule::FnCall => {
                let mut inner = primary.into_inner();
                // func
                self.parse_expr(inner.next().unwrap().into_inner())?;
                let mut arg_len = 0;
                // args
                for arg in inner.next().unwrap().into_inner() {
                    self.parse_expr(arg.into_inner())?;
                    arg_len += 1;
                }

                self.borrow_mut().code.emit_call(arg_len);
            }

            Rule::DebutBloc => {
                self.borrow_mut().begin_scope();

                self.build_ast_stmts(primary.into_inner())?;

                let mut comp = self.borrow_mut();

                // we prevent the cleanup of the last value because we want to return
                // it as the value of this expression
                comp.code.pop_if_op_is(Opcode::Pop);

                let nb_locals = comp.nb_local_scope_vars();

                // if we have local variables, they will get cleaned up with a series
                // of push. To save our value, we put it in the first local variable
                // of this block and we cleanup everything except that value.
                if nb_locals > 0 {
                    let first_local = (comp.locals.len() - nb_locals) as u16;
                    comp.code.emit_set_local(first_local);
                }

                comp.end_scope();

                // we prevent the cleanup of the last variable, because that stack slot
                // now holds the value of this expression
                comp.code.pop_if_op_is(Opcode::Pop);
            }

            Rule::EssayerExpr => {
                self.parse_expr(primary.into_inner())?;
            }

            Rule::FnExpr => {
                self.parse_fn(primary.into_inner(), None)?;
            }
            rule => Err(PestError::new_from_span(
                PestErrorVariant::ParsingError {
                    positives: vec![Rule::term],
                    negatives: vec![rule],
                },
                primary.as_span(),
            ))?,
        };

        Ok(self.borrow().code.len_inner() - before)
    }

    fn parse_cond_jump(
        &mut self,
        rhs_start_idx: usize,
        rhs_len: usize,
        cond: bool,
    ) -> Result<usize, CompilationError> {
        self.borrow_mut().code.set_cursor(rhs_start_idx);

        self.borrow_mut().code.emit_jump_test(rhs_len as i16, cond);

        self.borrow_mut().code.remove_cursor();

        Ok(3)
    }

    fn parse_expr(
        &mut self,
        pairs: impl Iterator<Item = Pair<'a, Rule>>,
    ) -> Result<usize, CompilationError> {
        PRATT_EXPR_PARSER
            .map_primary(|pair| Rc::clone(self).parse_top_expr(pair))
            .map_prefix(|prefix, rhs| {
                let mut nb_inst = rhs?;

                if let Ok(op) = UnaryOpcode::try_from(&prefix) {
                    match op {
                        UnaryOpcode::Negate => {
                            self.borrow_mut().code.emit_neg();
                        }
                        UnaryOpcode::Pas => {
                            self.borrow_mut().code.emit_not();
                        }
                        _ => {}
                    }
                    nb_inst += 1;
                    // Ok(Box::new(Expr::UnaryOp { expr: rhs, op }))
                    Ok(nb_inst)
                } else {
                    Err(PestError::new_from_span(
                        PestErrorVariant::ParsingError {
                            positives: vec![Rule::Not, Rule::Neg, Rule::Pos],
                            negatives: vec![prefix.as_rule()],
                        },
                        prefix.as_span(),
                    )
                    .into())
                }
            })
            .map_infix(|lhs, infix, rhs| {
                let len_lhs = lhs?;
                let len_rhs = rhs?;

                if let Ok(op) = BinOpcode::try_from(&infix) {
                    self.borrow_mut().code.emit_binop(op);
                    return Ok(len_lhs + len_rhs + 1);
                }
                if let Ok(op) = BinCompcode::try_from(&infix) {
                    self.borrow_mut().code.emit_bincomp(op);
                    return Ok(len_lhs + len_rhs + 1);
                }
                if let Ok(op) = BinLogiccode::try_from(&infix) {
                    let start_rhs = self.borrow().code.len_inner() - len_rhs;
                    match op {
                        BinLogiccode::Et => {
                            let et = Rc::clone(self).parse_cond_jump(start_rhs, len_rhs, false)?;
                            return Ok(len_lhs + len_rhs + et);
                        }
                        BinLogiccode::Ou => {
                            let ou = Rc::clone(self).parse_cond_jump(start_rhs, len_rhs, true)?;
                            return Ok(len_lhs + len_rhs + ou);
                        }
                        BinLogiccode::NonNul => todo!(),
                    }
                }
                todo!();
            })
            .map_postfix(|lhs, postfix| {
                let mut nb_inst = lhs?;

                match postfix.as_rule() {
                    Rule::AccessProp => {
                        let prop = postfix.into_inner().next().unwrap();
                        if matches!(prop.as_node_tag(), Some("prop")) {
                            let idx = self
                                .borrow_mut()
                                .get_or_add_const(Value::Texte(prop.as_str().to_string()));
                            self.borrow_mut().code.emit_get_attr(idx);
                            nb_inst += 1
                        } else {
                            Rc::clone(self).parse_expr(prop.into_inner())?;
                            self.borrow_mut().code.emit_get_item();
                            nb_inst += 1
                        }
                    }
                    _ => unreachable!(),
                }
                Ok(nb_inst)
            })
            .parse(pairs)
    }

    fn parse_fn(
        &mut self,
        inner: Pairs<'a, Rule>,
        fn_name: Option<String>,
    ) -> Result<(), CompilationError> {
        let params = inner.find_first_tagged("params").unwrap().into_inner();

        let closure = {
            let body = inner.find_first_tagged("body");
            let return_type = inner
                .find_first_tagged("return_type")
                .map(|te| self.parse_type(te.into_inner()))
                .invert()?;

            let mut c = Rc::new(RefCell::new(Compiler::new_closure(
                body.as_ref().map(|b| b.as_str()).unwrap_or(""),
                fn_name,
                Rc::clone(self),
                0,
                return_type.unwrap_or(Type::Tout.into()),
            )));
            c.parse_fn_params(params)?;

            match body {
                Some(body) => match body.as_rule() {
                    Rule::Expr => Rc::try_unwrap(c)
                        .unwrap()
                        .into_inner()
                        .compile_lambda_expr(body.into_inner())?,
                    Rule::StmtBody => Rc::try_unwrap(c)
                        .unwrap()
                        .into_inner()
                        .compile(body.into_inner())?,

                    b => unreachable!("{:?}", b),
                },
                None => Rc::try_unwrap(c).unwrap().into_inner().compile_empty()?,
            }
        };

        let idx = self.borrow_mut().get_or_add_const(Value::closure(closure));

        self.borrow_mut().code.emit_closure(idx);

        Ok(())
    }

    fn parse_fn_params(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError> {
        let mut param_indexes = Vec::with_capacity(pairs.len());

        for pair in pairs {
            let span = pair.as_span();
            let inner = pair.into_inner();
            let name = inner.find_first_tagged("p_name");

            let Some(name) = name else {
                return Err(PestError::new_from_span(
                    PestErrorVariant::ParsingError {
                        positives: vec![Rule::Ident],
                        negatives: inner.map(|p| p.as_rule()).collect(),
                    },
                    span,
                )
                .into());
            };

            let mut static_type = Type::Tout.into();
            if let Some(static_type_pair) = inner.find_first_tagged("p_type") {
                static_type = self.parse_type(static_type_pair.into_inner())?;
            }

            let var_idx = self
                .borrow_mut()
                .declare_local(name.as_str(), static_type, false);

            // TODO: the default value behavior
            // if let Some(default_value) = inner.find_first_tagged("p_default") {
            //     self.parse_expr(default_value.into_inner())?;
            //     self.borrow_mut().code.emit_set_local(var_idx);
            // }
            param_indexes.push(var_idx);
        }

        // ensures a parameter cannot reference another parameter
        for idx in param_indexes {
            self.borrow_mut().mark_initialized(idx);
        }

        Ok(())
    }

    fn parse_methode_def(
        &mut self,
        structure: ArcStructure,
        pair: Pair<'a, Rule>,
    ) -> Result<(), CompilationError> {
        let mut inner = pair.into_inner();

        let name = inner
            .find_first_tagged("name")
            .map(|node| node.as_str().to_string())
            .unwrap();

        let mut is_static_method = true;

        let mut params = inner.find_first_tagged("params").unwrap().into_inner();

        let closure = {
            let return_type = inner
                .find_first_tagged("return_type")
                .map(|te| self.parse_type(te.into_inner()))
                .invert()?;

            let body = inner
                .find(|node| node.as_rule() == Rule::MethodeBody)
                .unwrap()
                .into_inner()
                .next()
                .unwrap();

            let mut c = Rc::new(RefCell::new(Compiler::new_closure(
                body.as_str(),
                Some(name.clone()),
                Rc::clone(self),
                0,
                return_type.unwrap_or(Type::Tout.into()),
            )));

            if params
                .peek()
                .is_some_and(|first| matches!(first.as_node_tag(), Some("inst_param")))
            {
                // consume the inst param
                params.next();
                let inst_idx = c
                    .borrow_mut()
                    .declare_local("inst", Type::Tout.into(), true);
                c.borrow_mut().mark_initialized(inst_idx);

                is_static_method = false;
            }

            c.parse_fn_params(params)?;

            let inner_body = match body.as_rule() {
                Rule::Expr => body.into_inner(),
                Rule::StmtBody => body.into_inner(),
                _ => unreachable!(),
            };

            Rc::try_unwrap(c)
                .unwrap()
                .into_inner()
                .compile(inner_body)?
        };

        if is_static_method {
            structure
                .write()
                .unwrap()
                .struct_methods
                .insert(name, closure.into());
        } else {
            structure
                .write()
                .unwrap()
                .inst_methods
                .insert(name, closure.into());
        }

        Ok(())
    }

    fn parse_assign_vars(
        &mut self,
        pairs: Pairs<Rule>,
        is_const: bool,
        new_local: bool,
    ) -> Result<(), CompilationError> {
        let last_pair_idx = pairs.len() - 1;

        for (i, pair) in pairs.enumerate() {
            if i != last_pair_idx {
                self.borrow_mut().code.emit_dup();
            }
            self.borrow_mut().push_const(Value::Entier(i as i64));
            self.borrow_mut().code.emit_get_item();

            match pair.as_rule() {
                Rule::DeclIdent => {
                    let span = pair.as_span();
                    let decl_def = pair.into_inner();
                    let name = decl_def.find_first_tagged("var_name").unwrap().as_str();
                    let mut new_local = new_local;

                    let mut static_type = Type::Tout.into();
                    if let Some(pair_type) = decl_def.find_first_tagged("var_type") {
                        static_type = self.parse_type(pair_type.into_inner())?;
                        new_local = true;
                    }

                    let mut compiler = self.borrow_mut();

                    if !new_local {
                        // 1. Try to resolve as a LOCAL
                        if let Ok(Some((local_idx, local))) = compiler.resolve_local(name, false) {
                            if local.is_const {
                                return Err(
                                    CompilationErrorKind::assign_to_const(name).to_error(span)
                                );
                            }
                            compiler.mark_initialized(local_idx as u16);
                            compiler.code.emit_set_local(local_idx as u16);
                            continue;
                        }

                        // 2. Try to resolve as an UPVALUE
                        if let Ok(Some((upval_idx, local))) = compiler.resolve_upval(name) {
                            if local.is_const {
                                return Err(
                                    CompilationErrorKind::assign_to_const(name).to_error(span)
                                );
                            }
                            compiler.code.emit_set_upvalue(upval_idx as u16);
                            continue;
                        }
                    }

                    // 3. It defines a new local variable
                    let local_idx = compiler.declare_local(name, static_type, is_const);

                    compiler.mark_initialized(local_idx);
                    compiler.code.emit_set_local(local_idx);
                }
                Rule::DeclIdentList => {
                    self.parse_assign_vars(pair.into_inner(), is_const, new_local);
                }
                _ => unreachable!("{:#?}", pair),
            }
        }
        Ok(())
    }

    fn parse_assign(&mut self, pair: Pair<'a, Rule>) -> Result<(), CompilationError> {
        let span = pair.as_span();
        let pairs = pair.into_inner();

        let mut static_type = Type::Tout.into();
        let mut op = None;
        let mut new_local = false;
        let mut is_const = false;
        // let mut var_list = None;

        if let Some(p) = pairs.peek().and_then(|p| {
            if matches!(p.as_rule(), Rule::Const | Rule::Var) {
                Some(p)
            } else {
                None
            }
        }) {
            new_local = true;
            is_const = p.as_rule() == Rule::Const;
        }

        let var = pairs
            .find_first_tagged("var")
            .expect("One or more variables in the assign");

        if let Some(op_pair) = pairs.find_first_tagged("op") {
            let op_pair = op_pair.into_inner().next().unwrap();
            op = Some(BinOpcode::try_from(&op_pair).unwrap());
            // if we have an assign op, we push the current value of the var
            self.parse_top_expr(var.clone())?;
        }

        if let Some(value) = pairs.find_first_tagged("a_value") {
            self.parse_expr(value.into_inner())?;
        }

        // if we have an assign op, we already pushed the value of the var and
        // the value of the expression, the only thing that remains is adding
        // the bin op
        if let Some(op) = op {
            self.borrow_mut().code.emit_binop(op);
        }

        match var.as_rule() {
            Rule::AssignMember => {
                if new_local {
                    let span = var.as_span();
                    return Err(CompilationErrorKind::generic_error(
                        "Impossible de déclarer un membre",
                    )
                    .to_error(span));
                }

                let mut inner = var.into_inner();
                let last = inner.next_back().unwrap().into_inner().next().unwrap();

                // everything until the last
                self.parse_expr(inner)?;

                if matches!(last.as_node_tag(), Some("prop")) {
                    let const_idx = self
                        .borrow_mut()
                        .get_or_add_const(Value::Texte(last.as_str().to_string()));
                    self.borrow_mut().code.emit_set_field(const_idx);
                } else {
                    self.parse_top_expr(last)?;
                    self.borrow_mut().code.emit_set_item();
                }
            }
            Rule::Ident | Rule::DeclIdent => {
                let name = match var.as_rule() {
                    Rule::Ident => var.as_str(),
                    Rule::DeclIdent => {
                        let decl_def = var.into_inner();
                        let name = decl_def.find_first_tagged("var_name").unwrap().as_str();
                        if let Some(pair_type) = decl_def.find_first_tagged("var_type") {
                            static_type = self.parse_type(pair_type.into_inner())?;
                            new_local = true;
                        }
                        name
                    }
                    _ => unreachable!(),
                };
                let mut compiler = self.borrow_mut();

                if !new_local {
                    // 1. Try to resolve as a LOCAL
                    if let Ok(Some((local_idx, local))) = compiler.resolve_local(name, false) {
                        if local.is_const {
                            return Err(CompilationErrorKind::assign_to_const(name).to_error(span));
                        }
                        compiler.mark_initialized(local_idx as u16);
                        compiler.code.emit_set_local(local_idx as u16);
                        return Ok(());
                    }

                    // 2. Try to resolve as an UPVALUE
                    if let Ok(Some((upval_idx, local))) = compiler.resolve_upval(name) {
                        if local.is_const {
                            return Err(CompilationErrorKind::assign_to_const(name).to_error(span));
                        }
                        compiler.code.emit_set_upvalue(upval_idx as u16);
                        return Ok(());
                    }
                }

                // 3. It defines a new local variable
                let local_idx = compiler.declare_local(name, static_type, is_const);

                compiler.mark_initialized(local_idx);
                compiler.code.emit_set_local(local_idx);
            }
            Rule::MultiDeclIdent | Rule::DeclIdentList => {
                self.parse_assign_vars(var.into_inner(), is_const, new_local)?
            }
            _ => panic!("{:#?}", var),
        }

        Ok(())
    }

    fn parse_declare(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError> {
        let mut name = None;
        let mut static_type = Type::Tout.into();
        let mut is_const = false;

        if let Some(value) = pairs.find_first_tagged("a_value") {
            self.parse_expr(value.into_inner())?;
        }

        let var = pairs
            .find_first_tagged("var")
            .expect("One or more variables in the assign");

        match var.as_rule() {
            Rule::Ident => {
                name = Some(var.as_str());
            }
            Rule::AssignMember => {
                let span = var.as_span();
                return Err(CompilationErrorKind::generic_error(
                    "Impossible de déclarer un membre",
                )
                .to_error(span));
            }
            Rule::DeclIdent => {
                let decl_def = var.into_inner();
                name = Some(decl_def.find_first_tagged("var_name").unwrap().as_str());
                if let Some(pair_type) = decl_def.find_first_tagged("var_type") {
                    static_type = self.parse_type(pair_type.into_inner())?;
                }
            }
            Rule::MultiDeclIdent | Rule::DeclIdentList => {
                return self.parse_assign_vars(var.into_inner(), is_const, true);
            }
            _ => panic!("{:#?}", var),
        }

        let local_idx = self
            .borrow_mut()
            .declare_local(name.unwrap(), static_type, is_const);

        let mut compiler = self.borrow_mut();

        compiler.mark_initialized(local_idx);

        compiler.code.emit_set_local(local_idx);

        Ok(())
    }

    fn parse_lit(&mut self, pair: Pair<Rule>) -> Result<(), CompilationError> {
        let obj = match pair.as_rule() {
            Rule::Integer => Value::Entier(pair.as_str().parse::<i64>().unwrap()),
            Rule::Decimal => Value::Decimal(pair.as_str().parse::<f64>().unwrap()),
            Rule::Bool => Value::Booleen(pair.as_str() == "vrai"),
            Rule::Null => Value::Nul,
            Rule::Text => {
                let slice = pair.as_str();
                let s: String = slice[1..slice.len() - 1].parse().unwrap();
                Value::Texte(
                    s.replace(r"\n", "\n")
                        .replace(r"\t", "\t")
                        .replace(r"\r", "\r")
                        .to_owned(),
                )
            }
            rule => Err(PestError::new_from_span(
                PestErrorVariant::ParsingError {
                    positives: vec![Rule::Lit],
                    negatives: vec![rule],
                },
                pair.as_span(),
            ))?,
        };

        let mut compiler = self.borrow_mut();

        let idx = compiler.get_or_add_const(obj);

        compiler.code.emit_const(idx);

        Ok(())
    }

    fn parse_type(&mut self, pairs: Pairs<Rule>) -> Result<TypeSpec, CompilationError> {
        PRATT_TYPE_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::TypeExpr => Rc::clone(self).parse_type(primary.into_inner()),
                Rule::Ident => Ok(Type::from_str(primary.as_str())
                    .map_err(|_| {
                        PestError::new_from_span(
                            PestErrorVariant::CustomError {
                                message: format!("Unknown type {}", primary.as_str()),
                            },
                            primary.as_span(),
                        )
                    })?
                    .into()),
                // Rule::Lit => Ok(Box::new(Type::Lit(self.parse_lit(
                //     primary.into_inner().next().unwrap(),
                // )?))),
                rule => Err(PestError::new_from_span(
                    PestErrorVariant::ParsingError {
                        positives: vec![Rule::typeTerm],
                        negatives: vec![rule],
                    },
                    primary.as_span(),
                )
                .into()),
            })
            // .map_infix(|lhs, infix, rhs| todo!())
            .map_postfix(|lhs, postfix| match postfix.as_rule() {
                Rule::TypeArgs => {
                    let mut type_args = vec![];
                    let span = postfix.as_span();
                    for arg in postfix.into_inner() {
                        type_args.push(Rc::clone(self).parse_type(arg.into_inner())?);
                    }
                    Ok(lhs?
                        .compute(type_args)
                        .map_err(|err| {
                            PestError::new_from_span(
                                PestErrorVariant::CustomError {
                                    message: format!("{}", err.to_string()),
                                },
                                span,
                            )
                        })?
                        .into())
                }
                _ => todo!(),
            })
            .parse(pairs)
    }

    fn parse_if(&mut self, pair: Pair<'a, Rule>) -> Result<(), CompilationError> {
        let inner = pair.clone().into_inner();

        // cond
        self.parse_expr(
            inner
                .clone()
                .find(|p| matches!(p.as_node_tag(), Some("cond")))
                .unwrap()
                .into_inner(),
        )?;

        let if_not_cond_jmp = self.borrow_mut().push_cond_jump();

        // then br
        self.build_ast_stmts(
            inner
                .clone()
                .find(|p| matches!(p.as_node_tag(), Some("body")))
                .unwrap()
                .into_inner(),
        )?;

        let mut to_end_jmps = vec![self.borrow_mut().push_jump()];
        self.borrow_mut().patch_jump(if_not_cond_jmp);

        let mut curr_br = pair;

        loop {
            match curr_br.as_rule() {
                Rule::SiStmt => {
                    if let Some(next_br) = curr_br
                        .into_inner()
                        .find(|p| matches!(p.as_rule(), Rule::sinonSiBr | Rule::sinonBr))
                    {
                        curr_br = next_br;
                    } else {
                        break;
                    }
                }
                Rule::sinonSiBr => {
                    let mut inner_elif = curr_br.into_inner();
                    // cond
                    self.parse_expr(
                        inner_elif
                            .find(|p| matches!(p.as_node_tag(), Some("cond")))
                            .unwrap()
                            .into_inner(),
                    )?;
                    let elif_not_cond_jmp = self.borrow_mut().push_cond_jump();
                    // then br
                    self.build_ast_stmts(
                        inner_elif
                            .find(|p| matches!(p.as_node_tag(), Some("body")))
                            .unwrap()
                            .into_inner(),
                    )?;

                    to_end_jmps.push(self.borrow_mut().push_jump());
                    self.borrow_mut().patch_jump(elif_not_cond_jmp);

                    if let Some(next_br) =
                        inner_elif.find(|p| matches!(p.as_rule(), Rule::sinonSiBr | Rule::sinonBr))
                    {
                        curr_br = next_br;
                    } else {
                        break;
                    }
                }
                Rule::sinonBr => {
                    let body = curr_br.into_inner().next().unwrap();
                    if body.as_rule() == Rule::StmtBody {
                        self.build_ast_stmts(body.into_inner())?;
                    } else {
                        self.build_ast_stmt(body)?;
                    }
                    break;
                }
                _ => {}
            }
        }

        for to_end_jmp in to_end_jmps {
            self.borrow_mut().patch_jump(to_end_jmp);
        }

        Ok(())
    }

    fn build_ast_stmt(&mut self, pair: Pair<'a, Rule>) -> Result<(), CompilationError> {
        match pair.as_rule() {
            Rule::LireStmt => {
                let mut inner = pair.into_inner();
                // skip "lire"
                inner.next();

                let mut cast = false;
                let mut with_msg = false;

                if let Some(cast_fn) = inner.find_first_tagged("cast") {
                    // skip "Callable"
                    inner.next();
                    // skip "In"
                    inner.next();

                    cast = true;
                    self.parse_expr(cast_fn.into_inner())?;
                }

                let var = inner.next().unwrap();

                if let Some(msg) = inner.find_first_tagged("msg") {
                    with_msg = true;
                    self.parse_top_expr(msg)?;
                }

                let mut jump_to_sinon = 0;
                if cast {
                    let jump_idx = self.borrow_mut().code.inner().len() + 1;
                    self.borrow_mut().code.emit_read_call(0, with_msg);
                    self.borrow_mut().jump_stack.push(jump_idx);
                    jump_to_sinon = self.borrow_mut().jump_stack.len() - 1;
                } else {
                    self.borrow_mut().code.emit_read(with_msg);
                }

                self.parse_assign(var)?;

                if cast {
                    let skip_sinon_jmp = self.borrow_mut().push_jump();
                    self.borrow_mut().patch_jump(jump_to_sinon);
                    if let Some(lire_sinon) = inner.find(|token| token.as_rule() == Rule::LireSinon)
                    {
                        if let Some(lire_sinon_body) =
                            lire_sinon.into_inner().find_first_tagged("body")
                        {
                            self.build_ast_stmts(lire_sinon_body.into_inner());
                        }
                    }
                    self.borrow_mut().patch_jump(skip_sinon_jmp);
                }
            }
            Rule::UtiliserStmt => {
                let inner = pair.into_inner();
                let module_name = inner
                    .clone()
                    .find_first_tagged("module")
                    .map(|m| m.as_str().replace(".", "/"))
                    .or_else(|| {
                        inner
                            .find_first_tagged("path")
                            .map(|p| p.as_str().to_string())
                    })
                    .unwrap();

                let alias = inner
                    .clone()
                    .find(|node| node.as_rule() == Rule::ModuleAlias)
                    .map(|alias| alias.into_inner().next().unwrap().as_str().to_string());
                let vars = inner
                    .clone()
                    .find(|node| node.as_rule() == Rule::UtiliserMembers)
                    .map(|node| {
                        node.into_inner()
                            .find_tagged("member")
                            .map(|node| node.as_str().to_string())
                            .collect::<Vec<String>>()
                    });

                let module_name = module_name.as_str().trim_matches('"').to_string();
                let module_type = Type::Module(module_name.clone());
                let module_name_const = self
                    .borrow_mut()
                    .get_or_add_const(Value::Texte(module_name.clone()));
                self.borrow_mut().code.emit_load_module(module_name_const);

                let module_var = if let Some(alias) = alias {
                    self.borrow_mut()
                        .declare_local(&alias, module_type.into(), true)
                } else if vars.is_none() {
                    let module_file = module_name.rsplit_once("/").unwrap_or(("", &module_name)).1;
                    self.borrow_mut().declare_local(
                        &module_file.strip_suffix(".as").unwrap_or(module_file),
                        module_type.into(),
                        true,
                    )
                } else {
                    let module_file = module_name.rsplit_once("/").unwrap_or(("", &module_name)).1;
                    self.borrow_mut().declare_inner_local(
                        &module_file.strip_suffix(".as").unwrap_or(module_file),
                    )
                };

                self.borrow_mut().mark_initialized(module_var);
                self.borrow_mut().code.emit_set_local(module_var);
                if let Some(vars) = vars {
                    for var in vars {
                        let var_idx =
                            self.borrow_mut()
                                .declare_local(&var, Type::Tout.into(), true);

                        self.borrow_mut().mark_initialized(var_idx);

                        let var_name_idx = self
                            .borrow_mut()
                            .get_or_add_const(Value::Texte(var.clone()));

                        self.borrow_mut().code.emit_get_local(module_var);
                        self.borrow_mut().code.emit_get_attr(var_name_idx);
                        self.borrow_mut().code.emit_set_local(var_idx);
                    }
                }
            }
            Rule::DeclStmt => {
                self.parse_declare(pair.into_inner())?;
                // let (var, val) = self.parse_assign(pair.into_inner())?;
                // Stmt::Decl { var, val }
            }
            Rule::AssignStmt => {
                self.parse_assign(pair)?;
                // match  {
                //     (
                //         DeclVar::Var {
                //             name,
                //             static_type,
                //             is_const,
                //             public,
                //         },
                //         val,
                //     ) => Stmt::Assign {
                //         var: AssignVar::Var { name, static_type },
                //         val,
                //     },
                //     (decl @ DeclVar::ListUnpack(..), val) => Stmt::Assign {
                //         var: AssignVar::from(decl),
                //         val,
                //     },
                // };
            }
            Rule::CommandStmt => {
                let mut inner = pair.into_inner();
                // func
                self.parse_top_expr(inner.next().unwrap())?;
                // arg
                self.parse_expr(inner)?;
                self.borrow_mut().code.emit_call(1);
            }
            Rule::FnDef => {
                let mut inner = pair.into_inner();
                let name = inner
                    .find_first_tagged("name")
                    .map(|node| node.as_str().to_string());

                let local_idx = self.borrow_mut().declare_local(
                    name.as_ref().unwrap(),
                    Type::Fonction.into(),
                    false,
                );
                self.parse_fn(inner, name.clone())?;
                self.borrow_mut().mark_initialized(local_idx);
                self.borrow_mut().code.emit_set_local(local_idx);
            }

            Rule::StructureDef => {
                let mut inner = pair.into_inner();

                let name = inner
                    .find_first_tagged("name")
                    .map(|node| node.as_str().to_string())
                    .unwrap();

                // let mut struct_fields = HashMap::new();
                let mut fields_token = inner
                    .find(|node| node.as_rule() == Rule::StructureBody)
                    .unwrap()
                    .into_inner();

                // remove the "finStructure" token from the list
                fields_token.next_back();

                let mut field_types = HashMap::new();
                let mut fields = Vec::new();

                for field in fields_token {
                    let mut f_inner = field.into_inner();
                    let mut is_const = false;
                    let mut value = None;
                    match f_inner.peek().unwrap().as_rule() {
                        Rule::Const => {
                            f_inner.next();
                            is_const = true;
                        }
                        Rule::Var => {
                            f_inner.next();
                        }
                        _ => {}
                    }
                    let field_name = f_inner.next().unwrap().as_str().to_string();
                    let mut static_type = Type::Tout.into();
                    if let Some(static_type_pair) = f_inner.peek().and_then(|pair| {
                        if pair.as_rule() == Rule::TypeExpr {
                            Some(pair)
                        } else {
                            None
                        }
                    }) {
                        f_inner.next();
                        let static_type_span = static_type_pair.as_span();
                        static_type = self
                            .parse_type(static_type_pair.into_inner())?
                            .as_base_type()
                            .map_err(|err| err.to_error(static_type_span))?;
                    }

                    if let Some(_) = f_inner.next() {
                        let default_val = f_inner.next().unwrap();

                        let cmp = Compiler::new_closure(
                            default_val.as_str(),
                            None,
                            Rc::clone(self),
                            0,
                            static_type.clone().into(),
                        );

                        value = Some(Value::closure(
                            cmp.compile_lambda_expr(default_val.into_inner())?,
                        ));
                    }

                    field_types.insert(field_name.clone(), static_type.clone());

                    fields.push(ASFieldInfo {
                        name: field_name,
                        field_type: static_type.into(),
                        is_const,
                        is_private: false,
                        value,
                    });
                }

                let structure = ASStructure::new(name.clone(), fields);

                let idx = self
                    .borrow_mut()
                    .get_or_add_const(Value::Structure(Arc::new(RwLock::new(structure))));

                let struct_type = TypeSpec::new_simple(
                    name.clone(),
                    Type::Struct(StructType::new(name.clone(), field_types)),
                );

                let s_idx = self
                    .borrow_mut()
                    .declare_local(&name, struct_type.clone(), true);

                self.borrow_mut().mark_initialized(s_idx);

                self.borrow_mut().declare_local_type(&name, struct_type);

                self.borrow_mut().code.emit_const(idx);
                self.borrow_mut().code.emit_set_local(s_idx);
            }

            Rule::ImplDef => {
                let span = pair.as_span();
                let mut inner = pair.into_inner();

                let name = inner.next().map(|node| node.as_str().to_string()).unwrap();

                // remove the "finImpl" token from the list
                inner.next_back();

                let structure = self.borrow_mut().get_struct_const(&name).ok_or(
                    CompilationErrorKind::generic_error("Needs to be a struct").to_error(span),
                )?;

                for methode in inner {
                    self.parse_methode_def(Arc::clone(&structure), methode)?;
                }
            }

            Rule::SiStmt => self.parse_if(pair)?,

            Rule::RepeterStmt => {
                self.borrow_mut().begin_scope();

                let mut inner = pair.into_inner();
                let mut if_not_cond_jmp = None;

                let before_cond;

                if let Some(nb_iter) = inner
                    .clone()
                    .find(|p| matches!(p.as_node_tag(), Some("nb_iter")))
                {
                    let cptr = self.borrow_mut().declare_inner_local("compteur_repeter");
                    self.parse_expr(nb_iter.into_inner())?;

                    let mut compiler = self.borrow_mut();
                    compiler.code.emit_set_local(cptr);

                    // the code should jump to the start of the condition
                    before_cond = compiler.code.inner().len();

                    compiler.code.emit_get_local(cptr);
                    let num1 = compiler.get_or_add_const(Value::Entier(1));
                    compiler.code.emit_const(num1);

                    compiler.code.emit_binop(BinOpcode::Sub);
                    compiler.code.emit_set_local(cptr);

                    compiler.code.emit_get_local(cptr);
                    let num1 = compiler.get_or_add_const(Value::Entier(0));
                    compiler.code.emit_const(num1);
                    compiler.code.emit_bincomp(BinCompcode::Geq);

                    if_not_cond_jmp = Some(compiler.push_cond_jump());
                } else {
                    before_cond = self.borrow().code.inner().len();
                }

                if let Some(body) = inner.find(|p| p.as_rule() == Rule::StmtBody) {
                    self.build_ast_stmts(body.into_inner())?;
                }

                let now = self.borrow().code.inner().len();
                self.borrow_mut()
                    .code
                    .emit_jump(before_cond as i16 - now as i16 - 2); // - 2 here to account for this jump

                {
                    let mut compiler = self.borrow_mut();
                    while let Some(continue_jmp_idx) = compiler.continue_stack.pop() {
                        compiler.patch_jump_to(continue_jmp_idx, before_cond);
                    }
                }

                if let Some(if_not_cond_jmp) = if_not_cond_jmp {
                    self.borrow_mut().patch_jump(if_not_cond_jmp);
                }

                {
                    let mut compiler = self.borrow_mut();
                    while let Some(break_jmp_idx) = compiler.break_stack.pop() {
                        compiler.patch_jump(break_jmp_idx);
                    }
                }

                self.borrow_mut().end_scope();
            }

            Rule::TantQueStmt => {
                self.borrow_mut().begin_scope();

                let inner = pair.into_inner();
                let before_cond = self.borrow().code.inner().len();

                // cond
                self.parse_expr(
                    inner
                        .clone()
                        .find(|p| matches!(p.as_node_tag(), Some("cond")))
                        .unwrap()
                        .into_inner(),
                )?;

                let if_not_cond_jmp = self.borrow_mut().push_cond_jump();

                if let Some(body) = inner.clone().find(|p| p.as_rule() == Rule::StmtBody) {
                    self.build_ast_stmts(body.into_inner())?;
                }

                let now = self.borrow().code.inner().len();
                self.borrow_mut()
                    .code
                    .emit_jump(before_cond as i16 - now as i16 - 2); // - 2 here to account for this jump

                {
                    let mut compiler = self.borrow_mut();
                    while let Some(continue_jmp_idx) = compiler.continue_stack.pop() {
                        compiler.patch_jump_to(continue_jmp_idx, before_cond);
                    }
                }

                // instruction and its argument
                self.borrow_mut().patch_jump(if_not_cond_jmp);

                {
                    let mut compiler = self.borrow_mut();
                    while let Some(break_jmp_idx) = compiler.break_stack.pop() {
                        compiler.patch_jump(break_jmp_idx);
                    }
                }

                self.borrow_mut().end_scope();
            }
            Rule::PourStmt => {
                self.borrow_mut().begin_scope();

                let inner = pair.into_inner();

                // setup
                self.parse_expr(inner.find_first_tagged("iter").unwrap().into_inner())?;
                let iter_idx = self.borrow_mut().declare_inner_local("for_iter");
                let iter_state_idx = self.borrow_mut().declare_inner_local("for_state");
                self.borrow_mut().code.emit_set_local(iter_idx);
                self.borrow_mut().push_const(Value::Entier(0));
                self.borrow_mut().code.emit_set_local(iter_state_idx);

                // loop
                let start_loop = self.borrow().code.len_inner();

                self.borrow_mut().code.emit_for_next(iter_idx);
                let end_loop_jmp = self.borrow_mut().push_jump();

                // vars
                let vars = inner.find_first_tagged("vars").unwrap();
                self.parse_assign(vars)?;

                if let Some(body) = inner.clone().find(|p| p.as_rule() == Rule::StmtBody) {
                    self.build_ast_stmts(body.into_inner())?;
                }

                {
                    let mut compiler = self.borrow_mut();
                    while let Some(continue_jmp_idx) = compiler.continue_stack.pop() {
                        compiler.patch_jump_to(continue_jmp_idx, start_loop);
                    }
                }

                let end_loop = self.borrow().code.len_inner();
                // `- 2` because of the 'Jump' instruction that takes 2 instructions
                self.borrow_mut()
                    .code
                    .emit_jump(start_loop as i16 - end_loop as i16 - 2);

                self.borrow_mut().patch_jump(end_loop_jmp);

                {
                    let mut compiler = self.borrow_mut();
                    while let Some(break_jmp_idx) = compiler.break_stack.pop() {
                        compiler.patch_jump(break_jmp_idx);
                    }
                }

                self.borrow_mut().end_scope();
            }
            Rule::ContinuerStmt => {
                let jmp = self.borrow_mut().push_jump();
                self.borrow_mut().continue_stack.push(jmp);
            }
            Rule::SortirStmt => {
                let jmp = self.borrow_mut().push_jump();
                self.borrow_mut().break_stack.push(jmp);
            }
            Rule::RetournerStmt => {
                for expr in pair.into_inner().skip(1) {
                    self.parse_expr(expr.into_inner())?;
                }
                self.borrow_mut().code.emit_return();
            }
            Rule::Expr => {
                self.parse_expr(pair.into_inner())?;
                self.borrow_mut().code.emit_pop();
            }
            rule => Err(PestError::new_from_span(
                PestErrorVariant::ParsingError {
                    positives: vec![Rule::stmt],
                    negatives: vec![rule],
                },
                pair.as_span(),
            ))?,
        };
        Ok(())
    }

    fn build_ast_stmts(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError> {
        for pair in pairs {
            if matches!(pair.as_rule(), Rule::EOI) {
                continue;
            };
            self.build_ast_stmt(pair)?;
        }

        Ok(())
    }
}
