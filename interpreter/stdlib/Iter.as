(:
    un iterateur est un objet contenant la methode __prochain__() qui retourne
    une valeur et une methode __estFini__()
    indiquant si l'itérateur est fini
:)

fonction estIter(obj) -> booleen
    retourner (
        contientAttr(obj, "__prochain__") et
        typeDe(obj.__prochain__) == "fonction"
    )
fin fonction

classe suite
    debutSuite: entier
    finSuite: entier
    bond: entier
    _curseur: entier

    init(debutSuite: entier, finSuite: entier, bond: entier = 1)
        inst.debutSuite = debutSuite
        inst.finSuite = finSuite
        inst.bond = bond
        inst._curseur = debutSuite
    fin init

    methode __prochain__()
        si inst._curseur < inst.finSuite alors
            inst._curseur += inst.bond
            retourner inst._curseur - inst.bond
        sinon
            retourner nul
        fin si
    fin methode

    methode __estFini__()
        retourner inst._curseur >= inst.finSuite
    fin methode
fin classe

s = suite(1, 3)
tant que pas s.__estFini__()
    afficher s.__prochain__()
fin tant que

