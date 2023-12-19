utiliser Test {affirmer}

x = 12
affirmer(x == 12)

var y = 13
affirmer(y == 13)

y, x = [x, y]
affirmer(y == 12 et x == 13)

const w, z = [x, y]
affirmer(z == 12 et w == 13)


pour x, y dans [[1, 2], [3, 4], [5, 6]]
    affirmer(x == y - 1)
fin pour

