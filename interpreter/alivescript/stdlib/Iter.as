fonction gen(arg: [iter: fn(tout) -> tout, état: tout, init: tout])
  var [it, état, contrôle] = arg

  retourner fonction()
    si contrôle == nul alors retourner nul
    const résultats = it(état, contrôle)
    si résultats == nul alors 
      contrôle = nul
      retourner nul
    fin si

    contrôle = résultats[0]
    retourner résultats
  fin fonction
fin fonction


fonction iter(it: itérable)
  const taille = tailleDe(it)

  fonction prochain(it: itérable, i: entier)
      si i >= taille alors retourner nul
      sinon retourner [i + 1, it[i]]
  fin fonction

  retourner gen([prochain, it, 0])
fin fonction



