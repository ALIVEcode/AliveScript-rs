classe Option
    statique _vide = Option(nul)
    _val

    init(val)
        inst._val = val
    fin init

    methode getVal()
        si inst == Option._vide alors
            erreur("L'option ne contient pas de valeurs.")
        fin si
        retourner inst._val
    fin methode

    methode statique vide() -> Option
        retourner Option._vide
    fin methode

    methode __texte__()
        si inst == Option._vide
            afficher "Vide"
        sinon
            afficher "Val(" + inst._val + ")"
        fin si
    fin methode
fin classe

