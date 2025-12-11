use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use crate::{
    as_obj::{ASErreur, ASErreurType, ASObj},
    ast::{AssignVar, BinOpcode, DeclVar, DefFn, Expr, Stmt, Type},
    compiler::{
        bytecode::{Instructions, Opcode, JUMP_OFFSET},
        obj::{Closure, Function, Upvalue, UpvalueSpec, Value},
    },
    visitor::{Visitable, Visitor},
};

mod bitmasks;
mod bytecode;
mod module;
mod parser;
pub mod obj;
mod utils;
pub mod vm;

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
}

#[derive(Debug)]
pub struct Compiler<'a> {
    // Current function being built
    pub function: Rc<RefCell<Function>>,
    pub code: Instructions,

    // Compiler nesting
    pub parent: Option<Rc<RefCell<Compiler<'a>>>>,

    // Scope & locals
    pub locals: Vec<Local>,
    pub scope_depth: usize,

    // Captured variables
    pub upvalues: Vec<UpvalueSpec>,

    // Errors
    pub had_error: bool,
    pub panic_mode: bool,

    pub jump_stack: Vec<usize>, // offsets to patch later
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Self {
            function: Rc::new(RefCell::new(Function::new_anonymous())),
            code: Instructions::new(),
            parent: None,
            locals: vec![],
            scope_depth: 0,
            upvalues: vec![],
            had_error: false,
            panic_mode: false,
            jump_stack: vec![],
        }
    }

    fn new_closure(name: Option<String>, parent: Rc<RefCell<Compiler<'a>>>) -> Self {
        Self {
            function: Rc::new(RefCell::new(Function::new(name))),
            code: Instructions::new(),
            parent: Some(parent),
            locals: vec![],
            scope_depth: 0,
            upvalues: vec![],
            had_error: false,
            panic_mode: false,
            jump_stack: vec![],
        }
    }

    pub fn compile(self, stmts: &Vec<Box<Stmt>>) -> Closure {
        let mut rc_self = Rc::new(RefCell::new(self));
        rc_self.visit_body(stmts);

        rc_self.borrow_mut().code.emit_return();

        rc_self.borrow_mut().finish();

        let x = Closure {
            function: Rc::new(rc_self.borrow().function.borrow().clone()),
            upvalues: vec![],
        };
        x
    }

    pub fn compile_debug(self, stmts: &Vec<Box<Stmt>>) -> Closure {
        let mut rc_self = Rc::new(RefCell::new(self));
        rc_self.visit_body(stmts);

        rc_self.borrow_mut().code.emit_return();

        rc_self.borrow_mut().finish();

        println!("{:#?}", rc_self.borrow());

        let x = Closure {
            function: Rc::new(rc_self.borrow().function.borrow().clone()),
            upvalues: vec![],
        };
        x
    }

    fn finish(&mut self) {
        let code = self.code.inner().clone();
        self.function.borrow_mut().code = code;
        self.function.borrow_mut().upvalue_specs = self.upvalues.clone();
        self.function.borrow_mut().upvalue_count = self.upvalues.len();
    }

    fn get_or_add_const(&mut self, obj: Value) -> usize {
        let idx = self
            .function
            .borrow()
            .constants
            .iter()
            .enumerate()
            .find(|(i, o)| **o == obj)
            .map(|(i, o)| i);
        if let Some(idx) = idx {
            return idx;
        }

        let mut f = self.function.borrow_mut();
        f.constants.push(obj);
        f.constants.len() - 1
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
    }

    fn declare_local(&mut self, name: &str) -> u16 {
        self.locals.push(Local {
            name: name.to_string(),
            depth: -1, // not initialized yet
            is_captured: false,
        });
        self.locals.len() as u16 - 1
    }

    fn mark_initialized(&mut self) {
        let last = self.locals.last_mut().unwrap();
        last.depth = self.scope_depth as i32;
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

    fn resolve_local(&mut self, name: &str, allow_uninit: bool) -> Result<Option<usize>, ASErreurType> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                if local.depth == -1 && !allow_uninit{
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
}

impl<'a> Visitor for Rc<RefCell<Compiler<'a>>> {
    fn visit_body(&mut self, stmts: &Vec<Box<Stmt>>) {
        for stmt in stmts {
            stmt.accept(self);
        }
    }

    fn visit_generic_expr(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_generic_stmt(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_expr_lit(&mut self, expr: &Expr) {
        unpack!(Expr::Lit(obj) = expr);

        let idx = self
            .borrow_mut()
            .get_or_add_const(Value::ASObj(obj.clone()));

        self.borrow_mut().code.emit_const(idx as u16);
    }

    fn visit_expr_list(&mut self, expr: &Expr) {
        unpack!(Expr::List(exprs) = expr);
    }

    fn visit_expr_dict(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_ident(&mut self, expr: &Expr) {
        unpack!(Expr::Ident(ident) = expr);

        let mut compiler = self.borrow_mut();

        // 1. Try to resolve as a LOCAL
        if let Ok(Some(local_idx)) = compiler.resolve_local(ident, false) {
            compiler.code.emit_get_local(local_idx as u16);
            return;
        }

        // 2. Try to resolve as an UPVALUE
        if let Ok(Some(upval_idx)) = compiler.resolve_upval(ident) {
            compiler.code.emit_get_upvalue(upval_idx as u16);
            return;
        }

        // 3. Load a GLOBAL by setting the variable name as a string constant
        // and emiting a LoadGlobal
        let glob_name_idx = compiler.get_or_add_const(Value::ASObj(ASObj::ASTexte(ident.clone())));
        compiler.code.emit_get_global(glob_name_idx as u16);
    }

    fn visit_expr_accessprop(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_slice(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_fncall(&mut self, expr: &Expr) {
        unpack!(Expr::FnCall { func, args } = expr);

        func.accept(self);

        args.iter().for_each(|arg| arg.accept(self));

        self.borrow_mut().code.emit_call(args.len() as u16);
    }

    fn visit_expr_callrust(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_unaryop(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_binop(&mut self, expr: &Expr) {
        unpack!(Expr::BinOp { lhs, op, rhs } = expr);
        lhs.accept(self);
        rhs.accept(self);

        self.borrow_mut().code.emit_binop(*op);
    }

    fn visit_expr_bincomp(&mut self, expr: &Expr) {
        unpack!(Expr::BinComp { lhs, op, rhs } = expr);
        lhs.accept(self);
        rhs.accept(self);

        self.borrow_mut().code.emit_bincomp(*op);
    }

    fn visit_expr_binlogic(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_ternary(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_suite(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_deffn(&mut self, expr: &Expr) {
        unpack!(
            Expr::DefFn(DefFn {
                docs,
                name,
                params,
                return_type,
                body,
                public
            }) = expr
        );

        let closure = {
            let mut c = Compiler::new_closure(name.clone(), Rc::clone(self));
            for param in params {
                c.declare_local(&param.name);
                c.mark_initialized();
            }
            c.compile(body)
        };

        let idx = self
            .borrow_mut()
            .get_or_add_const(Value::Closure(Rc::new(closure)));

        self.borrow_mut().code.emit_closure(idx as u16);
    }

    fn visit_expr_debut(&mut self, expr: &Expr) {
        unpack!(Expr::Debut(stmts) = expr);

        self.borrow_mut().begin_scope();

        self.visit_body(stmts);

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

    fn visit_expr_essayer(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_stmt_expr(&mut self, stmt: &Stmt) {
        unpack!(Stmt::Expr(expr) = stmt);

        expr.accept(self);

        // we discard the value produced by the expression
        self.borrow_mut().code.emit_pop();
    }

    fn visit_stmt_afficher(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_lire(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_utiliser(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_decl(&mut self, stmt: &Stmt) {
        unpack!(Stmt::Decl { var, val } = stmt);

        unpack!(
            DeclVar::Var {
                name,
                static_type,
                is_const,
                public
            } = var
        );

        let local_idx = self.borrow_mut().declare_local(name);

        val.accept(self);

        let mut compiler = self.borrow_mut();

        compiler.mark_initialized();

        compiler.code.emit_set_local(local_idx);
    }

    fn visit_stmt_assign(&mut self, stmt: &Stmt) {
        unpack!(Stmt::Assign { var, val } = stmt);

        unpack!(AssignVar::Var { name, static_type } = var);

        val.accept(self);

        let mut compiler = self.borrow_mut();

        // 1. Try to resolve as a LOCAL
        if let Ok(Some(local_idx)) = compiler.resolve_local(name, false) {
            compiler.code.emit_set_local(local_idx as u16);
            return;
        }

        // 2. Try to resolve as an UPVALUE
        if let Ok(Some(upval_idx)) = compiler.resolve_upval(name) {
            compiler.code.emit_set_upvalue(upval_idx as u16);
            return;
        }

        // 3. It defines a new local variable
        let local_idx = compiler.declare_local(name);

        compiler.mark_initialized();
        compiler.code.emit_set_local(local_idx);
    }

    fn visit_stmt_opassign(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_si(&mut self, stmt: &Stmt) {
        unpack!(
            Stmt::Si {
                cond,
                then_br,
                elif_brs,
                else_br
            } = stmt
        );

        cond.accept(self);
        let if_not_cond_jmp = self.borrow_mut().push_cond_jump();
        self.visit_body(then_br);
        let mut to_end_jmps = vec![self.borrow_mut().push_jump()];
        self.borrow_mut().patch_jump(if_not_cond_jmp);

        for (elif_cond, elif_br) in elif_brs {
            elif_cond.accept(self);
            let elif_not_cond_jmp = self.borrow_mut().push_cond_jump();
            self.visit_body(elif_br);
            to_end_jmps.push(self.borrow_mut().push_jump());
            self.borrow_mut().patch_jump(elif_not_cond_jmp);
        }

        if let Some(else_br) = else_br {
            self.visit_body(else_br);
        }

        for to_end_jmp in to_end_jmps {
            self.borrow_mut().patch_jump(to_end_jmp);
        }
    }

    fn visit_stmt_condstmt(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_repeter(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_pour(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_tantque(&mut self, stmt: &Stmt) {
        unpack!(Stmt::TantQue { cond, body } = stmt);

        let before_cond = self.borrow().code.inner().len();

        cond.accept(self);
        let if_not_cond_jmp = self.borrow_mut().push_cond_jump();

        self.visit_body(body);

        let now = self.borrow().code.inner().len();
        self.borrow_mut()
            .code
            .emit_jump(before_cond as i16 - now as i16 - 2); // - 2 here to account for this
                                                             // instruction and its argument
        self.borrow_mut().patch_jump(if_not_cond_jmp);
    }

    fn visit_stmt_continuer(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_sortir(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_retourner(&mut self, stmt: &Stmt) {
        unpack!(Stmt::Retourner(exprs) = stmt);

        if exprs.len() > 1 {
            // FIXME: find a way to avoid this expensive and actually
            // useless .clone()
            self.visit_expr_list(&Expr::List(exprs.clone()));
        } else if exprs.len() == 1 {
            exprs[0].accept(self);
        }

        self.borrow_mut().code.emit_return();
    }

    fn visit_stmt_deffn(&mut self, stmt: &Stmt) {
        unpack!(
            Stmt::DefFn(DefFn {
                docs,
                name,
                params,
                return_type,
                body,
                public
            }) = stmt
        );

        let local_idx = self.borrow_mut().declare_local(name.as_ref().unwrap());

        let closure = {
            let mut c = Compiler::new_closure(name.clone(), Rc::clone(self));
            for param in params {
                c.declare_local(&param.name);
                c.mark_initialized();
            }
            c.compile(body)
        };

        self.borrow_mut().mark_initialized();

        let idx = self
            .borrow_mut()
            .get_or_add_const(Value::Closure(Rc::new(closure)));

        self.borrow_mut().code.emit_closure(idx as u16);
        self.borrow_mut().code.emit_set_local(local_idx as u16);
    }

    fn visit_stmt_defclasse(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_type(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_type_name(&mut self, t: &Type) {
        todo!()
    }

    fn visit_type_lit(&mut self, t: &Type) {
        todo!()
    }

    fn visit_type_binop(&mut self, t: &Type) {
        todo!()
    }

    fn visit_type_array(&mut self, t: &Type) {
        todo!()
    }

    fn visit_type_opt(&mut self, t: &Type) {
        todo!()
    }
}
