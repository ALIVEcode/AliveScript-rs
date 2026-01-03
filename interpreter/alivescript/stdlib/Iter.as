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
      si i + 1 >= taille alors retourner nul
      sinon retourner [i + 1, it[i]]
  fin fonction

  retourner gen([prochain, it, -1])
fin fonction


const prochain = iter([1, 2, 3, 4])

var [i, el] = prochain()
tant que i != nul
  [i, el] = quand prochain()
    vaut nul -> sortir
    sinon avec p si tailleDe(p) == 1 faire
      afficher tailleDe(p) == 1
      p
    sinon!
  fin quand

  afficher i + " " + el
fin tant que


