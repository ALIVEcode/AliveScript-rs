use std::collections::HashMap;

use crate::{
    as_obj::{ASObj, ASType, ASVar},
    ast::{BinCompcode, BinOpcode, DeclVar, Expr, Stmt},
    data::Data,
    visitor::{Visitable, Visitor},
};

type RunnerEnv = HashMap<String, (ASVar, ASObj)>;

pub struct Runner {
    expr_results: Vec<ASObj>,
    datas: Vec<Data>,
    envs: Vec<RunnerEnv>,
    early_exit: bool,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            expr_results: vec![],
            datas: vec![],
            envs: vec![HashMap::new()],
            early_exit: false,
        }
    }

    pub fn get_datas(&self) -> Vec<Data> {
        self.datas.clone()
    }

    fn should_early_exit(&self) -> bool {
        self.early_exit
    }

    fn eval_expr(&mut self, expr: &Expr) -> Option<ASObj> {
        expr.accept(self);
        self.expr_results.pop()
    }
}

impl Visitor for Runner {
    fn visit_body(&mut self, stmts: &Vec<Box<Stmt>>) {
        for stmt in stmts.iter() {
            if self.should_early_exit() {
                break;
            }
            self.expr_results.clear();
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
            if let Some((var, val)) = self.envs.iter().rev().find_map(|env| env.get(var_name)) {
                self.expr_results.push(val.clone());
            } else {
                panic!("Variable inconnue '{}'", var_name);
            }
        }
    }

    fn visit_expr_fncall(&mut self, expr: &Expr) {
        if let Expr::FnCall { func, args } = expr {
            let expr = self.eval_expr(func).expect("FnCall Fonc");
            if let ASObj::ASFonc {
                params,
                ref body,
                return_type,
            } = expr
            {
                let mut env = HashMap::new();
                let mut args_iter = args.iter();
                for param in params.iter() {
                    let arg = args_iter.next();
                    if let Some(arg_expr) = arg {
                        let arg_val = self.eval_expr(arg_expr).expect("FnCall arg");
                        env.insert(param.name.clone(), (param.to_asvar(), arg_val));
                    } else if let Some(default_expr) = param.default_value.clone() {
                        let default_val =
                            self.eval_expr(&default_expr).expect("FnCall default param");
                        env.insert(param.name.clone(), (param.to_asvar(), default_val));
                    } else {
                        panic!("Paramètre sans valeur")
                    }
                }
                self.envs.push(env);
                self.visit_body(body);
                if !self.early_exit {
                    self.expr_results.push(ASObj::ASNul);
                }
                self.early_exit = false;
                self.envs.pop();
                let type_returned = self.expr_results.last().unwrap().get_type();
                if !ASType::type_match(&return_type, &type_returned) {
                    panic!("Mauvais type de retour. Attendu: {:?}, Obtenu: {:?}", return_type, type_returned)
                }
            } else {
                panic!("Impossible d'appeler '{:?}'", expr);
            }
        }
    }

    fn visit_expr_binop(&mut self, expr: &Expr) {
        if let Expr::BinOp { lhs, op, rhs } = expr {
            (*lhs).accept(self);
            let lhs_value = self.expr_results.pop().expect("Lhs de binop");
            (*rhs).accept(self);
            let rhs_value = self.expr_results.pop().expect("Rhs de binop");

            use BinOpcode::*;
            self.expr_results.push(match op {
                Add => lhs_value + rhs_value,
                Sub => lhs_value - rhs_value,
                Mul => lhs_value * rhs_value,
                Div => lhs_value / rhs_value,
                DivInt => lhs_value.div_int(rhs_value),
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
        if let Stmt::Expr(expr) = stmt {
            self.eval_expr(expr);
        }
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
            self.envs
                .last_mut()
                .unwrap()
                .insert(name.clone(), (var, value));
        }
    }

    fn visit_stmt_assign(&mut self, stmt: &Stmt) {
        if let Stmt::Assign { var, val } = stmt {
            (*val).accept(self);
            let value = self.expr_results.pop().expect("Decl valeur");
            if let Expr::Ident(var_name) = var.as_ref() {
                let env = self.envs.last_mut().unwrap();
                if let Some((var, val)) = env.get(var_name) {
                    if var.is_const() {
                        panic!("Impossible de changer la valeur d'une constante")
                    }
                    if !var.type_match(&value.get_type()) {
                        panic!("Type Invalide");
                    }

                    env.insert(var_name.clone(), (var.clone(), value));
                } else {
                    panic!("Variable inconnue '{}'", var_name);
                }
            }
        }
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
        if let Stmt::DefFn {
            name,
            params,
            body,
            return_type,
        } = stmt
        {
            let func = ASObj::ASFonc {
                params: params.clone(),
                body: body.clone(),
                return_type: return_type.clone(),
            };

            let func_var = ASVar::new(name.clone(), Some(ASType::Fonction), true);

            self.envs
                .last_mut()
                .unwrap()
                .insert(name.clone(), (func_var, func));
        }
    }

    fn visit_stmt_retourner(&mut self, stmt: &Stmt) {
        if let Stmt::Retourner(expr) = stmt {
            if let Some(val_expr) = expr {
                // retourner valeur
                val_expr.accept(self);
            } else {
                // retourner
                self.expr_results.push(ASObj::ASNul);
            }
            self.early_exit = true;
        }
    }

    fn visit_stmt_defstruct(&mut self, stmt: &Stmt) {
        todo!()
    }
}
