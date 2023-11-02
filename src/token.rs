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

    #[token("sortir")]
    KwSortir,

    #[token("continuer")]
    KwContinuer,

    // Fonctions
    #[token("fonction")]
    KwFonction,

    #[token("retourner")]
    KwRetourner,

    // Variables
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().parse())]
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
    OpDiv,

    #[token("//")]
    OpDivInt,

    #[token("%")]
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
    OpDivIntAssign,

    #[token("%=")]
    OpModAssign,

    #[token("**=")]
    #[token("^=")]
    OpExpAssign,

    // Comparaisons Binaires
    #[token("==")]
    CompEq,

    #[token("!=")]
    CompNotEq,

    #[token("<")]
    CompLth,

    #[token("<=")]
    CompLeq,

    #[token(">")]
    CompGth,

    #[token(">=")]
    CompGeq,

    // Symboles
    #[token("=")]
    #[token("<-")]
    Assign,

    #[token("->")]
    RightArrow,

    #[token(".")]
    Dot,

    #[token("..")]
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

    #[regex(r"\n+")]
    #[token(";")]
    EoS,

    #[regex(r"[ \t\f]+", logos::skip)]
    #[error]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
