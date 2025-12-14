use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    hash::Hash,
    iter,
    rc::Rc,
    str::FromStr,
    sync::{Arc, RwLock},
};

use pest::{
    Parser as PestParser,
    error::{Error as PestError, ErrorVariant as PestErrorVariant},
    iterators::{Pair, Pairs},
};

use crate::{
    AlivescriptParser, Rule,
    as_obj::{ASErreur, ASErreurType},
    ast::{
        AssignVar, BinCompcode, BinLogiccode, BinOpcode, DeclVar, DefFn, Expr, FnParam, Stmt, Type,
        UnaryOpcode,
    },
    compiler::{
        bytecode::{Instructions, JUMP_OFFSET, Opcode, instructions_to_string_debug},
        err::CompilationError,
        obj::{ArcClosure, Upvalue, UpvalueSpec, Value},
        parser::{PRATT_EXPR_PARSER, PRATT_TYPE_PARSER},
        utils::format_table,
        value::{ASStructure, BaseType, Closure, Function, StructType, TypeSpec},
    },
    utils::Invert,
    visitor::{Visitable, Visitor},
};

mod bitmasks;
pub(crate) mod bytecode;
mod err;
pub mod obj;
mod parser;
mod utils;
pub mod value;

macro_rules! unpack {
    ($pat:pat = $expr:expr) => {
        let $pat = $expr else { unreachable!() };
    };
}

#[derive(Debug)]
pub struct Local {
    name: String, // Identifier text, needed for shadowing and error reporting.
    depth: i32,   // Scope depth: -1 = declared but not initialized,
    // 0+ = active scope levels.
    is_captured: bool, // Set to true if an inner function captures this variable.
    var_type: TypeSpec,
    is_const: bool,
}

#[derive(Debug)]
pub struct LocalType {
    spec: TypeSpec,
    depth: i32,
}

#[derive(Debug)]
pub struct Compiler<'a> {
    pub input: &'a str,

    // Current function being built
    pub function: Rc<RefCell<Function>>,
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

    pub jump_stack: Vec<usize>, // offsets to patch later
}

