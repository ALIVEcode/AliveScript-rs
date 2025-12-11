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

var x = apply(plus_1, plus_1(plus_1(plus_1(2))))

si x == 6 
  afficher "wow"
fin si

afficher "end"


