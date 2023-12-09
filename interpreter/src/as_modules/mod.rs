mod as_builtin;
mod as_inspecte;
mod as_liste;
mod as_math;
mod as_temps;
mod as_tests;
mod as_texte;
mod fonction_macro;

use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::as_obj::{ASEnv, ASErreurType, ASObj, ASScope, ASType, ASVar};
use crate::runner::Runner;

use self::as_builtin::BUILTIN_MOD;
use self::as_inspecte::INSPECTE_MOD;
use self::as_liste::LISTE_MOD;
use self::as_math::MATH_MOD;
use self::as_temps::TEMPS_MOD;
use self::as_tests::TEST_MOD;
use self::as_texte::TEXTE_MOD;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum ASModuleBuiltin {
    Builtin,
    Liste,
    Texte,
    Math,
    Temps,
    Voiture,
    Test,
    Inspecte,
    Custom(String),
}

const AS_MODULES: Lazy<HashMap<ASModuleBuiltin, Rc<RefCell<ASScope>>>> = Lazy::new(|| {
    let mut modules = HashMap::new();
    modules.insert(ASModuleBuiltin::Builtin, Rc::clone(&*BUILTIN_MOD));
    modules.insert(ASModuleBuiltin::Math, Rc::clone(&*MATH_MOD));
    modules.insert(ASModuleBuiltin::Liste, Rc::clone(&*LISTE_MOD));
    modules.insert(ASModuleBuiltin::Texte, Rc::clone(&*TEXTE_MOD));
    modules.insert(ASModuleBuiltin::Temps, Rc::clone(&*TEMPS_MOD));
    modules.insert(ASModuleBuiltin::Test, Rc::clone(&*TEST_MOD));
    modules.insert(ASModuleBuiltin::Inspecte, Rc::clone(&*INSPECTE_MOD));
    modules
});

static STDLIB_PATH: Lazy<String> = Lazy::new(|| {
    std::path::Path::new(env!("OUT_DIR"))
        .join("stdlib")
        .to_str()
        .unwrap()
        .into()
});

impl ASModuleBuiltin {
    pub fn is_custom(&self) -> bool {
        match self {
            ASModuleBuiltin::Custom(_) => true,
            _ => false,
        }
    }

