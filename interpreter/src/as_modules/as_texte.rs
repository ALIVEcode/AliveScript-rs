use crate::{
    as_cast, as_fonction, as_mod,
    as_obj::{ASObj, ASType},
};

as_mod! {
    TEXTE_MOD,
    as_fonction! {
        maj(txt: ASType::Texte) -> ASType::Texte; {
            as_cast!(ASObj::ASTexte(txt) = txt);
            Ok(Some(ASObj::ASTexte(txt.to_uppercase())))
        }
    },
    as_fonction! {
        minus(txt: ASType::Texte) -> ASType::Texte; {
            as_cast!(ASObj::ASTexte(txt) = txt);
            Ok(Some(ASObj::ASTexte(txt.to_lowercase())))
        }
    },
    as_fonction! {
        indexDe(txt: ASType::Texte, subtxt: ASType::Texte) -> ASType::optional(ASType::Entier); {
            as_cast!(ASObj::ASTexte(txt) = txt);
            as_cast!(ASObj::ASTexte(subtxt) = subtxt);

            let maybe_i = txt.find(subtxt);
            Ok(Some(match maybe_i {
                Some(i) => ASObj::ASEntier(i as i64),
                None => ASObj::ASNul,
            }))
        }
    },
    as_fonction! {
        remplacer(txt: ASType::Texte,
                  pattern: ASType::Texte,
                  remplacement: ASType::Texte,
                  n: ASType::optional(ASType::Entier)) -> ASType::Texte;
        {
            as_cast!(ASObj::ASTexte(txt) = txt);
            as_cast!(ASObj::ASTexte(pattern) = pattern);
            as_cast!(ASObj::ASTexte(remplacement) = remplacement);
            Ok(Some(match n {
                ASObj::ASNul => ASObj::ASTexte(txt.replace(pattern, remplacement)),
                ASObj::ASEntier(n) => {
                    ASObj::ASTexte(txt.replacen(pattern, remplacement, *n as usize))
                }
                _ => unreachable!(),
            }))
        }
    },
}
