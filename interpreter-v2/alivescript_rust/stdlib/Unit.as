utiliser Err {ErrAffirmation}

pub fonction affirmerVrai(test: booleen) -> rien
    si test alors 
        retourner 
    fin si

    lancer(ErrAffirmation(format("")))
fin fonction

const x = 1

