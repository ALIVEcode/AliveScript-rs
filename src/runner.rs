use crate::{
    as_modules::ASModuleBuiltin,
    as_obj::{ASEnv, ASFnParam, ASObj, ASScope, ASType, ASVar},
    ast::{BinCompcode, BinOpcode, DeclVar, Expr, LireVar, Stmt, Type, TypeBinOpcode},
    data::Data,
    visitor::{Visitable, Visitor},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum EarlyExit {
    Retourner, // retourner d'une fonctionc
    Continuer, // remonter au début d'une boucle
    Sortir,    // sortir d'une boucle
}

pub struct Runner {
    expr_results: Vec<ASObj>,
    type_results: Vec<ASType>,
    datas: Vec<Data>,
    env: ASEnv,
    early_exit: Option<EarlyExit>,
}

impl Runner {
    pub fn new() -> Self {
        let mut new = Self {
            expr_results: vec![],
            type_results: vec![],
            datas: vec![],
            env: ASEnv::new(),
            early_exit: None,
        };
        ASModuleBuiltin::Builtin.load(&None, &Some(vec!["*".into()]), &mut new.env);
        new
    }

    pub fn get_env(&mut self) -> &mut ASEnv {
        &mut self.env
    }

    pub fn push_data(&mut self, data: Data) {
        self.datas.push(data)
    }

    pub fn pop_value(&mut self) -> Option<ASObj> {
        self.expr_results.pop()
    }

    pub fn get_datas(&self) -> Vec<Data> {
        self.datas.clone()
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

    fn eval_type(&mut self, t: &Type) -> Option<ASType> {
        t.accept(self);
        self.type_results.pop()
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
        if let Expr::List(exprs) = expr {
            let liste = exprs
                .iter()
                .map(|expr| self.eval_expr(expr).unwrap())
                .collect();
            self.expr_results.push(ASObj::ASListe(liste));
        }
    }

    fn visit_expr_paire(&mut self, expr: &Expr) {
        if let Expr::Paire { clef, val } = expr {
            let clef_val = self.eval_expr(clef).expect("Paire clef");
            let val_val = self.eval_expr(val).expect("Paire val");
            self.expr_results.push(ASObj::ASPaire {
                key: Box::new(clef_val),
                val: Box::new(val_val),
            });
        }
    }

    fn visit_expr_dict(&mut self, expr: &Expr) {
        if let Expr::Dict(exprs) = expr {
            let dict = exprs
                .iter()
                .map(|expr| self.eval_expr(expr).unwrap())
                .collect();
            self.expr_results.push(ASObj::ASDict(dict));
        }
    }

    fn visit_expr_ident(&mut self, expr: &Expr) {
        if let Expr::Ident(var_name) = expr {
            if let Some((var, val)) = self.env.get_var(var_name) {
                self.expr_results.push(val.clone());
            } else {
                panic!("Variable inconnue '{}'", var_name);
            }
        }
    }

    fn visit_expr_accessprop(&mut self, expr: &Expr) {
        if let Expr::AccessProp { obj, prop } = expr {
            let obj_val = self.eval_expr(obj).expect("AccessProp obj");

            self.expr_results.push(match obj_val {
                ASObj::ASModule { env } => env.get(prop).expect("AccessProp prop").1.clone(),
                _ => todo!(),
            });
        }
    }

    fn visit_expr_idx(&mut self, expr: &Expr) {
        if let Expr::Idx { obj, idx } = expr {
            let obj_val = self.eval_expr(obj).expect("Idx obj");
            let idx = self.eval_expr(idx).expect("Idx idx");

            self.expr_results.push(match obj_val {
                ASObj::ASListe(lst) => {
                    let ASObj::ASEntier(i) = idx else { panic!() };
                    lst[i as usize].clone()
                }
                ASObj::ASTexte(txt) => {
                    let ASObj::ASEntier(i) = idx else { panic!() };
                    ASObj::ASTexte(txt[i as usize..i as usize + 1].into())
                }
                _ => todo!(),
            })
        }
    }

    fn visit_expr_fncall(&mut self, expr: &Expr) {
        if let Expr::FnCall { func, args } = expr {
            let expr = self.eval_expr(func).expect("FnCall Fonc");
            if let ASObj::ASFonc {
                name,
                docs,
                params,
                ref body,
                return_type,
            } = expr
            {
                let mut env = ASScope::new();
                let mut args_iter = args.iter();

                // Set params dans env local de la fonction
                for param in params.iter() {
                    let arg = args_iter.next();
                    if let Some(arg_expr) = arg {
                        let arg_val = self.eval_expr(arg_expr).expect("FnCall arg");
                        if !ASType::type_match(&param.static_type, &arg_val.get_type()) {
                            panic!(
                                "Type de l'argument invalide. Attendu: {:?}, obtenu: {:?}",
                                param.static_type,
                                arg_val.get_type()
                            );
                        }
                        env.insert(param.to_asvar(), arg_val);
                    } else if let Some(default_expr) = param.default_value.clone() {
                        let default_val =
                            self.eval_expr(&default_expr).expect("FnCall default param");
                        env.insert(param.to_asvar(), default_val);
                    } else {
                        panic!("Paramètre sans valeur")
                    }
                }

                // Exec Body
                self.env.push_scope(env);
                self.visit_body(body);

                if !self.should_early_exit() {
                    self.expr_results.push(ASObj::ASNul);
                } else if !self.early_exit_matches(EarlyExit::Retourner) {
                    panic!("Sortie d'une fonction autrement qu'avec `retourner`")
                }

                // Clean up
                self.early_exit = None;
                self.env.pop_scope();

                // Retourner
                let type_returned = if let Some(returned_value) = self.expr_results.last() {
                    returned_value.get_type()
                } else {
                    ASType::Rien
                };
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

    fn visit_expr_callrust(&mut self, expr: &Expr) {
        if let Expr::CallRust(proc) = expr {
            let result = proc(self);
            if let Some(value) = result {
                self.expr_results.push(value);
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

    fn visit_stmt_lire(&mut self, stmt: &Stmt) {
        if let Stmt::Lire {
            factory,
            var,
            prompt,
        } = stmt
        {
            match var {
                LireVar::Decl(DeclVar::Var {
                    name,
                    static_type,
                    is_const,
                }) => {}
                LireVar::Assign(name) => {}
                LireVar::Decl(DeclVar::ListUnpack(_)) => todo!(),
            }
        }
    }

    fn visit_stmt_utiliser(&mut self, stmt: &Stmt) {
        if let Stmt::Utiliser {
            module,
            alias,
            vars,
        } = stmt
        {
            ASModuleBuiltin::from(module.as_str()).load(alias, vars, &mut self.env);
        }
    }

    fn visit_stmt_decl(&mut self, stmt: &Stmt) {
        if let Stmt::Decl { var, val } = stmt {
            let value = self.eval_expr(val).expect("Decl valeur");
            let DeclVar::Var { name, static_type, is_const } = var else { panic!() };
            let static_type = static_type
                .as_ref()
                .map(|t| self.eval_type(t).expect("Decl var type"));
            if static_type.is_some()
                && !ASType::type_match(static_type.as_ref().unwrap(), &value.get_type())
            {
                panic!(
                    "Type invalide. Attendu: {:?}, obtenu: {:?}",
                    static_type.unwrap(),
                    value.get_type(),
                );
            }
            let var = ASVar::new(name.clone(), static_type.clone(), *is_const);
            if self.env.declare(var, value).is_some() {
                panic!("Variable redéclarée {:?}", name);
            };
        }
    }

    fn visit_stmt_assign(&mut self, stmt: &Stmt) {
        if let Stmt::Assign { var, val } = stmt {
            let value = self.eval_expr(val).expect("Assign valeur");
            match var.as_ref() {
                Expr::Ident(var_name) => {
                    if let Some((var, _old_val)) = self.env.get_var(var_name) {
                        if var.is_const() {
                            panic!("Impossible de changer la valeur d'une constante")
                        }
                        if !var.type_match(&value.get_type()) {
                            panic!(
                                "Type invalide. Attendu: {:?}, obtenu: {:?}",
                                var.get_type(),
                                value.get_type()
                            );
                        }

                        self.env.assign(var.clone(), value);
                    } else {
                        panic!("Variable inconnue '{}'", var_name);
                    }
                }
                _ => todo!(),
            }
        }
    }

    fn visit_stmt_opassign(&mut self, stmt: &Stmt) {
        if let Stmt::OpAssign { var, op, val } = stmt {
            let value = self.eval_expr(val).expect("Assign valeur");
            if let Expr::Ident(var_name) = var.as_ref() {
                if let Some((var, old_val)) = self.env.get_var(var_name) {
                    if var.is_const() {
                        panic!("Impossible de changer la valeur d'une constante")
                    }
                    let new_value = Runner::do_op(old_val.clone(), op, value.clone());
                    if !var.type_match(&new_value.get_type()) {
                        panic!(
                            "Type invalide. Attendu: {:?}, obtenu: {:?}",
                            var.get_type(),
                            new_value.get_type()
                        );
                    }

                    self.env.assign(var.clone(), new_value);
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
                ASObj::ASListe(ls) => ls,
                _ => panic!("Pas itérable"),
            };

            let DeclVar::Var { name, static_type, is_const } = var else { panic!() };
            let static_type = static_type
                .as_ref()
                .map(|t| self.eval_type(t).expect("Pour var type"));
            let var = ASVar::new(name.clone(), static_type.clone(), *is_const);

            for val in iter {
                self.env.push_scope(ASScope::from(vec![(var.clone(), val)]));
                self.visit_body(body);
                self.env.pop_scope();
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
                self.env.push_scope(ASScope::new());
                self.visit_body(body);
                self.env.pop_scope();
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
            let return_type = return_type
                .as_ref()
                .map(|t| self.eval_type(t).expect("Return func type"));

            let func = ASObj::asfonc(
                Some(name.clone()),
                None,
                params
                    .iter()
                    .map(|param| {
                        ASFnParam::new(
                            param.name.clone(),
                            param
                                .static_type
                                .clone()
                                .map(|t| self.eval_type(&t).expect("Param type")),
                            param.default_value.clone(),
                        )
                    })
                    .collect(),
                body.clone(),
                return_type.clone(),
            );

            let func_var = ASVar::new(name.clone(), Some(ASType::Fonction), true);

            self.env.declare(func_var, func);
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

    fn visit_type_lit(&mut self, t: &Type) {
        if let Type::Lit(t) = t {
            self.type_results.push(match t.as_str() {
                "entier" => ASType::Entier,
                "decimal" => ASType::Decimal,
                "nombre" => ASType::nombre(),
                "iterable" => ASType::iterable(),
                "texte" => ASType::Texte,
                "liste" => ASType::Liste,
                "rien" => ASType::Rien,
                "fonction" => ASType::Fonction,
                "nul" => ASType::Nul,
                "tout" => ASType::Tout,
                other => todo!("Check si le type existe {:?}", other),
            });
        }
    }

    fn visit_type_binop(&mut self, t: &Type) {
        if let Type::BinOp { lhs, op, rhs } = t {
            let lhs_value = self.eval_type(lhs).expect("Lhs de type binop");
            let rhs_value = self.eval_type(rhs).expect("Rhs de type binop");
            self.type_results.push(match op {
                TypeBinOpcode::Union => ASType::union_of(lhs_value, rhs_value),
                TypeBinOpcode::Intersection => todo!(),
            });
        }
    }

    fn visit_type_opt(&mut self, t: &Type) {
        if let Type::Opt(t) = t {
            let type_val = self.eval_type(t).expect("Opt type");
            self.type_results
                .push(ASType::union_of(type_val, ASType::Nul));
        }
    }
}
