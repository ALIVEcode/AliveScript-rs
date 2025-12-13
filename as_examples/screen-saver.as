i = 0

var couleurs = ["bleu", "orange", "noir", "vert"]

carre_x = 0
carre_y = 0
const CARRE_W = 50
const CARRE_H = 50

const VEL = 0.001
mov_x = VEL
mov_y = VEL

tant que vrai
  écran = tailleÉcran()
  w = écran[0]
  h = écran[1]

  si carre_x + CARRE_W > w alors 
    mov_x = -VEL
    carre_x = w - CARRE_W - 1
  sinon si carre_x < 0 alors 
    mov_x = VEL
    carre_x = 1
  fin si

  si carre_y + CARRE_H > h alors 
    mov_y = -VEL
    carre_y = h - CARRE_H - 1
  sinon si carre_y < 0 alors 
    mov_y = VEL
    carre_y = 1
  fin si

  carre_x += mov_x
  carre_y += mov_y

  dessinerRect(carre_x, carre_y, CARRE_W, CARRE_H, "blanc")
fin tant que