    fn try_load_custom_mod(mod_name: &String) -> Option<String> {
        let mod_file = format!("{}/{}.as", *STDLIB_PATH, mod_name);
        let mod_path = std::path::Path::new(&mod_file);
        if mod_path.exists() {
            let mod_content = std::fs::read_to_string(mod_path);
            match mod_content {
                Ok(content) => Some(content),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn load(
        &self,
        alias: &Option<String>,
        vars: &Option<Vec<String>>,
        runner: &mut Runner,
    ) -> Result<(), ASErreurType> {
        match self {
            Self::Custom(module) => {
                let Some(script) = ASModuleBuiltin::try_load_custom_mod(module) else {
                    return Err(ASErreurType::new_erreur_module_invalide(module.into()));
                };

                let mod_scope =
                    Rc::clone(&runner.run_script(script, Some(module.clone())).unwrap());

                ASModuleBuiltin::load_from_scope(
                    mod_scope,
                    module.clone(),
                    alias,
                    vars,
                    runner.get_env_mut(),
                );
            }
            _ => {
                let env = runner.get_env_mut();
                let mod_scope = AS_MODULES;
                let mod_scope = mod_scope.get(self).expect("Module that exists");
                ASModuleBuiltin::load_from_scope(
                    Rc::clone(&mod_scope),
                    self.name(),
                    alias,
                    vars,
                    env,
                );
            }
        }
        Ok(())
    }

    pub fn load_non_custom(
        &self,
        alias: &Option<String>,
        vars: &Option<Vec<String>>,
        env: &mut ASEnv,
    ) {
        if self.is_custom() {
            return;
        }
        let mod_scope = AS_MODULES;
        let mod_scope = mod_scope.get(self).expect("Module that exists");
        ASModuleBuiltin::load_from_scope(Rc::clone(&mod_scope), self.name(), alias, vars, env);
    }

    pub fn name(&self) -> String {
        match self {
            ASModuleBuiltin::Builtin => "builtin",
            ASModuleBuiltin::Liste => "Liste",
            ASModuleBuiltin::Texte => "Texte",
            ASModuleBuiltin::Math => "Math",
            ASModuleBuiltin::Temps => "Temps",
            ASModuleBuiltin::Voiture => "Voiture",
            ASModuleBuiltin::Test => "Test",
            ASModuleBuiltin::Inspecte => "Inspecte",
            ASModuleBuiltin::Custom(name) => name,
        }
        .into()
    }

    pub fn load_from_scope(
        mod_scope: Rc<RefCell<ASScope>>,
        name: String,
        alias: &Option<String>,
        vars: &Option<Vec<String>>,
        env: &mut ASEnv,
    ) {
        match alias {
            // Some() => mod_scope.iter().for_each(|(_name, (var, val))| {
            //     env.declare(var.clone(), val.clone());
            // }),
            Some(alias_name) => match vars.as_deref() {
                None => {
                    env.declare(
                        ASVar::new(alias_name.clone(), Some(ASType::Module), true),
                        ASObj::ASModule {
                            name,
                            alias: Some(alias_name.clone()),
                            env: Rc::clone(&mod_scope),
                        },
                    );
                }
                Some([x]) if x == "*" => {
                    let mut mod_env = ASScope::new();
                    mod_scope.borrow().iter().for_each(|(_name, (var, val))| {
                        mod_env.insert(var.clone(), val.clone());
                    });
                    env.declare(
                        ASVar::new(alias_name.clone(), Some(ASType::Module), true),
                        ASObj::ASModule {
                            name,
                            alias: Some(alias_name.clone()),
                            env: Rc::new(RefCell::new(mod_env)),
                        },
                    );
                }
                Some(used_vars) => {
                    let mut mod_env = ASScope::new();
                    used_vars.iter().for_each(|var_name| {
                        let var = mod_scope
                            .borrow()
                            .get(var_name)
                            .expect("Variable qui existe dans module.")
                            .clone();
                        mod_env.insert(var.0, var.1);
                    });
                    env.declare(
                        ASVar::new(alias_name.clone(), Some(ASType::Module), true),
                        ASObj::ASModule {
                            name,
                            alias: Some(alias_name.clone()),
                            env: Rc::new(RefCell::new(mod_env)),
                        },
                    );
                }
            },
            None => match vars.as_deref() {
                None => {
                    env.declare(
                        ASVar::new(name.clone(), Some(ASType::Module), true),
                        ASObj::ASModule {
                            name,
                            alias: None,
                            env: Rc::clone(&mod_scope),
                        },
                    );
                }
                Some([x]) if x == "*" => {
                    mod_scope.borrow().iter().for_each(|(_name, (var, val))| {
                        env.declare(var.clone(), val.clone());
                    });
                }
                Some(used_vars) => {
                    used_vars.iter().for_each(|var_name| {
                        let var = mod_scope
                            .borrow()
                            .get(var_name)
                            .expect("Variable qui existe dans module.")
                            .clone();
                        env.declare(var.0, var.1);
                    });
                }
            },
        };
    }
}

impl TryFrom<&str> for ASModuleBuiltin {
    type Error = ASErreurType;

    fn try_from(mod_name: &str) -> Result<ASModuleBuiltin, Self::Error> {
        use ASModuleBuiltin::*;

        match mod_name {
            "builtin" => Ok(Builtin),
            "Math" => Ok(Math),
            "Liste" => Ok(Liste),
            "Texte" => Ok(Texte),
            "Temps" => Ok(Temps),
            "Voiture" => Ok(Voiture),
            "Test" => Ok(Test),
            other => Ok(Custom(other.into())),
            // _ => Err(ASErreurType::new_erreur_module_invalide(mod_name.into())),
        }
    }
}
