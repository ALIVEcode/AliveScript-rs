use std::fmt::Display;

use derive_new::new;

use crate::{
    as_obj::{ASObj, ASType},
    ast::Type,
    data::Data,
};

#[derive(Debug, PartialEq, Clone, new)]
pub enum ASErreurType {
    VariableInconnue {
        var_name: String,
    },
    TypeInconnu {
        type_name: String,
    },
    ErreurVariableRedeclaree {
        var_name: String,
    },
    AffectationConstante {
        var_name: String,
    },
    ErreurNomType {
        bad_type: Box<Type>,
    },
    ErreurType {
        type_attendu: ASType,
        type_obtenu: ASType,
    },
    ErreurTypeRetour {
        nom_fonc: String,
        type_attendu: ASType,
        type_obtenu: ASType,
    },
    ErreurConversionType {
        type_cible: ASType,
        texte: String,
    },
    ErreurTypeAppel {
        func_name: Option<String>,
        param_name: String,
        type_attendu: ASType,
        type_obtenu: ASType,
    },
    ErreurNbArgs {
        func_name: Option<String>,
        nb_attendu: usize,
        nb_obtenu: usize,
    },
    ErreurOperation {
        op: String,
        lhs_type: ASType,
        rhs_type: ASType,
    },
    ErreurClef {
        mauvaise_clef: ASObj,
    },
    ErreurIndex {
        mauvais_index: i64,
        len: usize,
    },
    ErreurAccessPropriete {
        obj: ASObj,
        prop: String,
    },
    ErreurProprietePasInit {
        obj: ASObj,
        prop: String,
    },
    ErreurSuiteInvalide {
        start: ASObj,
        end: ASObj,
        step: ASObj,
    },
    ErreurValeur {
        raison: Option<String>,
        valeur: ASObj,
    },
    ErreurAffirmation {
        test: String,
        attendu: ASObj,
        obtenu: ASObj,
    },
    ErreurFichierIntrouvable {
        fichier: String,
    },
    ErreurModuleInvalide {
        module: String,
    },
    Erreur {
        nom: Option<String>,
        msg: String,
    },
}

impl ASErreurType {
    pub const fn error_name(&self) -> &'static str {
        match self {
            ASErreurType::VariableInconnue { .. } => "VariableInconnue",
            ASErreurType::TypeInconnu { .. } => "TypeInconnu",
            ASErreurType::ErreurVariableRedeclaree { .. } => "ErreurVariableRedeclaree",
            ASErreurType::AffectationConstante { .. } => "AffectationConstante",
            ASErreurType::ErreurType { .. } => "ErreurType",
            ASErreurType::ErreurTypeRetour { .. } => "ErreurTypeRetour",
            ASErreurType::ErreurConversionType { .. } => "ErreurConversionType",
            ASErreurType::ErreurTypeAppel { .. } => "ErreurTypeAppel",
            ASErreurType::ErreurOperation { .. } => "ErreurOperation",
            ASErreurType::ErreurClef { .. } => "ErreurClef",
            ASErreurType::ErreurIndex { .. } => "ErreurIndex",
            ASErreurType::ErreurAccessPropriete { .. } => "ErreurAccessPropriete",
            ASErreurType::ErreurProprietePasInit { .. } => "ErreurProprietePasInit",
            ASErreurType::ErreurSuiteInvalide { .. } => "SuiteInvalide",
            ASErreurType::ErreurValeur { .. } => "ErreurValeur",
            ASErreurType::ErreurAffirmation { .. } => "ErreurAffirmation",
            ASErreurType::ErreurNbArgs { .. } => "ErreurNbArgs",
            ASErreurType::ErreurFichierIntrouvable { .. } => "ErreurFichierIntrouvable",
            ASErreurType::ErreurModuleInvalide { .. } => "ErreurModuleInvalide",
            ASErreurType::ErreurNomType { .. } => "ErreurNomType",
            ASErreurType::Erreur { .. } => "Erreur",
        }
    }
}

