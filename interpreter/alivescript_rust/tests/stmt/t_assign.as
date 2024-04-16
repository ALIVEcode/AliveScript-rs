utiliser Test {affirmer, affirmerEgal}

x = 12
affirmer(x == 12)

var y = 13
affirmer(y == 13)

y, x = [x, y]
affirmer(y == 12 et x == 13)

const w, z = [x, y]
affirmer(z == 12 et w == 13)


pour x, y dans [[1, 2], [3, 4], [5, 6]]
    affirmerEgal(x, y - 1)
fin pour

foo: [entier, texte] = [1, "a"]


fonction dist(p1: [entier, entier], p2: [entier, entier]) -> decimal
    retourner ((p1[0] - p2[0]) ^ 2 + (p1[1] - p2[1]) ^ 2) ^ 0.5
fin fonction

afficher dist(
