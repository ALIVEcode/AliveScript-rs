#[derive(Debug, PartialEq, Clone)]
pub enum Data {
    Afficher(String),
    Erreur { texte: String, ligne: usize },
    Demander { prompt: Option<String> },
}

impl Data {
    pub fn is_erreur(&self) -> bool {
        matches!(self, Data::Erreur { .. })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Response {
    Text(String)
}