impl<'a> Compiler<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            function: Rc::new(RefCell::new(Function::new_anonymous(0))),
            code: Instructions::new(),
            parent: None,
            locals: vec![],
            local_types: vec![],
            scope_depth: 0,
            upvalues: vec![],
            had_error: false,
            panic_mode: false,
            jump_stack: vec![],
            return_type: BaseType::Tout.into(),
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
            function: Rc::new(RefCell::new(Function::new(name, nb_params))),
            code: Instructions::new(),
            parent: Some(parent),
            locals: vec![],
            local_types: vec![],
            scope_depth: 0,
            upvalues: vec![],
            had_error: false,
            panic_mode: false,
            jump_stack: vec![],
            return_type,
        }
    }

    pub fn parse_and_compile(self) -> Result<ArcClosure, CompilationError> {
        let stmts = AlivescriptParser::parse(Rule::script, self.input)?;

        Ok(ArcClosure::new(self.compile(stmts)?))
    }

    pub fn parse_and_compile_debug(self) -> Result<Closure, CompilationError> {
        let stmts = AlivescriptParser::parse(Rule::script, self.input)?;

        println!("{:#?}", &stmts);

        self.compile_debug(stmts)
    }

    pub fn compile(self, pairs: Pairs<'a, Rule>) -> Result<Closure, CompilationError> {
        let mut rc_self = Rc::new(RefCell::new(self));

        rc_self.build_ast_stmts(pairs)?;

        rc_self.borrow_mut().code.pop_if_op_is(Opcode::Pop);

        rc_self.borrow_mut().code.emit_return();

        rc_self.borrow_mut().finish();

        let x = Closure {
            function: Arc::new(rc_self.borrow().function.borrow().clone()),
            upvalues: vec![],
        };

        Ok(x)
    }

    pub fn compile_debug(self, pairs: Pairs<'a, Rule>) -> Result<Closure, CompilationError> {
        let mut rc_self = Rc::new(RefCell::new(self));

        rc_self.build_ast_stmts(pairs)?;

        rc_self.borrow_mut().code.pop_if_op_is(Opcode::Pop);

        rc_self.borrow_mut().code.emit_return();

        rc_self.borrow_mut().finish();

        println!(
            "----INSTRUCTIONS----\n{}",
            instructions_to_string_debug(
                &rc_self.borrow().function.borrow().code,
                Rc::clone(&rc_self)
            )
            .join("\n")
        );
        println!("----LOCALS----\n{:#?}", rc_self.borrow().locals);
        println!(
            "----CONSTANTS----\n{:#?}",
            rc_self.borrow().function.borrow().constants
        );

        let x = Closure {
            function: Arc::new(rc_self.borrow().function.borrow().clone()),
            upvalues: vec![],
        };

        Ok(x)
    }

    fn finish(&mut self) {
        let code = self.code.inner().clone();
        self.function.borrow_mut().code = code;
        self.function.borrow_mut().upvalue_specs = self.upvalues.clone();
        self.function.borrow_mut().upvalue_count = self.upvalues.len();
    }

    fn get_or_add_const(&mut self, obj: Value) -> u16 {
        let idx = self
            .function
            .borrow()
            .constants
            .iter()
            .enumerate()
            .find(|(i, o)| **o == obj)
            .map(|(i, o)| i);
        if let Some(idx) = idx {
            return idx as u16;
        }

        let mut f = self.function.borrow_mut();
        f.constants.push(obj);
        f.constants.len() as u16 - 1
    }

    fn func(&mut self) -> RefMut<'_, Function> {
        self.function.borrow_mut()
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
            depth: -1, // not initialized yet
            is_captured: false,
            var_type: BaseType::Tout.into(),
            is_const: false,
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
        });
        self.locals.len() as u16 - 1
    }

    fn mark_initialized(&mut self, index: u16) {
        let local = self.locals.get_mut(index as usize).unwrap();
        local.depth = self.scope_depth as i32;
    }

    fn declare_local_type(&mut self, name: &str, spec: TypeSpec) -> u16 {
        self.local_types.push(LocalType {
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

    fn resolve_upval(&mut self, name: &str) -> Result<Option<usize>, ASErreurType> {
        // 1. Check if we have a parent compiler
        let parent_rc = match &self.parent {
            Some(p) => Rc::clone(p),
            None => return Ok(None), // Reached the top-level script, not an upvalue
        };

        // We need mutable access to the parent's state (locals/upvalues)
        let mut parent = parent_rc.borrow_mut();

        // 2. Try to resolve as a LOCAL in the PARENT
        if let Some(local_idx) = parent.resolve_local(name, true)? {
            // FOUND: It's a local in the parent (Direct Capture)

            // Mark the local in the parent as captured.
            parent.mark_captured(local_idx);

            // Record it as a new upvalue in THIS compiler.
            // We capture the stack slot index (local_idx) from the parent's frame.
            let upval_idx = self.add_upvalue(true, local_idx);

            // Return the index of the newly created upvalue in *this* function's upvalues array.
            return Ok(Some(upval_idx));
        }

        // 3. Try to resolve as an UPVALUE in the PARENT (Indirect Capture)
        // Note: This is a recursive call!
        if let Some(upval_idx_in_parent) = parent.resolve_upval(name)? {
            // FOUND: It's already an upvalue in the parent's closure (Inherited Upvalue)

            // Record it as a new upvalue in THIS compiler.
            // We capture the upvalue index (upval_idx_in_parent) from the parent's upvalue array.
            let upval_idx = self.add_upvalue(false, upval_idx_in_parent);

            // Return the index of the newly created upvalue in *this* function's upvalues array.
            return Ok(Some(upval_idx));
        }

        // 4. Not found in the entire ancestry.
        Ok(None)
    }

    fn resolve_local(
        &mut self,
        name: &str,
        allow_uninit: bool,
    ) -> Result<Option<usize>, ASErreurType> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                if local.depth == -1 && !allow_uninit {
                    Err(ASErreurType::new_erreur(
                        Some("ErreurAccesVariableLocale".into()),
                        "Impossible de lire une variable dans son propre initialiseur.".into(),
                    ))?;
                }
                return Ok(Some(i));
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
        if let Ok(Some(local_idx)) = self.resolve_local(ident, false) {
            self.code.emit_get_local(local_idx as u16);
            return;
        }

        // 2. Try to resolve as an UPVALUE
        if let Ok(Some(upval_idx)) = self.resolve_upval(ident) {
            self.code.emit_get_upvalue(upval_idx as u16);
            return;
        }

        // 3. Load a GLOBAL by setting the variable name as a string constant
        // and emiting a LoadGlobal
        let glob_name_idx = self.get_or_add_const(Value::Texte(ident.to_string()));
        self.code.emit_get_global(glob_name_idx);
    }
}

trait Parser<'a> {
    fn parse_top_expr(&mut self, primary: Pair<'a, Rule>) -> Result<(), CompilationError>;

    fn parse_expr(
        &mut self,
        pairs: impl Iterator<Item = Pair<'a, Rule>>,
    ) -> Result<(), CompilationError>;

    fn parse_fn_params(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError>;

    fn parse_assign_vars(
        &mut self,
        pairs: Pairs<Rule>,
        is_const: Option<bool>,
        public: Option<bool>,
    ) -> Result<(), PestError<Rule>>;

    fn parse_assign(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError>;

    fn parse_declare(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError>;

    fn parse_lit(&mut self, pair: Pair<Rule>) -> Result<(), CompilationError>;

    fn parse_type(&mut self, pairs: Pairs<Rule>) -> Result<TypeSpec, CompilationError>;

    fn parse_if(&mut self, pair: Pair<'a, Rule>) -> Result<(), CompilationError>;

    fn build_ast_stmt(&mut self, pair: Pair<'a, Rule>) -> Result<(), CompilationError>;

    fn build_ast_stmts(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError>;
}

impl<'a> Parser<'a> for Rc<RefCell<Compiler<'a>>> {
    fn parse_top_expr(&mut self, primary: Pair<'a, Rule>) -> Result<(), CompilationError> {
        match primary.as_rule() {
            Rule::List => {
                let mut nb_el = 0;
                for arg in primary.into_inner() {
                    self.parse_expr(arg.into_inner())?;
                    nb_el += 1;
                }
                // todo: push list
                self.borrow_mut().code.emit_new_list(nb_el);
            }

            Rule::Expr => {
                self.parse_expr(primary.into_inner())?;
            }

            Rule::ListExpr => {
                for expr in primary.into_inner() {
                    self.parse_expr(expr.into_inner())?;
                }
            }

            Rule::Ident => {
                let ident = primary.as_str();
                self.borrow_mut().load_var(ident);
            }

            Rule::Lit => {
                self.parse_lit(primary.into_inner().next().unwrap())?;
            }

            Rule::StructInit => {
                let mut inner = primary.into_inner();

                let struct_name = inner.next().unwrap().as_str();

                while let Some(field) = inner.next() {}
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
                let inner = primary.into_inner();

                let params = inner.find_first_tagged("params").unwrap().into_inner();

                let closure = {
                    let body = inner.find_first_tagged("body").unwrap();
                    let return_type = inner
                        .find_first_tagged("return_type")
                        .map(|te| self.parse_type(te.into_inner()))
                        .invert()?;

                    let mut c = Rc::new(RefCell::new(Compiler::new_closure(
                        body.as_str(),
                        None,
                        Rc::clone(self),
                        0,
                        return_type.unwrap_or(BaseType::Tout.into()),
                    )));
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

                let idx = self
                    .borrow_mut()
                    .get_or_add_const(Value::Closure(Arc::new(closure)));

                self.borrow_mut().code.emit_closure(idx);
            }
            rule => Err(PestError::new_from_span(
                PestErrorVariant::ParsingError {
                    positives: vec![Rule::term],
                    negatives: vec![rule],
                },
                primary.as_span(),
            ))?,
        };

        Ok(())
    }

    fn parse_expr(
        &mut self,
        pairs: impl Iterator<Item = Pair<'a, Rule>>,
    ) -> Result<(), CompilationError> {
        PRATT_EXPR_PARSER
            .map_primary(|pair| Rc::clone(self).parse_top_expr(pair))
            .map_prefix(|prefix, rhs| {
                let rhs = rhs?;

                if let Ok(op) = UnaryOpcode::try_from(&prefix) {
                    match op {
                        UnaryOpcode::Negate => self.borrow_mut().code.emit_neg(),
                        _ => {}
                    }
                    // Ok(Box::new(Expr::UnaryOp { expr: rhs, op }))
                    Ok(())
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
                let lhs = lhs?;
                let rhs = rhs?;

                if let Ok(op) = BinOpcode::try_from(&infix) {
                    self.borrow_mut().code.emit_binop(op);
                    return Ok(());
                }
                if let Ok(op) = BinCompcode::try_from(&infix) {
                    self.borrow_mut().code.emit_bincomp(op);
                    return Ok(());
                }
                if let Ok(op) = BinLogiccode::try_from(&infix) {
                    todo!();
                }
                todo!();
            })
            .map_postfix(|lhs, postfix| {
                let lhs = lhs?;

                match postfix.as_rule() {
                    Rule::AccessProp => {
                        let prop = postfix.into_inner().next().unwrap();
                        if matches!(prop.as_node_tag(), Some("prop")) {
                            let idx = self
                                .borrow_mut()
                                .get_or_add_const(Value::Texte(prop.as_str().to_string()));
                            self.borrow_mut().code.emit_get_attr(idx);
                        } else {
                            Rc::clone(self).parse_expr(prop.into_inner())?;
                            self.borrow_mut().code.emit_get_item();
                        }
                    }
                    _ => unreachable!(),
                }
                //     Rule::Ternary => {
                //         let inner = postfix.into_inner();
                //         let then_expr = parse_expr(
                //             inner
                //                 .clone()
                //                 .find(|p| p.as_rule() == Rule::TernaryThen)
                //                 .unwrap()
                //                 .into_inner(),
                //         )?; // skip the "?"
                //         let else_expr = parse_expr(
                //             inner
                //                 .clone()
                //                 .find(|p| p.as_rule() == Rule::TernaryElse)
                //                 .unwrap()
                //                 .into_inner(),
                //         )?; // skip the ":"
                //         Ok(Box::new(Expr::Ternary {
                //             cond: lhs,
                //             then_expr,
                //             else_expr,
                //         }))
                //     }
                //     _ => Err(PestError::new_from_span(
                //         PestErrorVariant::ParsingError {
                //             positives: vec![Rule::Not, Rule::Neg, Rule::Pos],
                //             negatives: vec![postfix.as_rule()],
                //         },
                //         postfix.as_span(),
                //     )),
                // }
                Ok(())
            })
            .parse(pairs)?;
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

            let mut static_type = BaseType::Tout.into();
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

    fn parse_assign_vars(
        &mut self,
        pairs: Pairs<Rule>,
        is_const: Option<bool>,
        public: Option<bool>,
    ) -> Result<(), PestError<Rule>> {
        todo!()
    }

    fn parse_assign(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError> {
        let mut name = None;
        let mut static_type = BaseType::Tout.into();
        let mut op = None;
        // let mut var_list = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::Var | Rule::Assign => {}
                Rule::Expr => {
                    self.parse_expr(pair.into_inner())?;
                }
                Rule::AssignOp => {
                    let op_pair = pair.into_inner().next().unwrap();
                    op = Some(BinOpcode::try_from(&op_pair).unwrap());
                    // if we have an assign op, we push the current value of the var
                    self.borrow_mut().load_var(name.unwrap());
                }
                Rule::TypeExpr => static_type = self.parse_type(pair.into_inner())?,
                Rule::Ident => name = Some(pair.as_str()),
                // Rule::MultiDeclIdent => {
                //     var_list = Some(parse_assign_vars(
                //         pair.into_inner(),
                //         Some(is_const),
                //         Some(public),
                //     )?)
                // }
                // Rule::DeclIdentList => {
                //     var_list = Some(parse_assign_vars(
                //         pair.into_inner(),
                //         Some(is_const),
                //         Some(public),
                //     )?)
                // }
                _ => panic!("{:#?}", pair),
            }
        }

        let mut compiler = self.borrow_mut();

        // if we have an assign op, we already pushed the value of the var and
        // the value of the expression, the only thing that remains is adding
        // the bin op
        if let Some(op) = op {
            compiler.code.emit_binop(op);
        }

        let name = name.unwrap();

        // 1. Try to resolve as a LOCAL
        if let Ok(Some(local_idx)) = compiler.resolve_local(name, false) {
            compiler.code.emit_set_local(local_idx as u16);
            return Ok(());
        }

        // 2. Try to resolve as an UPVALUE
        if let Ok(Some(upval_idx)) = compiler.resolve_upval(name) {
            compiler.code.emit_set_upvalue(upval_idx as u16);
            return Ok(());
        }

        // 3. It defines a new local variable
        let local_idx = compiler.declare_local(name, BaseType::Tout.into(), false);

        compiler.mark_initialized(local_idx);
        compiler.code.emit_set_local(local_idx);

        Ok(())
    }

    fn parse_declare(&mut self, pairs: Pairs<'a, Rule>) -> Result<(), CompilationError> {
        let mut name = None;
        let mut static_type = BaseType::Tout.into();
        let mut is_const = false;
        let mut public = false;
        // let mut var_list = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::Const => is_const = true,
                Rule::Var | Rule::Assign => {}
                Rule::Pub => public = true,
                Rule::Expr => {
                    self.parse_expr(pair.into_inner())?;
                }
                Rule::TypeExpr => static_type = self.parse_type(pair.into_inner())?,
                Rule::Ident => name = Some(pair.as_str()),
                // Rule::MultiDeclIdent => {
                //     var_list = Some(parse_assign_vars(
                //         pair.into_inner(),
                //         Some(is_const),
                //         Some(public),
                //     )?)
                // }
                // Rule::DeclIdentList => {
                //     var_list = Some(parse_assign_vars(
                //         pair.into_inner(),
                //         Some(is_const),
                //         Some(public),
                //     )?)
                // }
                _ => panic!("{:#?}", pair),
            }
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
                Rule::Ident => Ok(BaseType::from_str(primary.as_str())
                    .map_err(|e| {
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
            Rule::AfficherStmt => self.parse_expr(pair.into_inner().skip(1))?,
            Rule::UtiliserStmt => {
                let inner = pair.into_inner();
                let module_name = inner.clone().next().unwrap();
                let alias = inner
                    .clone()
                    .find(|node| node.as_rule() == Rule::ModuleAlias)
                    .map(|alias| alias.as_str().to_string());
                let vars = inner
                    .clone()
                    .find(|node| node.as_rule() == Rule::UtiliserMembers)
                    .map(|node| {
                        node.into_inner()
                            .find_tagged("member")
                            .map(|node| node.as_str().to_string())
                            .collect::<Vec<String>>()
                    });
                // Stmt::Utiliser {
                //     module: module_name.as_str().trim_matches('"').to_string(),
                //     alias,
                //     vars,
                //     is_path: module_name.as_node_tag().is_some_and(|node| node == "path"),
                //     public: false,
                // }
            }
            Rule::DeclStmt => {
                self.parse_declare(pair.into_inner())?;
                // let (var, val) = self.parse_assign(pair.into_inner())?;
                // Stmt::Decl { var, val }
            }
            Rule::AssignStmt => {
                self.parse_assign(pair.into_inner())?;
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
                self.parse_top_expr(inner.next().unwrap())?;
                self.borrow_mut().code.emit_call(1);
            }
            Rule::PubStmt => {
                let mut inner = pair.into_inner();
                self.build_ast_stmt(inner.nth(1).unwrap())?;
                // result.mk_public();
                // *result
            }
            Rule::FnDef => {
                let mut inner = pair.into_inner();

                let name = inner
                    .find_first_tagged("name")
                    .map(|node| node.as_str().to_string());

                let params = inner.find_first_tagged("params").unwrap().into_inner();

                let local_idx = self.borrow_mut().declare_local(
                    name.as_ref().unwrap(),
                    BaseType::Fonction.into(),
                    false,
                );

                let closure = {
                    let return_type = inner
                        .find_first_tagged("return_type")
                        .map(|te| self.parse_type(te.into_inner()))
                        .invert()?;

                    let body = inner
                        .find(|node| node.as_rule() == Rule::FnBody)
                        .unwrap()
                        .into_inner()
                        .next()
                        .unwrap();

                    let mut c = Rc::new(RefCell::new(Compiler::new_closure(
                        body.as_str(),
                        name.clone(),
                        Rc::clone(self),
                        0,
                        return_type.unwrap_or(BaseType::Tout.into()),
                    )));
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

                let idx = self
                    .borrow_mut()
                    .get_or_add_const(Value::Closure(Arc::new(closure)));

                self.borrow_mut().code.emit_closure(idx as u16);
                self.borrow_mut().mark_initialized(local_idx);
            }

            Rule::StructureDef => {
                let mut inner = pair.into_inner();

                let name = inner
                    .find_first_tagged("name")
                    .map(|node| node.as_str().to_string())
                    .unwrap();

                // let mut struct_fields = HashMap::new();
                let fields_token = inner
                    .find(|node| node.as_rule() == Rule::StructureBody)
                    .unwrap()
                    .into_inner();

                let mut field_types = HashMap::new();
                for field in fields_token {}

                let structure = ASStructure::new(name.clone(), HashMap::new());

                let idx = self
                    .borrow_mut()
                    .get_or_add_const(Value::Structure(Arc::new(RwLock::new(structure))));

                self.borrow_mut().declare_local_type(
                    &name,
                    TypeSpec::new_simple(
                        name.clone(),
                        BaseType::Struct(StructType::new(name.clone(), field_types)),
                    ),
                );

                self.borrow_mut().code.emit_struct(idx as u16);
            }

            Rule::SiStmt => self.parse_if(pair)?,

            Rule::RepeterStmt => {
                let mut inner = pair.into_inner();
                let mut if_not_cond_jmp = None;

                let before_cond;

                if let Some(nb_iter) = inner
                    .clone()
                    .find(|p| matches!(p.as_node_tag(), Some("nb_iter")))
                {
                    let cptr = self.borrow_mut().declare_inner_local("compteur_repeter");
                    self.borrow_mut().mark_initialized(cptr);
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

                if let Some(if_not_cond_jmp) = if_not_cond_jmp {
                    self.borrow_mut().patch_jump(if_not_cond_jmp);
                }
            }

            Rule::TantQueStmt => {
                let inner = pair.into_inner();
                let before_cond = self.borrow().code.inner().len();

                let cond = self.parse_expr(
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
                // instruction and its argument
                self.borrow_mut().patch_jump(if_not_cond_jmp);
            }
            Rule::PourStmt => {
                let inner = pair.into_inner();
                // Stmt::Pour {
                //     var: parse_assign_vars(
                //         inner
                //             .clone()
                //             .find_first_tagged("vars")
                //             .unwrap()
                //             .into_inner(),
                //         None,
                //         Some(false),
                //     )?,
                //     iterable: parse_expr(
                //         inner
                //             .clone()
                //             .find_first_tagged("iter")
                //             .unwrap()
                //             .into_inner(),
                //     )?,
                //     body: inner
                //         .clone()
                //         .find(|p| p.as_rule() == Rule::StmtBody)
                //         .map(|body| build_ast_stmts(body.into_inner()))
                //         .invert()?
                //         .unwrap_or_default(),
                // }
            }
            Rule::ContinuerStmt => {}
            Rule::SortirStmt => {}
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
