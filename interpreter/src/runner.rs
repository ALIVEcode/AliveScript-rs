use lalrpop_util::ParseError;

use std::{cell::RefCell, ops::IndexMut, path::Path, rc::Rc, str::FromStr};

use crate::{
    as_modules::ASModuleBuiltin,
    as_obj::{
        ASClasse, ASClasseField, ASClasseInst, ASDict, ASEnv, ASErreur, ASErreurType, ASFnParam,
        ASFonc, ASMethode, ASObj, ASScope, ASType, ASVar,
    },
    as_py::run_python_script,
    ast::{
        AssignVar, BinCompcode, BinLogiccode, BinOpcode, CallRust, DeclVar, Expr, FnParam, LireVar,
        Stmt, Type, TypeBinOpcode, UnaryOpcode,
    },
    call_methode,
    data::{Data, Response},
    get_err_line,
    io::InterpretorIO,
    run_script_with_runner,
    visitor::{Visitable, Visitor},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum EarlyExit {
    Retourner, // retourner d'une fonctionc
    Continuer, // remonter au début d'une boucle
    Sortir,    // sortir d'une boucle
    Erreur,    // Erreur
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ModuleType {
    AliveScript,
    Python,
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
        result.ok().unwrap()
    }};
}

pub struct Runner<'a> {
    expr_results: Vec<ASObj>,
    type_results: Vec<ASType>,
    io: &'a mut dyn InterpretorIO,
    env: ASEnv,
    early_exit: Option<EarlyExit>,
    stmt_result: Option<ASObj>,
    current_file: Option<String>,
    used_files: Vec<String>,
}

impl<'a> Runner<'a> {
    pub fn new<IO: InterpretorIO + 'a>(intepretor_io: &'a mut IO) -> Self {
        let mut new = Self {
            expr_results: vec![],
            type_results: vec![],
            io: intepretor_io,
            env: ASEnv::new(),
            early_exit: None,
            stmt_result: None,
            current_file: None,
            used_files: vec![],
        };
        ASModuleBuiltin::Builtin.load_non_custom(&None, &Some(vec!["*".into()]), &mut new.env);
        new
    }

    pub fn new_with_file<IO: InterpretorIO + 'a>(intepretor_io: &'a mut IO, file: String) -> Self {
        let mut new = Self {
            expr_results: vec![],
            type_results: vec![],
            io: intepretor_io,
            env: ASEnv::new(),
            early_exit: None,
            stmt_result: None,
            current_file: Some(file.clone()),
            used_files: vec![file],
        };
        ASModuleBuiltin::Builtin.load_non_custom(&None, &Some(vec!["*".into()]), &mut new.env);
        new
    }

    pub fn get_env_mut(&mut self) -> &mut ASEnv {
        &mut self.env
    }

    pub fn get_env(&self) -> &ASEnv {
        &self.env
    }

    pub fn send_data(&mut self, data: Data) {
        self.io.send(data);
    }

    pub fn request_data(&mut self, data: Data) -> Option<Response> {
        self.io.request(data)
    }

    pub fn push_value(&mut self, value: ASObj) {
        self.expr_results.push(value);
    }

    pub fn pop_value(&mut self) -> Option<ASObj> {
        self.expr_results.pop()
    }

    pub fn get_stmt_result(&mut self) -> Option<ASObj> {
        self.stmt_result.take()
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
            Exp => lhs.pow(rhs),
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

    pub fn clear_early_exit(&mut self) {
        self.set_early_exit(None);
    }

    pub fn remove_error_status(&mut self) {
        if self.error_thrown() {
            self.early_exit = None;
        }
    }

    fn should_early_exit(&self) -> bool {
        self.early_exit.is_some()
    }

    pub fn error_thrown(&self) -> bool {
        self.early_exit_matches(EarlyExit::Erreur)
    }

    fn early_exit_matches(&self, early_exit: EarlyExit) -> bool {
        matches!(self.early_exit, Some(reason) if reason == early_exit)
    }

    fn parse_fn_params(&mut self, params: &Vec<FnParam>) -> Option<Vec<ASFnParam>> {
        let mut params_fonc = Vec::with_capacity(params.len());
        for param in params {
            param.static_type.as_ref().map(|t| t.accept(self));

            if self.error_thrown() {
                return None;
            }

            let param_type = self.type_results.pop();

            params_fonc.push(ASFnParam::new(
                param.name.clone(),
                param_type,
                param.default_value.clone(),
            ))
        }
        Some(params_fonc)
    }

    pub(crate) fn call_obj(&mut self, func: ASObj, args: Vec<ASObj>) -> Option<ASObj> {
        let to_call = Expr::FnCall {
            func: Expr::literal(func),
            args: args.into_iter().map(|arg| Expr::literal(arg)).collect(),
        };
        to_call.accept(self);
        if self.error_thrown() {
            return None;
        }
        self.pop_value()
    }

    pub(crate) fn run_script(
        &mut self,
        script: String,
        path: Option<String>,
    ) -> Option<Rc<RefCell<ASScope>>> {
        let expr_results = self.expr_results.clone();
        let type_results = self.type_results.clone();
        let old_file = self.current_file.take();
        self.current_file = path;
        self.env.push_scope(Rc::new(RefCell::new(ASScope::new())));
        if let Err(err) = run_script_with_runner(&script, self) {
            let err_txt = match err {
                ParseError::UnrecognizedToken { token, expected } => {
                    let (line, line_num) = get_err_line(&script, token.0, token.2);
                    format!("À la ligne {} ('{}'). Jeton non reconnu: {}. Jetons valides dans cette position: {}",
                             line_num, line, token.1, expected.join(", "))
                }
                ParseError::InvalidToken { location } => todo!(),
                ParseError::UnrecognizedEof { location, expected } => todo!(),
                ParseError::ExtraToken { token } => todo!(),
                ParseError::User { error } => todo!(),
            };
            self.send_data(Data::Erreur {
                texte: format!("ErreurSyntaxe: {}", err_txt),
                ligne: 0,
            });
            return None;
        }
        if self.error_thrown() {
            return None;
        }
        self.current_file = old_file;
        self.expr_results = expr_results;
        self.type_results = type_results;
        self.stmt_result = None;
        self.env.pop_scope()
    }

    pub fn to_bool(&mut self, obj: &ASObj) -> Result<bool, ASErreurType> {
        if let Some(result) = call_methode!(obj.__bool__(), self) {
            result.map(|obj| obj.unwrap().to_bool())
        } else {
            Ok(obj.to_bool())
        }
    }
}

