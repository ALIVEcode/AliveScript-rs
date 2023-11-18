mod as_builtin;
mod as_liste;
mod as_math;
mod as_temps;
mod as_texte;
mod as_tests;
mod fonction_macro;

use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};

use crate::as_obj::{ASEnv, ASObj, ASScope, ASType, ASVar};

use self::as_builtin::BUILTIN_MOD;
use self::as_liste::LISTE_MOD;
use self::as_math::MATH_MOD;
use self::as_temps::TEMPS_MOD;
use self::as_texte::TEXTE_MOD;

#[derive(Hash, Eq, PartialEq, Clone, Debug, Copy)]
pub enum ASModuleBuiltin {
    Builtin,
    Liste,
    Texte,
    Math,
    Temps,
    Voiture,
}

static AS_MODULES: Lazy<HashMap<ASModuleBuiltin, Arc<ASScope>>> = Lazy::new(|| {
    let mut modules = HashMap::new();
    modules.insert(ASModuleBuiltin::Builtin, Arc::clone(&*BUILTIN_MOD));
    modules.insert(ASModuleBuiltin::Math, Arc::clone(&*MATH_MOD));
    modules.insert(ASModuleBuiltin::Liste, Arc::clone(&*LISTE_MOD));
    modules.insert(ASModuleBuiltin::Texte, Arc::clone(&*TEXTE_MOD));
    modules.insert(ASModuleBuiltin::Temps, Arc::clone(&*TEMPS_MOD));
    modules
});

impl ASModuleBuiltin {
    pub fn load(&self, alias: &Option<String>, vars: &Option<Vec<String>>, env: &mut ASEnv) {
        let mod_scope = AS_MODULES.get(self).expect("Module that exists");

        match alias {
            // Some() => mod_scope.iter().for_each(|(_name, (var, val))| {
            //     env.declare(var.clone(), val.clone());
            // }),
            Some(alias_name) => match vars.as_deref() {
                None => {
                    env.declare(
                        ASVar::new(alias_name.clone(), Some(ASType::Module), true),
                        ASObj::ASModule {
                            env: Arc::clone(mod_scope),
                        },
                    );
                }
                Some([x]) if x == "*" => {
                    let mut mod_env = ASScope::new();
                    mod_scope.iter().for_each(|(_name, (var, val))| {
                        mod_env.insert(var.clone(), val.clone());
                    });
                    env.declare(
                        ASVar::new(alias_name.clone(), Some(ASType::Module), true),
                        ASObj::ASModule {
                            env: Arc::new(mod_env),
                        },
                    );
                }
                Some(used_vars) => {
                    let mut mod_env = ASScope::new();
                    used_vars.iter().for_each(|var_name| {
                        let var = mod_scope
                            .get(var_name)
                            .expect("Variable qui existe dans module.")
                            .clone();
                        mod_env.insert(var.0, var.1);
                    });
                    env.declare(
                        ASVar::new(alias_name.clone(), Some(ASType::Module), true),
                        ASObj::ASModule {
                            env: Arc::new(mod_env),
                        },
                    );
                }
            },
            None => match vars.as_deref() {
                None => {
                    env.declare(
                        ASVar::new(self.name(), Some(ASType::Module), true),
                        ASObj::ASModule {
                            env: Arc::clone(mod_scope),
                        },
                    );
                }
                Some([x]) if x == "*" => {
                    mod_scope.iter().for_each(|(_name, (var, val))| {
                        env.declare(var.clone(), val.clone());
                    });
                }
                Some(used_vars) => {
                    used_vars.iter().for_each(|var_name| {
                        let var = mod_scope
                            .get(var_name)
                            .expect("Variable qui existe dans module.")
                            .clone();
                        env.declare(var.0, var.1);
                    });
                }
            },
        };
    }

    pub fn name(&self) -> String {
        match self {
            ASModuleBuiltin::Builtin => "builtin",
            ASModuleBuiltin::Liste => "Liste",
            ASModuleBuiltin::Texte => "Texte",
            ASModuleBuiltin::Math => "Math",
            ASModuleBuiltin::Temps => "Temps",
            ASModuleBuiltin::Voiture => "Voiture",
        }
        .into()
    }
}

impl From<&str> for ASModuleBuiltin {
    fn from(mod_name: &str) -> ASModuleBuiltin {
        use ASModuleBuiltin::*;

        match mod_name {
            "builtin" => Builtin,
            "Math" => Math,
            "Liste" => Liste,
            "Texte" => Texte,
            "Temps" => Temps,
            "Voiture" => Voiture,
            _ => todo!(),
        }
    }
}
