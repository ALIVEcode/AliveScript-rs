(:
    un iterateur est un objet contenant la methode __prochain__() qui retourne
    une valeur et une methode __estFini__()
    indiquant si l'itérateur est fini
:)

fonction estIter(obj) -> booleen
    retourner (contientAttr(obj, "__prochain__") et typeDe(obj.__prochain__) == "fonction")
fin fonction

fonction prochain(it: iterable)
    retourner nul si it.__estFini__()
    retourner it.__prochain__()
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

    methode __iter__()
        retourner inst
    fin methode

    methode __prochain__()
        inst._curseur += inst.bond
        retourner inst._curseur - inst.bond
    fin methode

    methode __estFini__()
        retourner inst._curseur > inst.finSuite
    fin methode
fin classe

classe iter 
    iterateur: iterable
    idx: entier? = nul

    init(iterateur: iterable)
        iterMethode = getAttr(iterateur, "__iter__", nul)
        si iterMethode == nul alors
            inst.iterateur = liste(iterateur)
            inst.idx = 0
        sinon
            inst.iterateur = iterMethode()
        fin si
    fin init

    methode __iter__()
        retourner inst
    fin methode 

    methode __prochain__()
        retourner inst.iterateur.__prochain__() si inst.idx == nul

        inst.idx += 1
        retourner inst.iterateur[inst.idx - 1]
    fin methode

    methode __estFini__()
        retourner inst.iterateur.__estFini__() si inst.idx == nul
        retourner inst.idx > tailleDe(inst.iterateur)
    fin methode

    methode __texte__()
        retourner "obj iter"
    fin methode

fin classe


classe mapIter
    f: fonction
    i: iterable

    init(f: fonction, l: iterable)
        inst.f = f
        inst.i = iter(l)
    fin init

    methode __iter__()
        retourner inst
    fin methode

    methode __prochain__()
        retourner inst.f(inst.i.__prochain__())
    fin methode

    methode __estFini__()
        retourner inst.i.__estFini__()
    fin methode

    methode __texte__()
        retourner "obj map"
    fin methode
fin classe


fonction map(f: fonction, l: iterable)
    var liste_finale = []
    pour chaque e dans l
        liste_finale += f(e)
    fin pour
    retourner liste_finale
fin fonction

fonction filtrer(f: fonction, l: iterable)
    var liste_finale = []
    pour chaque e dans l
        liste_finale += e si f(e)
    fin pour
    retourner liste_finale
fin fonction

fonction reduire(f: fonction, l: iterable, inital=nul)
    l = iter(l)
    acc = initial ?? prochain(l)
    pour e dans l
        acc = f(e, acc)
    fin pour
    retourner acc
fin fonction

