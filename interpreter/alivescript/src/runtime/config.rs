use std::{borrow::Borrow, collections::HashSet, fmt::Debug, hash::Hash, str::FromStr};

use crate::{compiler::{CompilerOptions, obj::Function}, runtime::err::RuntimeError};

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
}

impl FromStr for VMAction {
    type Err = RuntimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lireEntreeStd" | "lireEntréeStd" => Ok(Self::ReadStdin),
            "ecrireSortieStd" | "écrireSortieStd" => Ok(Self::WriteStdout),
            "ecrireFichier" | "écrireFichier" => Ok(Self::WriteFile),
            "lireFichier" => Ok(Self::ReadFile),
            _ => Err(RuntimeError::generic_err(format!(
                "Valeur de permission inconnue: '{}'",
                s
            ))),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct VMConfig {
    pub allowed_modules: Option<PermissionSet<String>>,
    pub permissions: Option<PermissionSet<VMAction>>,
    pub compiler_options: CompilerOptions,
    pub module_searcher: Option<Function>,
}
