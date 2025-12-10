use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use crate::{
    as_obj::{ASErreur, ASErreurType, ASObj},
    ast::{AssignVar, BinOpcode, DeclVar, DefFn, Expr, Stmt, Type},
    compiler::{
        bytecode::{Instructions, Opcode},
        obj::{Closure, Function, Upvalue, Value},
    },
    visitor::{Visitable, Visitor},
};

mod bitmasks;
mod bytecode;
mod obj;
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
    pub upvalues: Vec<Upvalue>,

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

    fn new_with_parent(parent: Rc<RefCell<Compiler<'a>>>) -> Self {
        Self {
            function: Rc::new(RefCell::new(Function::new_anonymous())),
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
            upvalues: rc_self
                .borrow()
                .upvalues
                .iter()
                .map(|up| Rc::new(RefCell::new(up.clone())))
                .collect(),
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
            upvalues: rc_self
                .borrow()
                .upvalues
                .iter()
                .map(|up| Rc::new(RefCell::new(up.clone())))
                .collect(),
        };
        x
    }

    fn finish(&mut self) {
        let code = self.code.inner().clone();
        self.function.borrow_mut().code = code;
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

    fn declare_local(&mut self, name: &str) -> u8 {
        self.locals.push(Local {
            name: name.to_string(),
            depth: -1, // not initialized yet
            is_captured: false,
        });
        self.locals.len() as u8 - 1
    }

    fn mark_initialized(&mut self) {
        let last = self.locals.last_mut().unwrap();
        last.depth = self.scope_depth as i32;
    }

    fn resolve_local(&mut self, name: &str) -> Result<Option<usize>, ASErreurType> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                if local.depth == -1 {
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

    fn resolve_upval(&mut self, name: &str) {
        match &self.parent {
            Some(p) => todo!(),
            None => todo!(),
        }
    }

    fn mark_captured(&mut self, index: usize) {
        self.locals[index].is_captured = true;
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

        self.borrow_mut().code.emit_const(idx as u8);
    }

    fn visit_expr_list(&mut self, expr: &Expr) {
        unpack!(Expr::List(exprs) = expr);
    }

    fn visit_expr_dict(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_ident(&mut self, expr: &Expr) {
        unpack!(Expr::Ident(ident) = expr);

        let idx = self.borrow_mut().resolve_local(ident).unwrap();
        match idx {
            Some(idx) => {
                self.borrow_mut().code.emit_get_local(idx as u8);
            }

            // its an upvalue or a global variable
            None => todo!(),
        }
    }

    fn visit_expr_accessprop(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_slice(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_fncall(&mut self, expr: &Expr) {
        todo!()
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
        todo!()
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
        todo!()
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
            let first_local = (comp.locals.len() - nb_locals) as u8;
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

        self.borrow_mut().mark_initialized();

        self.borrow_mut().code.emit_set_local(local_idx);
    }

    fn visit_stmt_assign(&mut self, stmt: &Stmt) {
        unpack!(Stmt::Assign { var, val } = stmt);

        unpack!(AssignVar::Var { name, static_type } = var);

        val.accept(self);

        let local_idx = self.borrow_mut().resolve_local(name).unwrap();
        match local_idx {
            Some(local_idx) => {
                self.borrow_mut().code.emit_set_local(local_idx as u8);
            }

            // its an upvalue or a global variable
            None => todo!(),
        }
    }

    fn visit_stmt_opassign(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_si(&mut self, stmt: &Stmt) {
        todo!()
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
        todo!()
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
            let c = Compiler::new_with_parent(Rc::clone(self));
            c.compile(body)
        };

        self.borrow_mut().mark_initialized();

        let idx = self
            .borrow_mut()
            .get_or_add_const(Value::Closure(Rc::new(closure)));

        self.borrow_mut().code.emit_closure(idx as u8);
        self.borrow_mut().code.emit_set_local(local_idx as u8);
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
