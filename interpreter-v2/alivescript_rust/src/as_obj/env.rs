use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use crate::as_obj::{ASErreurType, ASObj, ASResult, ASType};

#[derive(Debug, Clone, PartialEq)]
pub struct ASScope(pub(crate) HashMap<String, (ASVar, ASObj)>);

impl ASScope {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from(vars: Vec<(ASVar, ASObj)>) -> Self {
        Self(HashMap::from_iter(
            vars.into_iter()
                .map(|(var, val)| (var.get_name().clone(), (var, val))),
        ))
    }

    pub fn get(&self, var_name: &String) -> Option<&(ASVar, ASObj)> {
        self.0.get(var_name)
    }

    pub fn get_value(&self, var_name: &String) -> Option<&ASObj> {
        self.0.get(var_name).map(|(_, val)| val)
    }

    pub fn insert(&mut self, var: ASVar, val: ASObj) -> Option<(ASVar, ASObj)> {
        self.0.insert(var.get_name().clone(), (var, val))
    }

    pub fn declare(&mut self, var: ASVar, val: ASObj) -> Option<(ASVar, ASObj)> {
        self.insert(var, val)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, (ASVar, ASObj)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> std::collections::hash_map::IntoIter<String, (ASVar, ASObj)> {
        self.0.into_iter()
    }

    pub fn assign_force(&mut self, var_name: &String, val: ASObj) -> Option<(ASVar, ASObj)> {
        let var = self.get(var_name).unwrap().0.clone();
        self.insert(var, val)
    }

    pub fn assign(
        &mut self,
        var_name: &String,
        val: ASObj,
    ) -> Result<Option<(ASVar, ASObj)>, ASErreurType> {
        let Some((var, old_val)) = &self.get(var_name) else {
            return Err(ASErreurType::new_variable_inconnue(var_name.clone()));
        };
        if var.is_const() && old_val != &ASObj::ASNoValue {
            Err(ASErreurType::new_affectation_constante(var_name.clone()))
        } else if !var.type_match(&val.get_type()) {
            Err(ASErreurType::new_erreur_type(
                var.get_type().clone(),
                val.get_type(),
            ))
        } else {
            Ok(self.insert(var.clone(), val))
        }
    }

    pub fn assign_type_strict(
        &mut self,
        var_name: &String,
        val: ASObj,
    ) -> Result<Option<(ASVar, ASObj)>, ASErreurType> {
        let Some(var) = &self.get(var_name) else {
            return Err(ASErreurType::new_variable_inconnue(var_name.clone()));
        };
        let var = &var.0;
        if !var.type_match(&val.get_type()) {
            Err(ASErreurType::new_erreur_type(
                var.get_type().clone(),
                val.get_type(),
            ))
        } else {
            Ok(self.insert(var.clone(), val))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ASEnv(Vec<Rc<RefCell<ASScope>>>);

impl Clone for ASEnv {
    fn clone(&self) -> Self {
        Self(self.0.iter().map(|scope| Rc::clone(scope)).collect())
    }
}

impl ASEnv {
    pub fn new() -> Self {
        Self(vec![Rc::new(RefCell::new(ASScope::new()))])
    }

    pub fn is_global(&self) -> bool {
        self.0.len() == 1
    }

    fn get_env_of_var(&mut self, var_name: &String) -> RefMut<'_, ASScope> {
        self.0
            .iter_mut()
            .rev()
            .find(|env| env.borrow().get(var_name).is_some())
            .unwrap()
            .borrow_mut()
    }

    pub fn get_curr_scope(&mut self) -> RefMut<'_, ASScope> {
        self.0.last_mut().unwrap().borrow_mut()
    }

    pub fn push_new_scope(&mut self, scope: ASScope) {
        self.0.push(Rc::new(RefCell::new(scope)));
    }

    pub fn push_scope(&mut self, scope: Rc<RefCell<ASScope>>) {
        self.0.push(scope);
    }

    pub fn pop_scope(&mut self) -> Option<Rc<RefCell<ASScope>>> {
        self.0.pop()
    }

    pub fn get_var(&self, var_name: &String) -> Option<(ASVar, ASObj)> {
        self.0
            .iter()
            .rev()
            .find_map(|env| env.borrow().get(var_name).cloned())
    }

    pub fn get_value(&self, var_name: &String) -> Option<ASObj> {
        Some(
            self.0
                .iter()
                .rev()
                .find_map(|env| env.borrow().get(var_name).cloned())?
                .1,
        )
    }

    pub fn declare(&mut self, var: ASVar, val: ASObj) -> Option<(ASVar, ASObj)> {
        self.0.last().unwrap().borrow_mut().insert(var, val)
    }

    pub fn declare_strict(&mut self, var: ASVar, val: ASObj) -> ASResult<Option<(ASVar, ASObj)>> {
        let var_name = var.get_name();
        if self
            .0
            .last()
            .unwrap()
            .borrow()
            .get(var_name)
            .is_some_and(|(v, _)| v.is_const())
        {
            Err(ASErreurType::new_affectation_constante(var_name.clone()))
        } else {
            Ok(self.0.last().unwrap().borrow_mut().insert(var, val))
        }
    }

    pub fn assign_force(&mut self, var_name: &String, val: ASObj) -> Option<(ASVar, ASObj)> {
        let mut scope = self.get_env_of_var(var_name);
        scope.assign_force(var_name, val)
    }

    pub fn assign(&mut self, var_name: &String, val: ASObj) -> ASResult<Option<(ASVar, ASObj)>> {
        let mut scope = self.get_env_of_var(var_name);
        scope.assign(var_name, val)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ASVar {
    name: String,
    static_type: ASType,
    is_const: bool,
}

impl PartialEq<String> for ASVar {
    fn eq(&self, other: &String) -> bool {
        &self.name == other
    }
}

impl ASVar {
    pub fn new(name: String, static_type: Option<ASType>, is_const: bool) -> Self {
        Self {
            name,
            static_type: static_type.into(),
            is_const,
        }
    }

    pub fn new_with_value(
        name: impl ToString,
        static_type: Option<ASType>,
        is_const: bool,
        value: ASObj,
    ) -> (Self, ASObj) {
        (Self::new(name.to_string(), static_type, is_const), value)
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_type(&self) -> &ASType {
        &self.static_type
    }

    pub fn is_const(&self) -> bool {
        self.is_const
    }

    pub fn type_match(&self, static_type: &ASType) -> bool {
        ASType::type_match(&self.static_type, static_type)
    }
}
