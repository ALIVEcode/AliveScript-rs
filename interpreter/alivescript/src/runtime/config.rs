use std::{borrow::Borrow, collections::HashSet, fmt::Debug, hash::Hash, str::FromStr};

use derive_builder::Builder;

use crate::{
    compiler::{CompilerOptions, obj::Function},
    runtime::err::RuntimeError,
};

#[derive(Debug, Clone)]
pub enum PermissionSet<T>
where
    T: Debug + Clone + PartialEq + Eq + Hash,
{
    Include(HashSet<T>),
    Exclude(HashSet<T>),
}

impl<T> PermissionSet<T>
where
    T: Debug + Clone + PartialEq + Eq + Hash,
{
    /// Returns true if:
    /// . self is `Include` and value *in* set
    /// . self is `Exclude` and value *not* in set
    ///
    /// Returns false if:
    /// . self is `Include` and value *not* in set
    /// . self is `Exclude` and value *in* set
    pub fn includes<Q>(&self, value: &Q) -> bool
    where
        Q: ?Sized,
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self {
            PermissionSet::Include(hash_set) => hash_set.contains(value),
            PermissionSet::Exclude(hash_set) => !hash_set.contains(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VMAction {
    ReadStdin,
    WriteStdout,

    ReadFile,
    WriteFile,

    Eval,
}

impl FromStr for VMAction {
    type Err = RuntimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lireEntreeStd" | "lireEntréeStd" => Ok(Self::ReadStdin),
            "ecrireSortieStd" | "écrireSortieStd" => Ok(Self::WriteStdout),
            "ecrireFichier" | "écrireFichier" => Ok(Self::WriteFile),
            "lireFichier" => Ok(Self::ReadFile),
            "eval" | "éval" => Ok(Self::Eval),
            _ => Err(RuntimeError::generic_err(format!(
                "Valeur de permission inconnue: '{}'",
                s
            ))),
        }
    }
}

#[derive(Debug, Default, Clone, Builder)]
pub struct VMConfig {
    #[builder(default)]
    pub allowed_modules: Option<PermissionSet<String>>,
    #[builder(setter(custom))]
    pub permissions: Option<PermissionSet<VMAction>>,
    #[builder(default)]
    pub compiler_options: CompilerOptions,
    #[builder(default)]
    pub module_searcher: Option<Function>,
}

impl VMConfigBuilder {
    pub fn include_permissions(&mut self, actions: Vec<VMAction>) -> &mut Self {
        match &mut self.permissions {
            Some(Some(p)) => match p {
                PermissionSet::Include(hash_set) => {
                    hash_set.extend(actions);
                }
                PermissionSet::Exclude(hash_set) => {
                    panic!("Already set to exclude")
                }
            },
            None => {
                self.permissions = Some(Some(PermissionSet::Include(HashSet::from_iter(actions))));
            }
            Some(None) => unreachable!(),
        }

        self
    }
    pub fn exclude_permissions(&mut self, actions: Vec<VMAction>) -> &mut Self {
        match &mut self.permissions {
            Some(Some(p)) => match p {
                PermissionSet::Exclude(hash_set) => {
                    hash_set.extend(actions);
                }
                PermissionSet::Include(hash_set) => {
                    panic!("Already set to include")
                }
            },
            None => {
                self.permissions = Some(Some(PermissionSet::Exclude(HashSet::from_iter(actions))));
            }
            Some(None) => unreachable!(),
        }

        self
    }
}