impl Visitor for Runner<'_> {
    fn visit_body(&mut self, stmts: &Vec<Box<Stmt>>) {
        for stmt in stmts.iter() {
            if self.should_early_exit() {
                return;
            }
            // self.expr_results.clear();
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
            self.push_value(value.clone());
        }
    }

    fn visit_expr_list(&mut self, expr: &Expr) {
        if let Expr::List(exprs) = expr {
            let mut liste = Vec::with_capacity(exprs.len());
            for expr in exprs {
                let val = eval!(expr, self, expr, "Element de liste");
                liste.push(val);
            }
            self.expr_results
                .push(ASObj::ASListe(Rc::new(RefCell::new(liste))));
        }
    }

    // fn visit_expr_paire(&mut self, expr: &Expr) {
    //     if let Expr::Paire { clef, val } = expr {
    //         // let clef_val = self.eval_expr(clef).expect("Paire clef");
    //         // let val_val = self.eval_expr(val).expect("Paire val");
    //         let clef_val = eval!(expr, self, clef, "Paire clef");
    //         let val_val = eval!(expr, self, val, "Paire val");
    //         self.push_value(ASObj::ASPaire {
    //             key: Box::new(clef_val),
    //             val: Box::new(val_val),
    //         });
    //     }
    // }

    fn visit_expr_dict(&mut self, expr: &Expr) {
        if let Expr::Dict(exprs) = expr {
            let mut dict = ASDict::default();
            for expr in exprs {
                let clef = eval!(expr, self, expr.clef, "Clef de dict");
                let val = eval!(expr, self, expr.val, "Val de dict");
                dict.insert(clef, val);
            }
            self.push_value(ASObj::ASDict(Rc::new(RefCell::new(dict))));
        }
    }

    fn visit_expr_ident(&mut self, expr: &Expr) {
        if let Expr::Ident(var_name) = expr {
            if let Some((var, val)) = self.env.get_var(var_name) {
                self.push_value(val.clone());
            } else {
                throw_err!(self, ASErreurType::new_variable_inconnue(var_name.clone()));
            }
        }
    }

    fn visit_expr_faire(&mut self, expr: &Expr) {
        let Expr::Faire(body) = expr else {
            unreachable!()
        };
        self.env.push_new_scope(ASScope::new());
        self.visit_body(body);
        self.env.pop_scope();
        if !self.should_early_exit() {
            if self.expr_results.last().is_none() {
                self.push_value(ASObj::ASNul);
            }
        } else if !self.early_exit_matches(EarlyExit::Retourner) {
            panic!("Sortie d'une fonction autrement qu'avec `retourner`")
        }
        if self.error_thrown() {
            return;
        }
        self.clear_early_exit();
    }

    fn visit_expr_accessprop(&mut self, expr: &Expr) {
        if let Expr::AccessProp { obj, prop } = expr {
            let obj_val = eval!(expr, self, obj, "AccessProp obj");

            if let Some(result) =
                call_methode!(obj_val.__getAttr__(ASObj::ASTexte(prop.clone())), self)
            {
                let result = throw_err!(?, self, result);
                self.push_value(result.unwrap());
                return;
            }
            let result = match &obj_val {
                ASObj::ASModule { name, alias, env } => {
                    let env_borrow = env.borrow();
                    let obj = env_borrow.get(prop);
                    match obj {
                        Some(obj) => obj.1.clone(),
                        None => throw_err!(
                            self,
                            ASErreurType::new_erreur_access_propriete(
                                obj_val.clone(),
                                prop.clone()
                            )
                        ),
                    }
                }
                ASObj::ASClasse(classe) => {
                    let env_borrow = classe.static_env().borrow();
                    let Some(value) = env_borrow.get_value(prop) else {
                        throw_err!(
                            self,
                            ASErreurType::new_erreur_access_propriete(
                                obj_val.clone(),
                                prop.clone()
                            )
                        );
                    };
                    value.clone()
                }
                ASObj::ASClasseInst(inst) => {
                    let env_borrow = inst.env().borrow();
                    let Some(value) = env_borrow.get_value(prop) else {
                        throw_err!(
                            self,
                            ASErreurType::new_erreur_access_propriete(
                                obj_val.clone(),
                                prop.clone()
                            )
                        );
                    };
                    value.clone()
                }
                obj => throw_err!(
                    self,
                    ASErreurType::new_erreur_access_propriete(obj.clone(), prop.clone())
                ),
            };
            self.push_value(result);
        }
    }

    fn visit_expr_slice(&mut self, expr: &Expr) {
        if let Expr::Slice { obj, slice } = expr {
            let obj_val = eval!(expr, self, obj, "Idx obj");
            let slice = eval!(expr, self, slice, "Idx idx");

            let result = match (obj_val, slice) {
                (ASObj::ASListe(lst), ASObj::ASEntier(i)) => lst.borrow()[i as usize].clone(),
                (ASObj::ASListe(lst), ASObj::ASListe(range)) => {
                    let mut lst_final = Vec::with_capacity(range.borrow().len());
                    for obj in range.borrow().iter() {
                        if let ASObj::ASEntier(i) = obj {
                            lst_final.push(lst.borrow()[*i as usize].clone());
                        } else {
                            throw_err!(
                                self,
                                ASErreurType::new_erreur_type(ASType::Entier, obj.get_type())
                            );
                        }
                    }
                    ASObj::ASListe(Rc::new(RefCell::new(lst_final)))
                }
                (ASObj::ASTexte(txt), ASObj::ASEntier(i)) => {
                    ASObj::ASTexte(txt[i as usize..i as usize + 1].into())
                }
                (ASObj::ASTexte(txt), ASObj::ASListe(range)) => {
                    let mut txt_final = String::with_capacity(range.borrow().len());
                    for obj in range.borrow().iter() {
                        if let ASObj::ASEntier(i) = obj {
                            txt_final.push(txt.chars().nth(*i as usize).unwrap());
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
                    let d = dict.borrow();
                    let el = d.get(&clef);
                    match el {
                        Some(paire) => paire.val().clone(),
                        None => throw_err!(self, ASErreurType::new_erreur_clef(clef.clone())),
                    }
                }
                _ => todo!(),
            };

            self.push_value(result);
        }
    }

    fn visit_expr_deffn(&mut self, expr: &Expr) {
        if let Expr::DefFn(f) = expr {
            let return_type = eval!(opt_type, self, f.return_type(), "Return Func type");

            let mut params_fonc = Vec::with_capacity(f.params().len());
            for param in f.params() {
                let param_type = eval!(opt_type, self, &param.static_type, "Param type");

                params_fonc.push(ASFnParam::new(
                    param.name.clone(),
                    param_type,
                    param.default_value.clone(),
                ))
            }

            let func = Rc::new(ASFonc::new(
                f.name().as_ref().cloned(),
                f.docs().clone(),
                params_fonc,
                f.body().clone(),
                return_type.into(),
                self.env.clone(),
            ));

            self.push_value(ASObj::ASFonc(func));
        }
    }

    fn visit_expr_fncall(&mut self, expr: &Expr) {
        let Expr::FnCall { func, args } = expr else {
            unreachable!()
        };

        let expr = eval!(expr, self, func, "FnCall Fonc");
        match expr {
            ASObj::ASFonc(f) => {
                let mut env = ASScope::new();
                if f.params().len() < args.len() {
                    throw_err!(
                        self,
                        ASErreurType::new_erreur_nb_args(
                            f.name().clone(),
                            f.params().len(),
                            args.len()
                        )
                    );
                }
                let mut args_iter = args.iter();

                // Set params dans env local de la fonction
                for param in f.params().iter() {
                    let arg = args_iter.next();
                    if let Some(arg_expr) = arg {
                        let arg_val = eval!(expr, self, arg_expr, "FnCall arg");
                        if !ASType::type_match(&param.static_type, &arg_val.get_type()) {
                            throw_err!(
                                self,
                                ASErreurType::ErreurTypeAppel {
                                    func_name: f.name().clone(),
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
                        throw_err!(
                            self,
                            ASErreurType::new_erreur_nb_args(
                                f.name().clone(),
                                f.params().len(),
                                args.len()
                            )
                        );
                    }
                }

                // Exec Body
                let old_env = self.env.clone();
                self.env = f.env().clone();
                self.env.push_scope(Rc::new(RefCell::new(env)));

                self.visit_body(f.body());

                // Clean up
                self.env = old_env;

                if self.error_thrown() {
                    return;
                }

                // Fonction termine sans de "retourner" ou "error"
                if !self.should_early_exit() {
                    if self.expr_results.last().is_none()
                        && !ASType::type_match(f.return_type(), &ASType::Rien)
                    {
                        self.push_value(ASObj::ASNul);
                    }
                } else if !self.early_exit_matches(EarlyExit::Retourner) {
                    panic!("Sortie d'une fonction autrement qu'avec `retourner`")
                }

                // FIXME: on clear avant le check
                self.clear_early_exit();

                // Retourner
                let type_returned = if let Some(returned_value) = self.expr_results.last() {
                    returned_value.get_type()
                } else {
                    ASType::Rien
                };

                if !ASType::type_match(f.return_type(), &type_returned) {
                    throw_err!(
                        self,
                        ASErreurType::ErreurTypeRetour {
                            type_attendu: f.return_type().clone(),
                            type_obtenu: type_returned,
                        }
                    );
                }
            }
            ASObj::ASClasse(classe) => {
                let env = Rc::new(RefCell::new(ASScope::new()));

                for field in classe.fields() {
                    let field_value = eval!(
                        opt_expr,
                        self,
                        field.default_value.clone(),
                        "Classe default value"
                    );
                    let field_var = ASVar::new(
                        field.name.clone(),
                        Some(field.static_type.clone()),
                        field.is_const,
                    );
                    if let Some(value) = field_value {
                        env.borrow_mut().declare(field_var, value);
                    } else {
                        env.borrow_mut().declare(field_var, ASObj::ASNoValue);
                    }
                }

                let inst = Rc::new(ASClasseInst::new(Rc::clone(&classe), Rc::clone(&env)));

                for fonction in classe.methods() {
                    env.borrow_mut().declare(
                        ASVar::new(
                            fonction.name().as_ref().unwrap().clone(),
                            Some(ASType::Fonction),
                            true,
                        ),
                        ASObj::ASMethode(Rc::new(ASMethode::new(
                            Rc::clone(fonction),
                            Rc::clone(&inst),
                        ))),
                    );
                }

                let mut init_args = vec![Expr::literal(ASObj::ASClasseInst(Rc::clone(&inst)))];
                init_args.extend(args.clone());

                if let Some(init) = classe.init() {
                    let to_call = Expr::FnCall {
                        func: Expr::literal(ASObj::ASFonc(Rc::clone(init))),
                        args: init_args,
                    };
                    self.visit_expr_fncall(&to_call);
                    if self.error_thrown() {
                        return;
                    }
                } else {
                    if args.len() > 0 {
                        throw_err!(
                            self,
                            ASErreurType::new_erreur_nb_args(
                                Some(format!("{}@init", classe.name().clone())),
                                0,
                                args.len()
                            )
                        );
                    }
                }
                self.push_value(ASObj::ASClasseInst(Rc::clone(&inst)));
            }

            ASObj::ASMethode(methode) => {
                let inst = Rc::clone(methode.inst());

                let mut methode_args = vec![Expr::literal(ASObj::ASClasseInst(Rc::clone(&inst)))];
                methode_args.extend(args.clone());

                let to_call = Expr::FnCall {
                    func: Expr::literal(ASObj::ASFonc(methode.func().clone())),
                    args: methode_args,
                };

                self.visit_expr_fncall(&to_call);
                if self.error_thrown() {
                    return;
                }
            }
            _ => {
                panic!("Impossible d'appeler '{:?}'", expr);
            }
        }
    }

    /* fn visit_expr_classe_init(&mut self, expr: &Expr) {
        let Expr::ClasseInst { classe, fields } = expr else {
            unreachable!();
        };

        let classe_parent = eval!(expr, self, classe, "Classe");
        let ASObj::ASClasse(classe) = &classe_parent else {
            throw_err!(
                self,
                ASErreurType::new_erreur_type(ASType::Classe, classe_parent.get_type())
            )
        };
        let mut env = ASScope::new();

        for field in classe.fields() {
            let field_value = eval!(
                opt_expr,
                self,
                field.default_value.clone(),
                "Classe default value"
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
            let ASObj::ASPaire { key, val } = eval!(expr, self, field_expr, "Classe field") else {
                unreachable!()
            };
            let ASObj::ASTexte(key) = *key else {
                unreachable!()
            };

            throw_err!(?, self, env.assign_type_strict(&key, *val));
        }
    } */

    fn visit_expr_callrust(&mut self, expr: &Expr) {
        let Expr::CallRust(CallRust(proc)) = expr else {
            return;
        };

        let result = proc(self);
        match result {
            Ok(Some(value)) => self.push_value(value),
            Ok(None) => {}
            Err(err) => throw_err!(self, err),
        }
    }

    fn visit_expr_suite(&mut self, expr: &Expr) {
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
            self.expr_results
                .push(ASObj::ASListe(Rc::new(RefCell::new(range))));
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
            self.expr_results
                .push(ASObj::ASListe(Rc::new(RefCell::new(range))));
        } else if start_val == end_val && *is_incl {
            self.push_value(ASObj::ASListe(Rc::new(RefCell::new(vec![
                ASObj::ASEntier(start_val),
            ]))));
        } else {
            self.push_value(ASObj::ASListe(Rc::new(RefCell::new(vec![]))));
        }
    }

    fn visit_expr_unaryop(&mut self, expr: &Expr) {
        if let Expr::UnaryOp { expr, op } = expr {
            use UnaryOpcode::*;

            let value = eval!(expr, self, expr, "Lhs de binop");

            let result = match op {
                Pas => ASObj::ASBooleen(!throw_err!(?, self, self.to_bool(&value))),
                Negate => todo!(),
            };

            self.push_value(result);
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
            self.push_value(result);
        }
    }

    fn visit_expr_binlogic(&mut self, expr: &Expr) {
        if let Expr::BinLogic { lhs, op, rhs } = expr {
            let lhs_value = eval!(expr, self, lhs, "Lhs de bin logique");

            use BinLogiccode::*;
            let result = match op {
                Et => {
                    if !throw_err!(?, self, self.to_bool(&lhs_value)) {
                        lhs_value
                    } else {
                        eval!(expr, self, rhs, "Rhs de bin logique")
                    }
                }
                Ou => {
                    if throw_err!(?, self, self.to_bool(&lhs_value)) {
                        lhs_value
                    } else {
                        eval!(expr, self, rhs, "Rhs de bin logique")
                    }
                }
            };

            self.push_value(result);
        }
    }

    fn visit_stmt_expr(&mut self, stmt: &Stmt) {
        if let Stmt::Expr(expr) = stmt {
            expr.accept(self);
            self.stmt_result = self.expr_results.pop();
        }
    }

    fn visit_stmt_afficher(&mut self, stmt: &Stmt) {
        let Stmt::Afficher(exprs) = stmt else { return };

        let mut values = Vec::with_capacity(exprs.len());
        for expr in exprs {
            let value = eval!(expr, self, expr, "Afficher prend un argument");
            if let Some(result) = call_methode!(value.__texte__(), self) {
                let result = throw_err!(?, self, result);
                values.push(result.unwrap().to_string());
            } else {
                values.push(value.to_string());
            }
        }
        let string = values.join(" ");
        self.send_data(Data::Afficher(string));
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
                            Err(ASErreurType::new_erreur_type(ASType::Texte, obj.get_type()))
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
            is_path,
        } = stmt
        {
            if *is_path {
                let p = Path::new(module);
                let module_type = match p.extension() {
                    Some(ext) if ext == "as" => ModuleType::AliveScript,
                    Some(ext) if ext == "py" => ModuleType::Python,
                    _ => {
                        throw_err!(
                            self,
                            ASErreurType::new_erreur_module_invalide(module.clone())
                        );
                    }
                };
                let module_name = p.file_stem().map(|s| s.to_str().unwrap().to_owned());
                if module_name.is_none() {
                    throw_err!(
                        self,
                        ASErreurType::new_erreur_module_invalide(module.clone())
                    );
                }

                let mut module = module.clone();
                // Si le module est relatif, on le rend absolu en ajoutant le
                // chemin du fichier courant
                if self.current_file.is_some() {
                    let current_path = Path::new(self.current_file.as_ref().unwrap());
                    module = current_path
                        .parent()
                        .unwrap()
                        .join(module)
                        .to_str()
                        .unwrap()
                        .to_owned();
                }
                // Si le module est déjà utilisé, on ne le réutilise pas
                if self.used_files.contains(&module) {
                    return;
                }
                let script = self.request_data(Data::GetFichier(module.clone()));
                let Some(Response::Text(script)) = script else {
                    throw_err!(
                        self,
                        ASErreurType::new_erreur_fichier_introuvable(module.clone())
                    );
                };
                self.used_files.push(module.clone());

                let mod_scope = Rc::clone(&match module_type {
                    ModuleType::AliveScript => self.run_script(script, Some(module)).unwrap(),
                    ModuleType::Python => run_python_script(script).unwrap(),
                });

                ASModuleBuiltin::load_from_scope(
                    mod_scope,
                    module_name.unwrap().to_owned(),
                    alias,
                    vars,
                    &mut self.env,
                )
            } else {
                let module_path = module.clone() + ".as";
                let script = self.request_data(Data::GetFichier(module_path.clone()));
                self.used_files.push(module.clone());

                let Some(Response::Text(script)) = script else {
                    // Si le fichier n'existe pas, on essaye de charger un module builtin
                    let module = throw_err!(?, self, ASModuleBuiltin::try_from(module.as_str()));
                    throw_err!(?, self, module.load(alias, vars, self));
                    return;
                };

                let mod_scope = Rc::clone(&self.run_script(script, Some(module.clone())).unwrap());

                ASModuleBuiltin::load_from_scope(
                    mod_scope,
                    module.clone(),
                    alias,
                    vars,
                    &mut self.env,
                )
            }
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
                        throw_err!(?, self, self.env.assign(name, value));
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
                        (ASListe(lst), ASEntier(i)) => {
                            *lst.borrow_mut().index_mut(i as usize) = value;
                        }
                        (ASDict(d), obj) => {
                            let mut d = d.borrow_mut();
                            d.insert(obj, value);
                        }
                        _ => todo!(),
                    }
                }
                AssignVar::AccessProp { obj, prop } => {
                    let var_val = eval!(expr, self, obj, "Assign AccessProp Obj");
                    match var_val {
                        ASObj::ASClasseInst(inst) => {
                            throw_err!(?, self, inst.env().borrow_mut().assign(prop, value));
                        }
                        ASObj::ASClasse(classe) => {
                            throw_err!(?, self, classe.static_env().borrow_mut().assign(prop, value));
                        }
                        _ => todo!(),
                    }
                }
                AssignVar::Decl(_) => todo!(),
            }
        }
    }

    fn visit_stmt_opassign(&mut self, stmt: &Stmt) {
        let Stmt::OpAssign {
            var: assign_var,
            op,
            val,
        } = stmt
        else {
            unreachable!()
        };

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

                    throw_err!(?, self, self.env.assign(name, value));
                } else {
                    throw_err!(self, ASErreurType::new_variable_inconnue(name.clone()))
                }
            }
            AssignVar::Slice { obj, slice } => {
                use ASObj::*;

                let var_val = eval!(expr, self, obj, "Assign Slice Obj");
                let slice_val = eval!(expr, self, slice, "Assign Slice Slice");
                match (var_val, slice_val) {
                    (ASListe(lst), ASEntier(i)) => {
                        let old_value = lst.borrow()[i as usize].clone();
                        *lst.borrow_mut().index_mut(i as usize) =
                            Runner::<'_>::do_op(old_value, op, value);
                    }
                    _ => todo!(),
                }
            }
            AssignVar::AccessProp { obj, prop } => {
                let var_val = eval!(expr, self, obj, "Assign AccessProp Obj");

                match &var_val {
                    ASObj::ASClasseInst(inst) => {
                        let mut env_borrow = inst.env().borrow_mut();
                        let Some(old_value) = env_borrow.get_value(prop) else {
                            throw_err!(
                                self,
                                ASErreurType::new_erreur_access_propriete(
                                    var_val.clone(),
                                    prop.clone()
                                )
                            );
                        };
                        let new_value = Runner::<'_>::do_op(old_value.clone(), op, value);
                        throw_err!(?, self, env_borrow.assign(prop, new_value));
                    }
                    _ => todo!(),
                }
            }
            AssignVar::Decl(_) => todo!(),
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
            if throw_err!(?, self, self.to_bool(&cond_result)) {
                self.visit_body(then_br);
            } else if let Some(else_br) = else_br {
                self.visit_body(else_br);
            }
        }
    }

    fn visit_stmt_condstmt(&mut self, stmt: &Stmt) {
        if let Stmt::CondStmt { cond, then_stmt } = stmt {
            let cond_result = eval!(expr, self, cond, "CondStmt cond");
            if throw_err!(?, self, self.to_bool(&cond_result)) {
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
                self.env.push_scope(Rc::new(RefCell::new(ASScope::new())));
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
            let mut iter_obj = eval!(expr, self, iterable, "Pour iterable");

            if let Some(obj) = call_methode!(iter_obj.__iter__() or throw, self) {
                iter_obj = throw_err!(?, self, obj).unwrap();
            }

            let iter: Rc<RefCell<Vec<ASObj>>> = match iter_obj {
                ASObj::ASTexte(s) => Rc::new(RefCell::new(
                    s.chars().map(|c| ASObj::ASTexte(String::from(c))).collect(),
                )),
                ASObj::ASListe(ref ls) => Rc::clone(ls),
                _ => throw_err!(
                    self,
                    ASErreurType::new_erreur_type(ASType::iterable(), iter_obj.get_type())
                ),
            };

            let DeclVar::Var {
                name,
                static_type,
                is_const,
            } = var
            else {
                todo!()
            };
            let static_type = eval!(opt_type, self, static_type, "Pour var type");

            let var = ASVar::new(name.clone(), static_type.clone(), *is_const);

            for val in iter.borrow().iter() {
                self.env
                    .push_new_scope(ASScope::from(vec![(var.clone(), val.clone())]));
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
            let cond_obj = eval!(expr, self, cond, "Si cond");
            while throw_err!(?, self, self.to_bool(&cond_obj)) {
                self.env.push_new_scope(ASScope::new());
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

    fn visit_stmt_retourner(&mut self, stmt: &Stmt) {
        if let Stmt::Retourner(expr) = stmt {
            if let Some(val_expr) = expr {
                // retourner valeur
                let result = eval!(expr, self, val_expr, "retourner");
                self.push_value(result);
            } else {
                // retourner
                self.push_value(ASObj::ASNul);
            }
            self.set_early_exit(Some(EarlyExit::Retourner));
        }
    }

    fn visit_stmt_deffn(&mut self, stmt: &Stmt) {
        if let Stmt::DefFn(f) = stmt {
            let return_type = eval!(opt_type, self, f.return_type(), "Return Func type");

            let mut params_fonc = Vec::with_capacity(f.params().len());
            for param in f.params() {
                let param_type = eval!(opt_type, self, &param.static_type, "Param type");

                params_fonc.push(ASFnParam::new(
                    param.name.clone(),
                    param_type,
                    param.default_value.clone(),
                ))
            }

            let func = Rc::new(ASFonc::new(
                f.name().as_ref().cloned(),
                f.docs().clone(),
                params_fonc,
                f.body().clone(),
                return_type.into(),
                self.env.clone(),
            ));

            let func_var = ASVar::new(
                f.name().as_ref().unwrap().clone(),
                Some(ASType::Fonction),
                true,
            );

            self.env.declare(func_var, ASObj::ASFonc(func));
        }
    }

    fn visit_stmt_defclasse(&mut self, stmt: &Stmt) {
        if let Stmt::DefClasse {
            name,
            docs,
            fields,
            init,
            methods,
        } = stmt
        {
            let mut static_env = ASScope::new();
            let mut static_fields = vec![];
            let mut as_fields = Vec::with_capacity(fields.len());
            for field in fields.into_iter() {
                let field_type = eval!(opt_type, self, field.static_type.clone(), "Field Type");
                let as_field = ASClasseField {
                    name: field.name.clone(),
                    vis: field.vis.into(),
                    static_type: field_type.into(),
                    is_const: field.is_const,
                    default_value: field.default_value.clone(),
                };
                if *field.is_static() {
                    let Some(value_expr) = field.default_value() else {
                        throw_err!(
                            self,
                            ASErreurType::new_erreur_valeur(
                                Some(
                                    "Les champs statiques doivent avoir une valeur par défaut."
                                        .into()
                                ),
                                ASObj::ASNoValue
                            )
                        );
                    };
                    static_fields.push((field.name.clone(), value_expr));
                    static_env.declare(
                        ASVar::new(
                            field.name.clone(),
                            Some(as_field.static_type.clone()),
                            field.is_const,
                        ),
                        ASObj::ASNoValue,
                    );
                } else {
                    as_fields.push(as_field);
                }
            }
            let mut as_methods = Vec::with_capacity(methods.len());
            for method in methods.into_iter() {
                let method_params = self.parse_fn_params(method.params());
                let Some(method_params) = method_params else {
                    return;
                };
                let return_type = eval!(opt_type, self, method.return_type(), "Method return type");

                let method_params = {
                    if *method.is_static() {
                        method_params
                    } else {
                        let mut method_params_inst = vec![ASFnParam::new(
                            String::from("inst"),
                            Some(ASType::Objet(name.clone())),
                            None,
                        )];
                        method_params_inst.extend(method_params);
                        method_params_inst
                    }
                };

                let as_method = Rc::new(ASFonc::new(
                    method.name().clone(),
                    method.docs().clone(),
                    method_params,
                    method.body().clone(),
                    return_type.into(),
                    self.env.clone(),
                ));

                if *method.is_static() {
                    static_env.declare(
                        ASVar::new(
                            method.name().as_ref().unwrap().clone(),
                            Some(ASType::Fonction),
                            true,
                        ),
                        ASObj::ASFonc(Rc::clone(&as_method)),
                    );
                } else {
                    as_methods.push(as_method);
                }
            }
            let static_env = Rc::new(RefCell::new(static_env));
            let as_classe = ASObj::ASClasse(Rc::new(ASClasse::new(
                name.clone(),
                docs.clone(),
                as_fields,
                {
                    if let Some(init) = init {
                        let init_params = self.parse_fn_params(init.params());
                        if init_params.is_none() {
                            return;
                        }
                        let return_type =
                            eval!(opt_type, self, init.return_type(), "Init return type");

                        let mut init_params_inst = vec![ASFnParam::new(
                            String::from("inst"),
                            Some(ASType::Objet(name.clone())),
                            None,
                        )];
                        init_params_inst.extend(init_params.unwrap());

                        Some(Rc::new(ASFonc::new(
                            Some(format!("{}@init", name)),
                            init.docs().clone(),
                            init_params_inst,
                            init.body().clone(),
                            return_type.into(),
                            self.env.clone(),
                        )))
                    } else {
                        None
                    }
                },
                as_methods,
                Rc::clone(&static_env),
            )));

            throw_err!(?, self, self.env.declare_strict(
                ASVar::new(name.clone(), Some(ASType::Classe), true),
                as_classe,
            ));

            for (name, value_expr) in static_fields {
                let field_value = eval!(expr, self, value_expr, "Classe static value");
                let mut env_borrow = static_env.borrow_mut();
                throw_err!(?, self, env_borrow.assign(&name, field_value));
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
