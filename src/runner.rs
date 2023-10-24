use std::collections::HashMap;

use crate::{
    as_obj::{ASObj, ASVar},
    ast::{BinCompcode, BinOpcode, DeclVar, Expr, Stmt},
    data::Data,
    visitor::{Visitable, Visitor},
};

pub struct Runner {
    expr_results: Vec<ASObj>,
    datas: Vec<Data>,
    env: HashMap<String, (ASVar, ASObj)>,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            expr_results: vec![],
            datas: vec![],
            env: HashMap::new(),
        }
    }

    pub fn get_datas(&self) -> Vec<Data> {
        self.datas.clone()
    }
}

impl Visitor for Runner {
    fn visit_generic_expr(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_generic_stmt(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_expr_lit(&mut self, expr: &Expr) {
        if let Expr::Lit(value) = expr {
            self.expr_results.push(value.clone());
        }
    }

    fn visit_expr_list(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_dict(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_ident(&mut self, expr: &Expr) {
        if let Expr::Ident(var_name) = expr {
            if let Some((var, val)) = self.env.get(var_name) {
                self.expr_results.push(val.clone());
            } else {
                panic!("Variable inconnue '{}'", var_name);
            }
        }
    }

    fn visit_expr_fncall(&mut self, expr: &Expr) {
        todo!()
    }

    fn visit_expr_binop(&mut self, expr: &Expr) {
        if let Expr::BinOp { lhs, op, rhs } = expr {
            (*lhs).accept(self);
            let lhs_value = self.expr_results.pop().expect("Lhs de bincomp");
            (*rhs).accept(self);
            let rhs_value = self.expr_results.pop().expect("Rhs de bincomp");

            use BinOpcode::*;
            self.expr_results.push(match op {
                Add => lhs_value + rhs_value,
                Sub => lhs_value - rhs_value,
                Mul => lhs_value * rhs_value,
                Div => lhs_value / rhs_value,
                _ => todo!(),
            });
        }
    }

    fn visit_expr_bincomp(&mut self, expr: &Expr) {
        if let Expr::BinComp { lhs, op, rhs } = expr {
            (*lhs).accept(self);
            let lhs_value = self.expr_results.pop().expect("Lhs de bincomp");
            (*rhs).accept(self);
            let rhs_value = self.expr_results.pop().expect("Rhs de bincomp");

            use BinCompcode::*;
            self.expr_results.push(ASObj::ASBooleen(match op {
                Eq => lhs_value == rhs_value,
                _ => false,
            }));
        }
    }

    fn visit_stmt_expr(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_afficher(&mut self, stmt: &Stmt) {
        if let Stmt::Afficher(expr) = stmt {
            (*expr).accept(self);
            let value = self.expr_results.pop().expect("Afficher prend un argument");
            self.datas.push(Data::Afficher(value.to_string()));
        }
    }

    fn visit_stmt_decl(&mut self, stmt: &Stmt) {
        if let Stmt::Decl { var, val } = stmt {
            (*val).accept(self);
            let value = self.expr_results.pop().expect("Decl valeur");
            let DeclVar::Var { name, static_type, is_const } = var else { panic!() };
            if static_type.is_some() && static_type.as_ref().unwrap() != &value.get_type() {
                panic!("Type Invalide");
            }
            let var = ASVar::new(name.clone(), static_type.clone(), *is_const);
            self.env.insert(name.clone(), (var, value));
        }
    }

    fn visit_stmt_assign(&mut self, stmt: &Stmt) {
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

    fn visit_stmt_deffn(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_retourner(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_defstruct(&mut self, stmt: &Stmt) {
        todo!()
    }
}
