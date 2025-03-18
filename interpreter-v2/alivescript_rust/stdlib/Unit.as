utiliser Err {ErrAffirmation}

pub fonction affirmerVrai(test: booleen) -> rien
    si test alors retourner
    lancer(ErrAffirmation(format("Attendu: vrai, obtenu: {}", [test])))
fin fonction

const x = 1

