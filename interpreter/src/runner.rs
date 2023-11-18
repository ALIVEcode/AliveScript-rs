use std::{
    ops::{Deref, Index, IndexMut},
    str::FromStr,
};

use crate::{
    as_modules::ASModuleBuiltin,
    as_obj::{
        ASEnv, ASErreur, ASErreurType, ASFnParam, ASObj, ASScope, ASStructField, ASType, ASVar,
    },
    ast::{
        AssignVar, BinCompcode, BinLogiccode, BinOpcode, DeclVar, Expr, LireVar, Stmt, Type,
        TypeBinOpcode, UnaryOpcode,
    },
    data::{Data, Response},
    io::InterpretorIO,
    visitor::{Visitable, Visitor},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum EarlyExit {
    Retourner, // retourner d'une fonctionc
    Continuer, // remonter au début d'une boucle
    Sortir,    // sortir d'une boucle
    Erreur,    // Erreur
}

macro_rules! eval {
    (expr, $runner:ident, $expr:expr, $expect:literal) => {{
        ($expr).accept($runner);
        if $runner.error_thrown() {
            return;
        } else {
            $runner.expr_results.pop().expect($expect)
        }
    }};

    (opt_expr, $runner:ident, $expr:expr, $expect:literal) => {{
        if let Some(e) = $expr {
            e.accept($runner);
            if $runner.error_thrown() {
                return;
            } else {
                Some($runner.expr_results.pop().expect($expect))
            }
        } else {
            None
        }
    }};

    (call, $runner:ident, $func:expr, $args:expr, $expect:literal) => {{
        let to_call = Expr::FnCall {
            func: $func,
            args: $args.into_iter().map(|arg| Expr::literal(arg)).collect(),
        };
        to_call.accept($runner);
        if $runner.error_thrown() {
            return;
        } else {
            $runner.expr_results.pop().expect($expect)
        }
    }};

    (type, $runner:ident, $expr:expr, $expect:literal) => {{
        ($expr).accept($runner);
        if $runner.error_thrown() {
            return;
        } else {
            $runner.type_results.pop().expect($expect)
        }
    }};

    (opt_type, $runner:ident, $expr:expr, $expect:literal) => {{
        if let Some(t) = $expr {
            t.accept($runner);
            if $runner.error_thrown() {
                return;
            } else {
                Some($runner.type_results.pop().expect($expect))
            }
        } else {
            None
        }
    }};
}

macro_rules! throw_err {
    ($self:expr, $err:expr) => {{
        $self.throw_err($err);
        return;
    }};

    (?, $self:expr, $err:expr) => {{
        let result = $err;
        if result.is_err() {
            $self.throw_err(result.err().unwrap());
            return;
        }
    }};
}

pub struct Runner<'a> {
    expr_results: Vec<ASObj>,
    type_results: Vec<ASType>,
    io: &'a mut dyn InterpretorIO,
    env: ASEnv,
    early_exit: Option<EarlyExit>,
}

impl<'a> Runner<'a> {
    pub fn new<IO: InterpretorIO + 'a>(intepretor_io: &'a mut IO) -> Self {
        let mut new = Self {
            expr_results: vec![],
            type_results: vec![],
            io: intepretor_io,
            env: ASEnv::new(),
            early_exit: None,
        };
        ASModuleBuiltin::Builtin.load(&None, &Some(vec!["*".into()]), &mut new.env);
        new
    }

    pub fn get_env(&mut self) -> &mut ASEnv {
        &mut self.env
    }

    pub fn send_data(&mut self, data: Data) {
        self.io.send(data);
    }

    pub fn request_data(&mut self, data: Data) -> Option<Response> {
        self.io.request(data)
    }

    pub fn pop_value(&mut self) -> Option<ASObj> {
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
            BitwiseXor => (lhs ^ rhs).unwrap(),
            _ => todo!(),
        }
    }

    fn throw_err(&mut self, err: ASErreurType) {
        let error = ASErreur::new(err, 0);
        self.send_data(error.into());
        self.early_exit = Some(EarlyExit::Erreur);
    }

    fn set_early_exit(&mut self, early_exit: Option<EarlyExit>) {
        if !self.early_exit_matches(EarlyExit::Erreur) {
            self.early_exit = early_exit;
        }
    }

    fn clear_early_exit(&mut self) {
        self.set_early_exit(None);
    }

    fn should_early_exit(&self) -> bool {
        self.early_exit.is_some()
    }

    fn error_thrown(&self) -> bool {
        self.early_exit_matches(EarlyExit::Erreur)
    }

    fn early_exit_matches(&self, early_exit: EarlyExit) -> bool {
        matches!(self.early_exit, Some(reason) if reason == early_exit)
    }
}

