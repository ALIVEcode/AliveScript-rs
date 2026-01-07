var x = 255

var y = x * 2
var w = x * 2

x = debut 
  var t = x + x
  var t2 = x + t
  x + t2
fin


fonction plus(x)
  retourner fonction(y)
    retourner x + y
  fin fonction
fin fonction

var plus_1 = plus(1)

fonction apply(f, arg)
  retourner f(arg)
fin fonction

x = apply(plus_1, plus_1(plus_1(plus_1(2))))

si x == 5 alors
  afficher "wow"
sinon si x == 9
  afficher "not"
sinon si faux
  y = 2
  afficher y
sinon 
  afficher("sinoN " + x)
fin si

afficher "end"
tant que x > 0
  x = x - 1
  afficher x
fin tant que



var ls: liste<entier> = [1, 2, 3]

ls2 = [1]
ls += ls2

afficher typeDe(ls)
afficher ls

x = 12

x *= -1

afficher x

afficher tailleDe(ls)

foo = fonction(): "x = " + x

afficher foo()

structure Point
  x = foo()
  y: entier = 12
fin structure

implémentation Point
  methode creer_vide() -> Point
    retourner Point {x: 0, y: 0}
  fin methode

  méthode getX(inst) -> texte
    retourner "from getX: " + inst.x + " " + x
  fin méthode
fin implémentation

var p = Point {}
x = 22
var p2 = Point {}
afficher p
afficher p2.y
afficher p.getX()
afficher p2
afficher p

lsp = [p, p2]
lsp[1].y += 22
afficher ("p2 = " + p2)

var p3 = Point.creer_vide()
afficher p3


fonction abc()
fin fonction

afficher abc
afficher abc()

#lire entier dans age, "Ton âge : " sinon
#    afficher "Erreur : vous devez entrer un nombre entier !"
#    age = 0 # Valeur par défaut
#    afficher 1
#fin lire
#
#afficher ("age = " + age)

afficher 1


x, [y, z] = [1, [2, 3]]
afficher x
afficher y
afficher z


ls[2] += 44
afficher ls

afficher (5 et faux ou afficher("wow") et "s")


pour a, b dans [1, 2], [2, 3], [1, 5]
  afficher a
  si b == 3 alors continuer
  afficher b
fin pour

var x = essayer "a" / 2 sinon 12

afficher x


var y = essayer "a" / 2 sinon 
  afficher "oups"
  44
fin essayer

afficher y


fonction foo(x, y, z=12, plusieurs autres: entier)
  afficher autres
  afficher "x={} y={} z={}".format([x, y, z])
  retourner x + y + z
fin fonction

afficher foo(1, 2, 33, 77, 222, 1, "allo")
