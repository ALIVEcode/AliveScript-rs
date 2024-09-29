utiliser Err {ErrAffirmation}

pub fonction affirmerVrai(test: booleen) -> rien
    lancer(ErrAffirmation(format("Attendu: vrai, obtenu: {}", [test])))
fin fonction

const x = 1

