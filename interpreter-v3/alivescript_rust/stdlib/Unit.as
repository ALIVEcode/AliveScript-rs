utiliser Err {ErrAffirmation}

pub fonction affirmerVrai(test: booleen) -> rien
    si test alors retourner
    lancer(ErrAffirmation(format("Attendu: vrai, obtenu: {}", [test])))
fin fonction

test = 0
si test alors; afficher "allo"; fin si

const x = 1