impl Visitor for Runner<'_> {
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
            let mut liste = Vec::with_capacity(exprs.len());
            for expr in exprs {
                let val = eval!(expr, self, expr, "Element de liste");
                liste.push(val);
            }
            self.expr_results.push(ASObj::ASListe(liste));
        }
    }

    fn visit_expr_paire(&mut self, expr: &Expr) {
        if let Expr::Paire { clef, val } = expr {
            // let clef_val = self.eval_expr(clef).expect("Paire clef");
            // let val_val = self.eval_expr(val).expect("Paire val");
            let clef_val = eval!(expr, self, clef, "Paire clef");
            let val_val = eval!(expr, self, val, "Paire val");
            self.expr_results.push(ASObj::ASPaire {
                key: Box::new(clef_val),
                val: Box::new(val_val),
            });
        }
    }

    fn visit_expr_dict(&mut self, expr: &Expr) {
        if let Expr::Dict(exprs) = expr {
            let mut dict = Vec::with_capacity(exprs.len());
            for expr in exprs {
                let val = eval!(expr, self, expr, "Element de dict");
                dict.push(val);
            }
            self.expr_results.push(ASObj::ASDict(dict));
        }
    }

    fn visit_expr_ident(&mut self, expr: &Expr) {
        if let Expr::Ident(var_name) = expr {
            if let Some((var, val)) = self.env.get_var(var_name) {
                self.expr_results.push(val.clone());
            } else {
                throw_err!(self, ASErreurType::new_variable_inconnue(var_name.clone()));
            }
        }
    }

    fn visit_expr_accessprop(&mut self, expr: &Expr) {
        if let Expr::AccessProp { obj, prop } = expr {
            let obj_val = eval!(expr, self, obj, "AccessProp obj");

            let result = match &obj_val {
                ASObj::ASModule { env } => env.get(prop).expect("AccessProp prop").1.clone(),
                ASObj::ASStructure { name, fields } => {
                    let field = fields.into_iter().find(|field| &field.name == prop);
                    if let Some(ASStructField {
                        name,
                        vis,
                        static_type,
                        default_value,
                        is_const,
                    }) = field
                    {
                        let Some(value) =
                            eval!(opt_expr, self, default_value, "Default value field")
                        else {
                            throw_err!(
                                self,
                                ASErreurType::new_erreur_access_propriete(obj_val, prop.clone())
                            );
                        };
                        value
                    } else {
                        throw_err!(
                            self,
                            ASErreurType::new_erreur_propriete_pas_init(obj_val, prop.clone())
                        );
                    }
                }
                _ => todo!(),
            };
            self.expr_results.push(result);
        }
    }

    fn visit_expr_slice(&mut self, expr: &Expr) {
        if let Expr::Slice { obj, slice } = expr {
            let obj_val = eval!(expr, self, obj, "Idx obj");
            let slice = eval!(expr, self, slice, "Idx idx");

            let result = match (obj_val, slice) {
                (ASObj::ASListe(lst), ASObj::ASEntier(i)) => lst[i as usize].clone(),
                (ASObj::ASListe(lst), ASObj::ASListe(range)) => {
                    let mut lst_final = Vec::with_capacity(range.len());
                    for obj in range {
                        if let ASObj::ASEntier(i) = obj {
                            lst_final.push(lst[i as usize].clone());
                        } else {
                            throw_err!(
                                self,
                                ASErreurType::new_erreur_type(ASType::Entier, obj.get_type(),)
                            );
                        }
                    }
                    ASObj::ASListe(lst_final)
                }
                (ASObj::ASTexte(txt), ASObj::ASEntier(i)) => {
                    ASObj::ASTexte(txt[i as usize..i as usize + 1].into())
                }
                (ASObj::ASTexte(txt), ASObj::ASListe(range)) => {
                    let mut txt_final = String::with_capacity(range.len());
                    for obj in range {
                        if let ASObj::ASEntier(i) = obj {
                            txt_final.push(txt.chars().nth(i as usize).unwrap());
                        } else {
                            throw_err!(
                                self,
                                ASErreurType::new_erreur_type(ASType::Entier, obj.get_type(),)
                            );
                        }
                    }
                    ASObj::ASTexte(txt_final)
                }
                (ASObj::ASDict(dict), clef) => {
                    let el = dict.into_iter().find(
                        |el| matches!(el, ASObj::ASPaire { key, val } if *key.deref() == clef),
                    );
                    match el {
                        Some(ASObj::ASPaire { key, val }) => val.deref().clone(),
                        None => throw_err!(self, ASErreurType::new_erreur_clef(clef.clone())),
                        Some(_) => unreachable!(),
                    }
                }
                _ => todo!(),
            };

            self.expr_results.push(result);
        }
    }

    fn visit_expr_fncall(&mut self, expr: &Expr) {
        if let Expr::FnCall { func, args } = expr {
            let expr = eval!(expr, self, func, "FnCall Fonc");
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
                        let arg_val = eval!(expr, self, arg_expr, "FnCall arg");
                        if !ASType::type_match(&param.static_type, &arg_val.get_type()) {
                            throw_err!(
                                self,
                                ASErreurType::ErreurTypeAppel {
                                    func_name: name.clone(),
                                    param_name: param.name.clone(),
                                    type_attendu: param.static_type.clone(),
                                    type_obtenu: arg_val.get_type().clone(),
                                }
                            );
                        }
                        env.insert(param.to_asvar(), arg_val);
                    } else if let Some(default_expr) = param.default_value.clone() {
                        let default_val = eval!(expr, self, default_expr, "FnCall default param");
                        env.insert(param.to_asvar(), default_val);
                    } else {
                        panic!("Paramètre sans valeur")
                    }
                }

                // Exec Body
                self.env.push_scope(env);
                self.visit_body(body);

                // Fonction termine sans de "retourner"
                if !self.should_early_exit() && !ASType::type_match(&return_type, &ASType::Rien) {
                    self.expr_results.push(ASObj::ASNul);
                } else if !self.early_exit_matches(EarlyExit::Retourner) {
                    panic!("Sortie d'une fonction autrement qu'avec `retourner`")
                }

                // Clean up
                self.clear_early_exit();
                self.env.pop_scope();

                // Retourner
                let type_returned = if let Some(returned_value) = self.expr_results.last() {
                    returned_value.get_type()
                } else {
                    ASType::Rien
                };

                if !ASType::type_match(&return_type, &type_returned) {
                    throw_err!(
                        self,
                        ASErreurType::ErreurTypeRetour {
                            type_attendu: return_type,
                            type_obtenu: type_returned,
                        }
                    );
                }
            } else {
                panic!("Impossible d'appeler '{:?}'", expr);
            }
        }
    }

    fn visit_expr_struct_inst(&mut self, expr: &Expr) {
        let Expr::StructInst { structure, fields } = expr else {
            return;
        };

        let struct_parent = eval!(expr, self, structure, "Struct");
        let ASObj::ASStructure {
            name: struct_name,
            fields: parent_fields,
        } = &struct_parent
        else {
            throw_err!(
                self,
                ASErreurType::new_erreur_type(ASType::Structure, struct_parent.get_type())
            )
        };
        let mut env = ASScope::new();

        for field in parent_fields {
            let field_value = eval!(
                opt_expr,
                self,
                field.default_value.clone(),
                "Struct default value"
            );
            let field_var = ASVar::new(
                field.name.clone(),
                Some(field.static_type.clone()),
                field.is_const,
            );
            if let Some(value) = field_value {
                env.declare(field_var, value);
            } else {
                env.declare(field_var, ASObj::ASNul);
            }
        }

        for field_expr in fields.into_iter() {
            let ASObj::ASPaire { key, val } = eval!(expr, self, field_expr, "Struct field") else {
                unreachable!()
            };
            let ASObj::ASTexte(key) = *key else {
                unreachable!()
            };

            throw_err!(?, self, env.assign_type_strict(&key, *val));
        }
    }

    fn visit_expr_callrust(&mut self, expr: &Expr) {
        let Expr::CallRust(proc) = expr else { return };

        let result = proc(self);
        match result {
            Ok(Some(value)) => self.expr_results.push(value),
            Ok(None) => {}
            Err(err) => throw_err!(self, err),
        }
    }

    fn visit_expr_range(&mut self, expr: &Expr) {
        let Expr::Range {
            start,
            end,
            step,
            is_incl,
        } = expr
        else {
            return;
        };

        let start = eval!(expr, self, start, "Range start");
        let end = eval!(expr, self, end, "Range end");
        let step = eval!(opt_expr, self, step, "Range step").unwrap_or(ASObj::ASEntier(1));

        let (mut start_val, mut end_val, step_val) = match (&start, &end, &step) {
            (ASObj::ASEntier(s), ASObj::ASEntier(e), ASObj::ASEntier(step)) if *step != 0 => {
                (*s, *e, *step)
            }
            (s, e, step) => {
                throw_err!(
                    self,
                    ASErreurType::new_erreur_suite_invalide(s.clone(), e.clone(), step.clone(),)
                );
            }
        };

        if start_val > end_val && step_val < 0 {
            (start_val, end_val) = (end_val + 1, start_val + 1);
            let range = if *is_incl {
                start_val - 1..end_val
            } else {
                start_val..end_val
            };
            let range = range
                .rev()
                .step_by(step_val.abs() as usize)
                .map(|i| ASObj::ASEntier(i))
                .collect();
            self.expr_results.push(ASObj::ASListe(range));
        } else if start_val < end_val && step_val > 0 {
            let range = if *is_incl {
                start_val..end_val + 1
            } else {
                start_val..end_val
            };
            let range = range
                .step_by(step_val as usize)
                .map(|i| ASObj::ASEntier(i))
                .collect();
            self.expr_results.push(ASObj::ASListe(range));
        } else {
            throw_err!(self, ASErreurType::new_erreur_suite_invalide(start, end, step));
        }
    }

    fn visit_expr_unaryop(&mut self, expr: &Expr) {
        if let Expr::UnaryOp { expr, op } = expr {
            use UnaryOpcode::*;

            let value = eval!(expr, self, expr, "Lhs de binop");

            let result = match op {
                Pas => ASObj::ASBooleen(!value.to_bool()),
                Negate => todo!(),
            };

            self.expr_results.push(result);
        }
    }

    fn visit_expr_binop(&mut self, expr: &Expr) {
        if let Expr::BinOp { lhs, op, rhs } = expr {
            let lhs_value = eval!(expr, self, lhs, "Lhs de binop");
            let rhs_value = eval!(expr, self, rhs, "Rhs de binop");

            self.expr_results
                .push(Runner::do_op(lhs_value, op, rhs_value));
        }
    }

    fn visit_expr_bincomp(&mut self, expr: &Expr) {
        if let Expr::BinComp { lhs, op, rhs } = expr {
            let lhs_value = eval!(expr, self, lhs, "Lhs de binop");
            let rhs_value = eval!(expr, self, rhs, "Rhs de binop");

            use BinCompcode::*;
            let result = ASObj::ASBooleen(match op {
                Eq => lhs_value == rhs_value,
                NotEq => lhs_value != rhs_value,
                Lth => lhs_value < rhs_value,
                Gth => lhs_value > rhs_value,
                Leq => lhs_value <= rhs_value,
                Geq => lhs_value >= rhs_value,
                Dans => {
                    let r = rhs_value.contains(&lhs_value);
                    match r {
                        Err(err) => throw_err!(self, err),
                        Ok(r) => r,
                    }
                }
                PasDans => {
                    let r = rhs_value.contains(&lhs_value);
                    match r {
                        Err(err) => throw_err!(self, err),
                        Ok(r) => !r,
                    }
                }
            });
            self.expr_results.push(result);
        }
    }

    fn visit_expr_binlogic(&mut self, expr: &Expr) {
        if let Expr::BinLogic { lhs, op, rhs } = expr {
            let lhs_value = eval!(expr, self, lhs, "Lhs de bin logique");

            use BinLogiccode::*;
            let result = match op {
                Et => {
                    if !lhs_value.to_bool() {
                        lhs_value
                    } else {
                        eval!(expr, self, rhs, "Rhs de bin logique")
                    }
                }
                Ou => {
                    if lhs_value.to_bool() {
                        lhs_value
                    } else {
                        eval!(expr, self, rhs, "Rhs de bin logique")
                    }
                }
            };

            self.expr_results.push(result);
        }
    }

    fn visit_stmt_expr(&mut self, stmt: &Stmt) {
        if let Stmt::Expr(expr) = stmt {
            expr.accept(self);
            self.expr_results.pop();
        }
    }

    fn visit_stmt_afficher(&mut self, stmt: &Stmt) {
        if let Stmt::Afficher(exprs) = stmt {
            let mut values = Vec::with_capacity(exprs.len());
            for expr in exprs {
                let value = eval!(expr, self, expr, "Afficher prend un argument");
                values.push(value.to_string());
            }
            let string = values.join(" ");
            self.send_data(Data::Afficher(string));
        }
    }

    fn visit_stmt_lire(&mut self, stmt: &Stmt) {
        if let Stmt::Lire {
            factory,
            var,
            prompt,
        } = stmt
        {
            let prompt_obj = eval!(opt_expr, self, prompt, "Prompt lire");

            match var {
                LireVar::Decl(DeclVar::Var {
                    name,
                    static_type,
                    is_const,
                })
                | LireVar::Assign(AssignVar::Decl(DeclVar::Var {
                    name,
                    static_type,
                    is_const,
                })) => {
                    let is_assign = matches!(var, LireVar::Assign(_));

                    let mut static_type = eval!(opt_type, self, static_type, "Lire var type");

                    if is_assign {
                        if let Some((var, _old_val)) = self.env.get_var(name) {
                            if var.is_const() {
                                throw_err!(
                                    self,
                                    ASErreurType::AffectationConstante {
                                        var_name: name.clone()
                                    }
                                )
                            }
                            if static_type.is_none() {
                                static_type = Some(var.get_type().clone());
                            }
                        }
                    }
                    let res_prompt = prompt_obj.map(|obj| {
                        if let ASObj::ASTexte(prompt) = obj {
                            Ok(prompt)
                        } else {
                            Err(ASErreurType::new_erreur_type(
                                ASType::Texte,
                                obj.get_type(),
                            ))
                        }
                    });
                    if let Some(Err(err)) = res_prompt {
                        throw_err!(self, err);
                    }
                    let Response::Text(reponse) = self
                        .request_data(Data::Demander {
                            prompt: res_prompt.map(|p| p.ok().unwrap()),
                        })
                        .unwrap();

                    let reponse = reponse.trim().to_string();

                    let static_type: ASType = static_type.into();
                    let value = match factory {
                        Some(factory) => Ok(eval!(
                            call,
                            self,
                            factory.clone(),
                            vec![ASObj::ASTexte(reponse)],
                            "Factory returns a value"
                        )),
                        None => static_type.convert_to_obj(reponse),
                    };

                    let Ok(value) = value else {
                        throw_err!(self, value.err().unwrap())
                    };

                    if !ASType::type_match(&static_type, &value.get_type()) {
                        throw_err!(
                            self,
                            ASErreurType::ErreurType {
                                type_attendu: static_type,
                                type_obtenu: value.get_type(),
                            }
                        );
                    }

                    let var = ASVar::new(name.clone(), Some(static_type), *is_const);
                    self.env.declare(var, value);
                }
                LireVar::Assign(_) => todo!(),
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
            let value = eval!(expr, self, val, "Decl valeur");
            let DeclVar::Var {
                name,
                static_type,
                is_const,
            } = var
            else {
                panic!()
            };
            let static_type = eval!(opt_type, self, static_type, "Decl var type");
            if static_type.is_some()
                && !ASType::type_match(static_type.as_ref().unwrap(), &value.get_type())
            {
                throw_err!(
                    self,
                    ASErreurType::ErreurType {
                        type_attendu: static_type.unwrap(),
                        type_obtenu: value.get_type(),
                    }
                );
            }
            let var = ASVar::new(name.clone(), static_type.clone(), *is_const);
            if self.env.declare(var, value).is_some() {
                panic!("Variable redéclarée {:?}", name);
            };
        }
    }

    fn visit_stmt_assign(&mut self, stmt: &Stmt) {
        if let Stmt::Assign {
            var: assign_var,
            val,
        } = stmt
        {
            let value = eval!(expr, self, val, "Assign valeur");
            match assign_var {
                AssignVar::Decl(DeclVar::Var {
                    name,
                    static_type,
                    is_const,
                }) => {
                    if let Some((var, _old_val)) = self.env.get_var(name) {
                        // if var.is_const() {
                        //     throw_err!(
                        //         self,
                        //         ASErreurType::AffectationConstante {
                        //             var_name: name.clone()
                        //         }
                        //     )
                        // }
                        // if !var.type_match(&value.get_type()) {
                        //     throw_err!(
                        //         self,
                        //         ASErreurType::ErreurType {
                        //             type_attendu: var.get_type().clone(),
                        //             type_obtenu: value.get_type()
                        //         }
                        //     );
                        // }

                        throw_err!(?, self, self.env.assign_strict(name, value));
                    } else {
                        let static_type = eval!(opt_type, self, static_type, "Assign var type");
                        if static_type.is_some()
                            && !ASType::type_match(static_type.as_ref().unwrap(), &value.get_type())
                        {
                            throw_err!(
                                self,
                                ASErreurType::ErreurType {
                                    type_attendu: static_type.unwrap(),
                                    type_obtenu: value.get_type(),
                                }
                            );
                        }
                        let var = ASVar::new(name.clone(), static_type.clone(), *is_const);
                        self.env.declare(var, value);
                    }
                }
                AssignVar::Slice { obj, slice } => {
                    use ASObj::*;

                    let var_val = eval!(expr, self, obj, "Assign Slice Obj");
                    let slice_val = eval!(expr, self, slice, "Assign Slice Slice");
                    match (var_val, slice_val) {
                        (ASListe(mut lst), ASEntier(i)) => {
                            *lst.index_mut(i as usize) = value;
                        }
                        _ => todo!(),
                    }
                }
                AssignVar::Decl(_) => todo!(),
            }
        }
    }

    fn visit_stmt_opassign(&mut self, stmt: &Stmt) {
        if let Stmt::OpAssign {
            var: assign_var,
            op,
            val,
        } = stmt
        {
            let value = eval!(expr, self, val, "Assign valeur");
            match assign_var {
                AssignVar::Decl(DeclVar::Var {
                    name,
                    static_type,
                    is_const,
                }) => {
                    if let Some((var, old_val)) = self.env.get_var(name) {
                        if var.is_const() {
                            throw_err!(
                                self,
                                ASErreurType::AffectationConstante {
                                    var_name: name.clone()
                                }
                            )
                        }

                        let value = Runner::<'_>::do_op(old_val.clone(), op, value);

                        if !var.type_match(&value.get_type()) {
                            throw_err!(
                                self,
                                ASErreurType::ErreurType {
                                    type_attendu: var.get_type().clone(),
                                    type_obtenu: value.get_type()
                                }
                            );
                        }

                        self.env.assign(name, value);
                    } else {
                        throw_err!(self, ASErreurType::new_variable_inconnue(name.clone()))
                    }
                }
                AssignVar::Slice { obj, slice } => {
                    use ASObj::*;

                    let var_val = eval!(expr, self, obj, "Assign Slice Obj");
                    let slice_val = eval!(expr, self, slice, "Assign Slice Slice");
                    match (var_val, slice_val) {
                        (ASListe(mut lst), ASEntier(i)) => {
                            let old_value = lst.index(i as usize).clone();
                            *lst.index_mut(i as usize) = Runner::<'_>::do_op(old_value, op, value);
                        }
                        _ => todo!(),
                    }
                }
                AssignVar::Decl(_) => todo!(),
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
            let cond_result = eval!(expr, self, cond, "Si cond");
            if cond_result.to_bool() {
                self.visit_body(then_br);
            } else if let Some(else_br) = else_br {
                self.visit_body(else_br);
            }
        }
    }

    fn visit_stmt_condstmt(&mut self, stmt: &Stmt) {
        if let Stmt::CondStmt { cond, then_stmt } = stmt {
            let cond_result = eval!(expr, self, cond, "CondStmt cond");
            if cond_result.to_bool() {
                then_stmt.accept(self);
            }
        }
    }

    fn visit_stmt_repeter(&mut self, stmt: &Stmt) {
        if let Stmt::Repeter { n, body } = stmt {
            let n_iter = eval!(opt_expr, self, n, "Repeter n");
            let n_value = match n_iter {
                Some(ASObj::ASEntier(i)) => {
                    if i >= 0 {
                        Some(i)
                    } else {
                        throw_err!(
                            self,
                            ASErreurType::new_erreur_valeur(
                                Some("La valeur doit être un entier >= 0.".into()),
                                ASObj::ASEntier(i)
                            )
                        )
                    }
                }
                Some(o) => throw_err!(
                    self,
                    ASErreurType::new_erreur_type(ASType::Entier, o.get_type().clone())
                ),
                None => None,
            };

            let mut counter = 0;
            loop {
                if let Some(n) = n_value {
                    if n == counter {
                        break;
                    }
                    counter += 1;
                }
                self.env.push_scope(ASScope::new());
                self.visit_body(body);
                self.env.pop_scope();
                match self.early_exit {
                    Some(EarlyExit::Retourner | EarlyExit::Erreur) => break,
                    Some(EarlyExit::Continuer) => self.clear_early_exit(),
                    Some(EarlyExit::Sortir) => {
                        self.clear_early_exit();
                        break;
                    }
                    None => {}
                };
            }
        }
    }

    fn visit_stmt_pour(&mut self, stmt: &Stmt) {
        if let Stmt::Pour {
            var,
            iterable,
            body,
        } = stmt
        {
            let iter_obj = eval!(expr, self, iterable, "Pour iterable");

            let iter: Vec<ASObj> = match iter_obj {
                ASObj::ASTexte(s) => s.chars().map(|c| ASObj::ASTexte(String::from(c))).collect(),
                ASObj::ASListe(ls) => ls,
                _ => panic!("Pas itérable"),
            };

            let DeclVar::Var {
                name,
                static_type,
                is_const,
            } = var
            else {
                panic!()
            };
            let static_type = eval!(opt_type, self, static_type, "Pour var type");

            let var = ASVar::new(name.clone(), static_type.clone(), *is_const);

            for val in iter {
                self.env.push_scope(ASScope::from(vec![(var.clone(), val)]));
                self.visit_body(body);
                self.env.pop_scope();
                match self.early_exit {
                    Some(EarlyExit::Retourner | EarlyExit::Erreur) => break,
                    Some(EarlyExit::Continuer) => self.clear_early_exit(),
                    Some(EarlyExit::Sortir) => {
                        self.clear_early_exit();
                        break;
                    }
                    None => {}
                }
            }
        }
    }

    fn visit_stmt_tantque(&mut self, stmt: &Stmt) {
        if let Stmt::TantQue { cond, body } = stmt {
            while eval!(expr, self, cond, "Si cond").to_bool() {
                self.env.push_scope(ASScope::new());
                self.visit_body(body);
                self.env.pop_scope();
                match self.early_exit {
                    Some(EarlyExit::Retourner | EarlyExit::Erreur) => break,
                    Some(EarlyExit::Continuer) => self.clear_early_exit(),
                    Some(EarlyExit::Sortir) => {
                        self.clear_early_exit();
                        break;
                    }
                    None => {}
                }
            }
        }
    }

    fn visit_stmt_continuer(&mut self, stmt: &Stmt) {
        self.set_early_exit(Some(EarlyExit::Continuer));
    }

    fn visit_stmt_sortir(&mut self, stmt: &Stmt) {
        self.set_early_exit(Some(EarlyExit::Sortir));
    }

    fn visit_stmt_deffn(&mut self, stmt: &Stmt) {
        if let Stmt::DefFn {
            name,
            params,
            body,
            return_type,
        } = stmt
        {
            let return_type = eval!(opt_type, self, return_type, "Return Func type");

            let mut params_fonc = Vec::with_capacity(params.len());
            for param in params {
                let param_type = eval!(opt_type, self, &param.static_type, "Param type");

                params_fonc.push(ASFnParam::new(
                    param.name.clone(),
                    param_type,
                    param.default_value.clone(),
                ))
            }

            let func = ASObj::asfonc(
                Some(name.clone()),
                None,
                params_fonc,
                body.clone(),
                return_type,
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
            self.set_early_exit(Some(EarlyExit::Retourner));
        }
    }

    fn visit_stmt_defstruct(&mut self, stmt: &Stmt) {
        if let Stmt::DefStruct { name, fields } = stmt {
            let mut as_fields = Vec::with_capacity(fields.len());
            for field in fields.into_iter() {
                let field_type = eval!(opt_type, self, field.static_type.clone(), "Field Type");
                as_fields.push(ASStructField {
                    name: field.name.clone(),
                    vis: field.vis.into(),
                    static_type: field_type.into(),
                    is_const: field.is_const,
                    default_value: field.default_value.clone(),
                })
            }
            let as_struct = ASObj::ASStructure {
                name: name.clone(),
                fields: as_fields,
            };
            let var = ASVar::new(name.clone(), Some(ASType::Structure), true);
            let result = self.env.declare_strict(var, as_struct);
            if result.is_err() {
                throw_err!(self, result.err().unwrap());
            }
        }
    }

    fn visit_type_lit(&mut self, t: &Type) {
        if let Type::Lit(t) = t {
            self.type_results
                .push(ASType::from_str(t.as_str()).unwrap())
        }
    }

    fn visit_type_binop(&mut self, t: &Type) {
        if let Type::BinOp { lhs, op, rhs } = t {
            let lhs_value = eval!(type, self, lhs, "Lhs de type binop");
            let rhs_value = eval!(type, self, rhs, "Rhs de type binop");
            self.type_results.push(match op {
                TypeBinOpcode::Union => ASType::union_of(lhs_value, rhs_value),
                TypeBinOpcode::Intersection => todo!(),
            });
        }
    }

    fn visit_type_opt(&mut self, t: &Type) {
        if let Type::Opt(t) = t {
            let type_val = eval!(type, self, t, "Opt type");
            self.type_results.push(ASType::optional(type_val));
        }
    }
}
