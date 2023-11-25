use crate::{
    as_fonction, as_mod,
    as_obj::{ASObj, ASType},
    unpack_as,
};

as_mod! {
    TEXTE_MOD,
    as_fonction! {
        maj(txt: ASType::Texte) -> ASType::Texte; {
            unpack_as!(ASObj::ASTexte(txt) = txt);
            Ok(Some(ASObj::ASTexte(txt.to_uppercase())))
        }
    },
    as_fonction! {
        minus(txt: ASType::Texte) -> ASType::Texte; {
            unpack_as!(ASObj::ASTexte(txt) = txt);
            Ok(Some(ASObj::ASTexte(txt.to_lowercase())))
        }
    },
    as_fonction! {
        indexDe(txt: ASType::Texte, subtxt: ASType::Texte) -> ASType::optional(ASType::Entier); {
            unpack_as!(ASObj::ASTexte(txt) = txt);
            unpack_as!(ASObj::ASTexte(subtxt) = subtxt);

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
            unpack_as!(ASObj::ASTexte(txt) = txt);
            unpack_as!(ASObj::ASTexte(pattern) = pattern);
            unpack_as!(ASObj::ASTexte(remplacement) = remplacement);
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
