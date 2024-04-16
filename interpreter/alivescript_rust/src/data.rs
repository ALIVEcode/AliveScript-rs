use derive_new::new;

#[derive(Debug, PartialEq, Clone, Eq, new)]
pub enum Data {
    Afficher(String),
    Erreur { texte: String, ligne: usize },
    Demander { prompt: Option<String> },
    GetFichier(String),
    NotifInfo { msg: String },
    NotifErr { msg: String },
}

impl Data {
    pub fn is_erreur(&self) -> bool {
        matches!(self, Data::Erreur { .. })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Response {
    Text(String),
}
