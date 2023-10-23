fonction allo()
fin fonction

fonction saluer(mot: texte)
    afficher mot
    afficher "bye"
fin fonction


fonction saluer(mot: texte = "hey") -> entier
    afficher mot
    afficher "bye"
    retourner 12
fin fonction

var x = 5

repeter
  afficher x
  sortir si x != 4
fin repeter


tant que x == 4
  afficher x
fin tant que
