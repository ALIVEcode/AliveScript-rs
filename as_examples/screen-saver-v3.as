utiliser Graphique

structure Carré
  x: entier
  y: entier
  l: entier = 50
  h: entier = 50
  couleur: texte
  vel: décimal = 5
  mov: liste<décimal> = [5, 5]
fin structure

implémentation Carré
  méthode créer(x: entier, y: entier, couleur: texte)
    #afficher [x, y, couleur]
    retourner Carré {x: x, y: y, couleur: couleur}
  fin méthode

  méthode bouger(inst)
    var [ecr_l: décimal, ecr_h: décimal] = Graphique.tailleÉcran()

    si inst.x + inst.l > ecr_l alors 
      inst.mov[0] = -inst.vel
      inst.x = ecr_l - inst.l - 1
    sinon si inst.x < 0 alors 
      inst.mov[0] = inst.vel
      inst.x = 1
    fin si

    si inst.y + inst.h > ecr_h alors 
      inst.mov[1] = -inst.vel
      inst.y = ecr_h - inst.h - 1
    sinon si inst.y < 0 alors 
      inst.mov[1] = inst.vel
      inst.y = 1
    fin si

    inst.x += inst.mov[0]
    inst.y += inst.mov[1]
  fin méthode

  méthode dessiner(inst)
    Graphique.dessinerRect(inst.x, inst.y, inst.l, inst.h, inst.couleur)
  fin méthode
fin implémentation

var couleurs = ["bleu", "rouge", "orange", "vert", "blanc"]
var carrés: liste<Carré> = []

fonction init()
  pour chaque i dans suite(0, 10) faire
    carrés.ajouter(Carré.créer(i, i * 75, couleurs[i % tailleDe(couleurs)]))
  fin pour
fin fonction

fonction update()
  pour chaque carré dans carrés faire
    carré.bouger()
  fin pour
fin fonction

fonction dessiner()
  Graphique.changerFond("noir")
  pour chaque carré dans carrés faire
    carré.dessiner()
  fin pour
fin fonction
