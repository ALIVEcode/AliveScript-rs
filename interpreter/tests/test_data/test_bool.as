utiliser Test {affirmer, affirmerFaux, affirmerEgal, affirmerPasEgal}

affirmer(vrai)
affirmer(pas faux)
affirmer(non faux)

affirmerFaux(faux)
affirmerFaux(non vrai)
affirmerFaux(pas vrai)

affirmerEgal(vrai, non faux)
affirmerEgal(vrai, pas faux)
affirmerEgal(faux, non vrai)
affirmerEgal(faux, pas vrai)

affirmerPasEgal(faux, pas faux)
affirmerPasEgal(faux, non faux)
affirmerPasEgal(vrai, pas vrai)
affirmerPasEgal(vrai, non vrai)


affirmer(booleen(1))
affirmerFaux(booleen(0))

affirmer(booleen([1]))
affirmerFaux(booleen([]))

affirmer(booleen("1"))
affirmerFaux(booleen(""))

affirmer(booleen({1:1}))
affirmerFaux(booleen({}))

affirmerEgal(vrai, non non vrai)