impl Display for ASErreurType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ASErreurType::*;

        let to_string = match self {
            TypeInconnu { type_name } => format!("Type inconnu '{}'", type_name),
            VariableInconnue { var_name } => format!("Variable inconnue '{}'", var_name),
            ErreurVariableRedeclaree { var_name } => {
                format!("Variable '{}' déjà déclarée", var_name)
            }

            AffectationConstante { var_name } => format!("Impossible de changer la valeur d'une constante: '{}'", var_name),

            ErreurConversionType { type_cible, texte } => format!("Impossible de convertir \"{}\" en {}", texte, type_cible),

            ErreurValeur { raison, valeur } => format!("Valeur invalide: {}. {}", valeur, raison.clone().unwrap_or_default()),

            ErreurType {
                type_obtenu,
                type_attendu,
            } => format!(
                "Erreur de type. Type attendu: '{}', type obtenu: '{}'",
                type_attendu, type_obtenu,
            ),

            ErreurNbArgs {
                func_name,
                nb_attendu,
                nb_obtenu,
            } => format!(
                "Nombre d'arguments invalide pour la fonction '{}'. Attendu: {}, obtenu: {}",
                func_name.as_ref().unwrap_or(&"<sans-nom>".to_string()),
                nb_attendu,
                nb_obtenu,
            ),

            ErreurTypeRetour {
                nom_fonc,
                type_obtenu,
                type_attendu,
            } => format!(
                "Mauvais type de retour pour {}. Attendu: '{}', Obtenu: '{}'",
                nom_fonc, type_attendu, type_obtenu
            ),

            ErreurTypeAppel {
                func_name,
                param_name,
                type_obtenu,
                type_attendu,
            } => format!(
                "Dans la fonction {}: Type de l'argument invalide pour le paramètre '{}'. Attendu: '{}', obtenu: '{}'",
                func_name.as_ref().unwrap_or(&"<sans-nom>".to_string()), 
                param_name,
                type_attendu,
                type_obtenu,
            ),

            ErreurOperation {
                op,
                lhs_type,
                rhs_type,
            } => format!(
                 "Opération '{}' non définie pour les valeurs de type '{}' et de type '{}'",
                 op,
                 lhs_type,
                 rhs_type,
            ),

            ErreurClef { mauvaise_clef } => format!("La clef {} n'est pas dans le dictionnaire", mauvaise_clef.repr()),

            ErreurIndex { mauvais_index, len } => format!("Index {} invalide, car la longueur est {}", mauvais_index, len),

            ErreurAccessPropriete { obj, prop } => format!("La propriété {} n'existe pas dans {}", prop, obj),

            ErreurProprietePasInit { obj, prop } => format!("La propriété {} n'est pas initialisé dans {}", prop, obj),

            ErreurSuiteInvalide { start, end, step } => {
                format!("Suite invalide: {} .. {} bond {}", start, end, step)
            }

            ErreurAffirmation { attendu, obtenu, test } => {
                format!("Affirmation échouée pour le test `{}`. Résultat attendu: '{}'. Résultat obtenu: '{}'.",
                        test,
                        attendu,
                        obtenu)
            }

            ErreurFichierIntrouvable { fichier } => format!("Fichier introuvable: {}", fichier),
            ErreurModuleInvalide { module } => format!("Module introuvable: {}", module),

            ErreurNomType { bad_type } => format!("Mauvais type: {:?}", bad_type),

            Erreur { nom, msg } => msg.clone(),
        };

        write!(f, "{}: {}", self.error_name(), to_string)
    }
}

#[derive(Debug, PartialEq, Clone, new)]
pub struct ASErreur {
    err_type: ASErreurType,
    ligne: usize,
    file: Option<String>,
}

impl Into<Data> for ASErreur {
    fn into(self) -> Data {
        Data::Erreur {
            texte: format!(
                "{}{}",
                self.file
                    .map(|f| format!("Dans {}: ", f))
                    .unwrap_or_default(),
                self.err_type.to_string()
            ),
            ligne: self.ligne,
        }
    }
}
