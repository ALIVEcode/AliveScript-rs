use std::collections::HashMap;

use crate::{
    as_obj::{ASObj, ASType, ASVar},
    ast::{BinCompcode, BinOpcode, DeclVar, Expr, Stmt},
    data::Data,
    visitor::{Visitable, Visitor},
};

type RunnerEnv = HashMap<String, (ASVar, ASObj)>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum EarlyExit {
    Retourner, // retourner d'une fonctionc
    Continuer, // remonter au début d'une boucle
    Sortir,    // sortir d'une boucle
}

pub struct Runner {
    expr_results: Vec<ASObj>,
    datas: Vec<Data>,
    envs: Vec<RunnerEnv>,
    early_exit: Option<EarlyExit>,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            expr_results: vec![],
            datas: vec![],
            envs: vec![HashMap::new()],
            early_exit: None,
        }
    }

    pub fn get_datas(&self) -> Vec<Data> {
        self.datas.clone()
    }

    fn get_env(&mut self) -> &mut RunnerEnv {
        self.envs.last_mut().unwrap()
    }

    fn should_early_exit(&self) -> bool {
        self.early_exit.is_some()
    }

    fn early_exit_matches(&self, early_exit: EarlyExit) -> bool {
        self.should_early_exit() && self.early_exit.unwrap() == early_exit
    }

    fn eval_expr(&mut self, expr: &Expr) -> Option<ASObj> {
        expr.accept(self);
        self.expr_results.pop()
    }

    fn do_op(lhs: ASObj, op: &BinOpcode, rhs: ASObj) -> ASObj {
        use BinOpcode::*;

        match op {
            Add => lhs + rhs,
            Sub => lhs - rhs,
            Mul => lhs * rhs,
            Div => lhs / rhs,
            DivInt => lhs.div_int(rhs),
            Mod => lhs % rhs,
            _ => todo!(),
        }
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

                // Set params dans env local de la fonction
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

                // Exec Body
                self.envs.push(env);
                self.visit_body(body);

                if !self.should_early_exit() {
                    self.expr_results.push(ASObj::ASNul);
                } else if !self.early_exit_matches(EarlyExit::Retourner) {
                    panic!("Sortie d'une fonction autrement qu'avec `retourner`")
                }

                // Clean up
                self.early_exit = None;
                self.envs.pop();

                // Retourner
                let type_returned = self.expr_results.last().unwrap().get_type();
                if !ASType::type_match(&return_type, &type_returned) {
                    panic!(
                        "Mauvais type de retour. Attendu: {:?}, Obtenu: {:?}",
                        return_type, type_returned
                    )
                }
            } else {
                panic!("Impossible d'appeler '{:?}'", expr);
            }
        }
    }

    fn visit_expr_range(&mut self, expr: &Expr) {
        if let Expr::Range {
            start,
            end,
            step,
            is_incl,
        } = expr
        {
            let start = self.eval_expr(start).expect("Range start");
            let end = self.eval_expr(end).expect("Range end");
            let (mut start, mut end) = match (start, end) {
                (ASObj::ASEntier(s), ASObj::ASEntier(e)) => (s, e),
                (s, e) => {
                    panic!("Range invalide: {:?}, {:?}", s, e);
                }
            };
            if start > end {
                (start, end) = (end + 1, start + 1);
                let range = if *is_incl { start - 1..end } else { start..end };
                let range = range.map(|i| ASObj::ASEntier(i)).rev().collect();
                self.expr_results.push(ASObj::ASListe(range));
            } else {
                let range = if *is_incl { start..end + 1 } else { start..end };
                let range = range.map(|i| ASObj::ASEntier(i)).collect();
                self.expr_results.push(ASObj::ASListe(range));
            }
        }
    }

    fn visit_expr_binop(&mut self, expr: &Expr) {
        if let Expr::BinOp { lhs, op, rhs } = expr {
            (*lhs).accept(self);
            let lhs_value = self.expr_results.pop().expect("Lhs de binop");
            (*rhs).accept(self);
            let rhs_value = self.expr_results.pop().expect("Rhs de binop");

            self.expr_results
                .push(Runner::do_op(lhs_value, op, rhs_value));
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
                NotEq => lhs_value != rhs_value,
                Lth => lhs_value < rhs_value,
                Gth => lhs_value > rhs_value,
                Leq => lhs_value <= rhs_value,
                Geq => lhs_value >= rhs_value,
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
            let value = self.eval_expr(expr).expect("Afficher prend un argument");
            self.datas.push(Data::Afficher(value.to_string()));
        }
    }

    fn visit_stmt_decl(&mut self, stmt: &Stmt) {
        if let Stmt::Decl { var, val } = stmt {
            let value = self.eval_expr(val).expect("Decl valeur");
            let DeclVar::Var { name, static_type, is_const } = var else { panic!() };
            if static_type.is_some() && static_type.as_ref().unwrap() != &value.get_type() {
                panic!("Type Invalide");
            }
            let var = ASVar::new(name.clone(), static_type.clone(), *is_const);
            if self.get_env().insert(name.clone(), (var, value)).is_some() {
                panic!("Variable redéclarée {:?}", name);
            };
        }
    }

    fn visit_stmt_assign(&mut self, stmt: &Stmt) {
        if let Stmt::Assign { var, val } = stmt {
            let value = self.eval_expr(val).expect("Assign valeur");
            if let Expr::Ident(var_name) = var.as_ref() {
                let env = self.get_env();
                if let Some((var, _old_val)) = env.get(var_name) {
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

    fn visit_stmt_opassign(&mut self, stmt: &Stmt) {
        if let Stmt::OpAssign { var, op, val } = stmt {
            let value = self.eval_expr(val).expect("Assign valeur");
            if let Expr::Ident(var_name) = var.as_ref() {
                let env = self.get_env();
                if let Some((var, old_val)) = env.get(var_name) {
                    if var.is_const() {
                        panic!("Impossible de changer la valeur d'une constante")
                    }
                    if !var.type_match(&value.get_type()) {
                        panic!("Type Invalide");
                    }
                    let new_value = Runner::do_op(old_val.clone(), op, value);

                    env.insert(var_name.clone(), (var.clone(), new_value));
                } else {
                    panic!("Variable inconnue '{}'", var_name);
                }
            }
        }
    }

    fn visit_stmt_si(&mut self, stmt: &Stmt) {
        if let Stmt::Si {
            cond,
            then_br,
            elif_brs,
            else_br,
        } = stmt
        {
            let cond_result = self.eval_expr(cond).expect("Si cond");
            if cond_result.to_bool() {
                self.visit_body(then_br);
            } else if let Some(else_br) = else_br {
                self.visit_body(else_br);
            }
        }
    }

    fn visit_stmt_condstmt(&mut self, stmt: &Stmt) {
        if let Stmt::CondStmt { cond, then_stmt } = stmt {
            let cond_result = self.eval_expr(cond).expect("CondStmt cond");
            if cond_result.to_bool() {
                then_stmt.accept(self);
            }
        }
    }

    fn visit_stmt_repeter(&mut self, stmt: &Stmt) {
        todo!()
    }

    fn visit_stmt_pour(&mut self, stmt: &Stmt) {
        if let Stmt::Pour {
            var,
            iterable,
            body,
        } = stmt
        {
            let iter_obj = self.eval_expr(iterable).expect("Pour iterable");
            let iter: Vec<ASObj> = match iter_obj {
                ASObj::ASTexte(s) => s.chars().map(|c| ASObj::ASTexte(String::from(c))).collect(),
                _ => panic!("Pas itérable"),
            };

            let DeclVar::Var { name, static_type, is_const } = var else { panic!() };
            let var = ASVar::new(name.clone(), static_type.clone(), *is_const);

            for val in iter {
                self.envs
                    .push(HashMap::from([(name.clone(), (var.clone(), val))]));
                self.visit_body(body);
                self.envs.pop();
                match self.early_exit {
                    Some(EarlyExit::Retourner) => break,
                    Some(EarlyExit::Continuer) => self.early_exit = None,
                    Some(EarlyExit::Sortir) => {
                        self.early_exit = None;
                        break;
                    }
                    None => {}
                }
            }
        }
    }

    fn visit_stmt_tantque(&mut self, stmt: &Stmt) {
        if let Stmt::TantQue { cond, body } = stmt {
            while self.eval_expr(cond).expect("Si cond").to_bool() {
                self.envs.push(HashMap::new());
                self.visit_body(body);
                self.envs.pop();
                match self.early_exit {
                    Some(EarlyExit::Retourner) => break,
                    Some(EarlyExit::Continuer) => self.early_exit = None,
                    Some(EarlyExit::Sortir) => {
                        self.early_exit = None;
                        break;
                    }
                    None => {}
                }
            }
        }
    }

    fn visit_stmt_continuer(&mut self, stmt: &Stmt) {
        self.early_exit = Some(EarlyExit::Continuer);
    }

    fn visit_stmt_sortir(&mut self, stmt: &Stmt) {
        self.early_exit = Some(EarlyExit::Sortir);
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

            self.get_env().insert(name.clone(), (func_var, func));
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
            self.early_exit = Some(EarlyExit::Retourner);
        }
    }

    fn visit_stmt_defstruct(&mut self, stmt: &Stmt) {
        todo!()
    }
}
