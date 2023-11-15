use logos::Logos;
use std::fmt;

#[derive(Logos, Clone, Debug, PartialEq)]
// #[logos(skip r"[ \t]+")]
pub enum Token {
    #[token("utiliser")]
    KwUtiliser,

    #[token("lire")]
    KwLire,

    #[token("afficher")]
    KwAfficher,

    // Déclarations
    #[token("var")]
    KwVar,

    #[token("const")]
    KwConst,

    #[token("fin")]
    KwFin,

    // Conditionnels
    #[token("si")]
    KwSi,

    #[token("sinon")]
    KwSinon,

    #[token("alors")]
    KwAlors,

    #[token("pas")]
    KwPas,

    #[token("non")]
    KwNon,

    #[token("et")]
    KwEt,

    #[token("ou")]
    KwOu,

    #[token("xor")]
    KwXor,

    // Boucles
    #[token("tant que")]
    KwTantQue,

    #[token("pour")]
    KwPour,

    #[token("faire")]
    KwFaire,

    #[token("repeter")]
    #[token("répéter")]
    KwRepeter,

    #[token("dans")]
    KwDans,

    #[token("bond")]
    KwBond,

    #[token("sortir")]
    KwSortir,

    #[token("continuer")]
    KwContinuer,

    // Fonctions
    #[token("fonction")]
    KwFonction,

    #[token("retourner")]
    KwRetourner,

    // Structures
    #[token("structure")]
    KwStructure,

    #[token("methode")]
    #[token("méthode")]
    KwMethode,

    // Variables
    #[regex(r"[a-zA-Zα-ζΑ-Ζ_ïöëäíóéáìòèàîôêâçÏÖËÄÍÓÉÁÌÒÈÀÎÔÊÂÇ][a-zA-Z0-9_α-ζΑ-ΖïöëäíóéáìòèàîôêâçÏÖËÄÍÓÉÁÌÒÈÀÎÔÊÂÇ]*", |lex| lex.slice().parse())]
    Ident(String),

    // Types de données
    #[regex(r"-?\d(_?\d)*", |lex| lex.slice().replace("_", "").parse())]
    Int(i64),

    #[regex(r"-?\d+\.\d+", |lex| lex.slice().parse())]
    Float(f64),

    #[regex(r#""[^"]*"|'[^']*'"#, |lex| {
        let slice = lex.slice();
        slice[1..slice.len()-1].parse()
    })]
    Text(String),

    #[token("vrai", |_| true)]
    #[token("faux", |_| false)]
    Bool(bool),

    #[token("nul")]
    Nul,

    // Opérateurs Binaires
    #[token("+")]
    OpAdd,

    #[token("-")]
    OpMinus,

    #[token("*")]
    Star,

    #[token("/")]
    #[token("÷")]
    OpDiv,

    #[token("//")]
    #[token("div")]
    OpDivInt,

    #[token("%")]
    #[token("mod")]
    OpMod,

    #[token("**")]
    #[token("^")]
    OpExp,

    #[token("|")]
    OpPipe,

    // Opérateurs Binaires
    #[token("+=")]
    OpAddAssign,

    #[token("-=")]
    OpMinusAssign,

    #[token("*=")]
    OpTimesAssign,

    #[token("/=")]
    OpDivAssign,

    #[token("//=")]
    #[token("div=")]
    OpDivIntAssign,

    #[token("%=")]
    #[token("mod=")]
    OpModAssign,

    #[token("**=")]
    #[token("^=")]
    OpExpAssign,

    // Comparaisons Binaires
    #[token("==")]
    CompEq,

    #[token("!=")]
    #[token("≠")]
    #[token("<>")]
    CompNotEq,

    #[token("<")]
    CompLth,

    #[token("<=")]
    #[token("≤")]
    CompLeq,

    #[token(">")]
    CompGth,

    #[token("≥")]
    #[token(">=")]
    CompGeq,

    // Symboles
    #[token("=")]
    #[token("<-")]
    #[token("←")]
    Assign,

    #[token("->")]
    #[token("→")]
    RightArrow,

    #[token(".")]
    Dot,

    #[token("?")]
    QuestionMark,

    #[token("..")]
    #[token("jusqu'a")]
    #[token("jusqu'à")]
    RangeExcl,

    #[token("..=")]
    RangeIncl,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("{")]
    LCurly,

    #[token("}")]
    RCurly,

    #[regex(r"\n")]
    #[token(";")]
    EoS,

    #[regex(r"\(-:([^:]|:[^-]|:-[^\)])*:-\)")]
    ASDocs,

    #[regex(r"[ \t\f]+", logos::skip)]
    #[regex(r"#[^\n]*\n", logos::skip)]
    #[regex(r"\(:([^:]|:[^\)])*:\)", logos::skip)]
    #[error]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
