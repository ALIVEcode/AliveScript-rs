use logos::Logos;
use std::fmt;

#[derive(Logos, Clone, Debug, PartialEq)]
// #[logos(skip r"[ \t]+")]
pub enum Token {
    #[token("utiliser")]
    KwUtiliser,

    #[token("alias")]
    KwAlias,

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

    #[token("chaque")]
    KwChaque,

    #[token("faire")]
    KwFaire,

    #[token("repeter")]
    #[token("répéter")]
    KwRepeter,

    #[token("dans")]
    #[token("∈")]
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
    #[token("classe")]
    KwClasse,

    #[token("init")]
    KwInit,

    #[token("methode")]
    #[token("méthode")]
    KwMethode,

    #[token("statique")]
    KwStatique,

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
        let s: String = slice[1..slice.len()-1].parse().unwrap();
        s.replace(r"\n", "\n").replace(r"\t", "\t").replace(r"\r", "\r").to_owned()
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

    #[token(">=")]
    #[token("≥")]
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

    #[token("??")]
    DoubleQuestionMark,

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

    #[regex(r"\(-:([^:]|:[^-]|:-[^\)])*:-\)", |lex| {
        let slice = lex.slice();
        slice[3..slice.len()-3].parse().map(|s: String| s.trim().to_owned())
    })]
    ASDocs(String),

    #[regex(r"[ \t\f]+", logos::skip)]
    #[regex(r"#[^\n]*", logos::skip)]
    #[regex(r"\(:([^:]|:[^\)])*:\)", logos::skip)]
    #[error]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_string = match self {
            Token::KwUtiliser => "utiliser".to_owned(),
            Token::KwAlias => "alias".to_owned(),

            Token::KwLire => "lire".to_owned(),
            Token::KwAfficher => "afficher".to_owned(),
            Token::KwVar => "var".to_owned(),
            Token::KwConst => "const".to_owned(),

            Token::KwFin => "fin".to_owned(),

            Token::KwSi => "si".to_owned(),
            Token::KwSinon => "sinon".to_owned(),
            Token::KwAlors => "alors".to_owned(),
            Token::KwTantQue => "tant que".to_owned(),
            Token::KwPour => "pour".to_owned(),
            Token::KwChaque => "chaque".to_owned(),
            Token::KwFaire => "faire".to_owned(),
            Token::KwRepeter => "repeter".to_owned(),
            Token::KwSortir => "sortir".to_owned(),
            Token::KwContinuer => "continuer".to_owned(),

            Token::KwPas => "pas".to_owned(),
            Token::KwNon => "non".to_owned(),
            Token::KwEt => "et".to_owned(),
            Token::KwOu => "ou".to_owned(),
            Token::KwXor => "xor".to_owned(),

            Token::KwFonction => "fonction".to_owned(),
            Token::KwRetourner => "retourner".to_owned(),
            Token::KwClasse => "classe".to_owned(),
            Token::KwMethode => "methode".to_owned(),
            Token::KwStatique => "statique".to_owned(),
            Token::KwInit => "init".to_owned(),

            Token::Nul => format!("NUL"),
            Token::Ident(v) => format!("IDENTIFIANT({v})"),
            Token::Int(i) => format!("ENTIER({i})"),
            Token::Float(d) => format!("DÉCIMAL({d})"),
            Token::Text(s) => format!("TEXTE(\"{s}\")"),
            Token::Bool(b) => format!("BOOLÉEN({b})"),

            Token::OpAdd => "PLUS(+)".to_owned(),
            Token::OpMinus => "MOINS(-)".to_owned(),
            Token::Star => "ÉTOILE(*)".to_owned(),
            Token::OpDiv => "DIV(/)".to_owned(),
            Token::OpDivInt => "DIV_ENTIÈRE(//, div)".to_owned(),
            Token::OpMod => "MODULO(%, mod)".to_owned(),
            Token::OpExp => "EXPOSANT(**, ^)".to_owned(),
            Token::OpPipe => "BARRE(|)".to_owned(),

            Token::OpAddAssign => "PLUS_AFFECT(+=)".to_owned(),
            Token::OpMinusAssign => "MOINS_AFFECT(-=)".to_owned(),
            Token::OpTimesAssign => "FOIS_AFFECT(*=)".to_owned(),
            Token::OpDivAssign => "DIV_AFFECT(/=)".to_owned(),
            Token::OpDivIntAssign => "DIV_ENTIÈRE_AFFECT(//=, div=)".to_owned(),
            Token::OpModAssign => "MODULO_AFFECT(%=, mod=)".to_owned(),
            Token::OpExpAssign => "EXPOSANT_AFFECT(**=, ^=)".to_owned(),

            Token::CompEq => "EGAL(==)".to_owned(),
            Token::CompNotEq => "PAS_EGAL(!=, <>, ≠)".to_owned(),
            Token::CompLth => "PLUS_PETIT(<)".to_owned(),
            Token::CompLeq => "PLUS_PETIT_EGAL(<=, ≤)".to_owned(),
            Token::CompGth => "PLUS_GRAND(>)".to_owned(),
            Token::CompGeq => "PLUS_GRAND_EGAL(>=, ≥)".to_owned(),
            Token::KwDans => "dans".to_owned(),

            Token::Assign => "AFFECTER(=, <-, ←)".to_owned(),
            Token::RightArrow => "FLECHE_DROITE(->, →)".to_owned(),
            Token::Dot => "POINT(.)".to_owned(),
            Token::QuestionMark => "POINT_INTER(?)".to_owned(),
            Token::DoubleQuestionMark => "DOUBLE_POINT_INTER(??)".to_owned(),

            Token::RangeExcl => "SUITE_EXCL(.., jusqu'a, jusqu'à)".to_owned(),
            Token::RangeIncl => "SUITE_INCL(..=)".to_owned(),
            Token::KwBond => "bond".to_owned(),

            Token::Comma => "VIRGULE(,)".to_owned(),
            Token::Colon => "DEUX_POINTS(:)".to_owned(),
            Token::LParen => "PARENT_G(()".to_owned(),
            Token::RParen => "PARENT_D())".to_owned(),
            Token::LBracket => "CROCHET_G([)".to_owned(),
            Token::RBracket => "CROCHET_D(])".to_owned(),
            Token::LCurly => "ACCOLADE_G({)".to_owned(),
            Token::RCurly => "ACCOLADE_D(})".to_owned(),
            Token::EoS => "FIN_COMMANDE(\\n, ;)".to_owned(),
            Token::ASDocs(docs) => format!("DOCUMENTATION({docs})"),
            Token::Error => "ERREUR".to_owned(),
        };
        write!(f, "{}", to_string)
    }
}
