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

var p3 = Point.creer_vide()
afficher p3


