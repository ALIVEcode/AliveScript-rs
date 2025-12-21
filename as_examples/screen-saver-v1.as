utiliser Graphique

var carré_x: entier = 0
var carré_y: entier = 0

const CARRÉ_W = 50
const CARRÉ_H = 50

const VEL: décimal = 0.005

var mov_x: décimal = VEL
var mov_y: décimal = VEL


répéter
  #Graphique.attendre(Graphique.tempsImage() * 100)
  var vel = VEL
  var écran: liste = Graphique.tailleÉcran()
  var w: décimal = écran[0]
  var h: décimal = écran[1]

  si carré_x + CARRÉ_W > w alors 
    mov_x = -vel
    carré_x = w - CARRÉ_W - 1
  sinon si carré_x < 0 alors 
    mov_x = vel
    carré_x = 1
  fin si

  si carré_y + CARRÉ_H > h alors 
    mov_y = -vel
    carré_y = h - CARRÉ_H - 1
  sinon si carré_y < 0 alors 
    mov_y = vel
    carré_y = 1
  fin si

  carré_x += mov_x
  carré_y += mov_y

  Graphique.dessinerRect(carré_x, carré_y, CARRÉ_W, CARRÉ_H, "blanc")
fin répéter

