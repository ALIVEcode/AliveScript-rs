utiliser Graphique
utiliser Aléatoire

structure Carré
  x: entier
  y: entier
  l: entier = 20
  h: entier = 20
  couleur: texte
  vel: décimal = 5
  mov: liste<décimal> = [Aléatoire.choix([-2, 2]), Aléatoire.choix([-2, 2])]
fin structure

implémentation Carré
  méthode créer(x: entier, y: entier, couleur: texte)
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

  # --- MÉTHODE : COLLISIONNE ---
  # Vérifie si l'instance actuelle touche une autre instance de Carré
  # Retourne vrai s'il y a contact, faux sinon.
  méthode collisionne(inst, autre : Carré) -> booleen
      # On vérifie les 4 limites (gauche, droite, haut, bas)
      var collision_x : booleen = (inst.x < autre.x + autre.l) et (inst.x + inst.l > autre.x)
      var collision_y : booleen = (inst.y < autre.y + autre.h) et (inst.y + inst.h > autre.y)
      
      retourner collision_x et collision_y
  fin méthode

  # --- MÉTHODE : REBONDIR ---
  # Inverse les vecteurs de mouvement si un impact est détecté
  méthode rebondir(inst, autre : Carré) -> rien
    si inst.collisionne(autre) alors
      # 1. Calcul de l'enfoncement sur chaque axe
      var delta_x : décimal = (inst.x + inst.l / 2.0) - (autre.x + autre.l / 2.0)
      var delta_y : décimal = (inst.y + inst.h / 2.0) - (autre.y + autre.h / 2.0)
        
      # 2. On détermine l'axe de collision le plus profond
      # (Si la différence en X est plus grande que celle en Y, le choc est latéral)
      si abs(delta_x) > abs(delta_y) alors
          # Choc Horizontal : on inverse le mouvement X
          inst.mov[0] *= -1.0
          autre.mov[0] *= -1.0
          
          # Résolution : on pousse un peu pour séparer
          var overlap : entier = ((inst.l / 2 + autre.l / 2) - entier(abs(delta_x))) * 2
          si delta_x > 0 alors inst.x += overlap 
          sinon inst.x -= overlap
      sinon
          # Choc Vertical : on inverse le mouvement Y
          inst.mov[1] *= -1.0
          autre.mov[1] *= -1.0
          
          # Résolution : on pousse un peu pour séparer
          var overlap : entier = ((inst.h / 2 + autre.h / 2) - entier(abs(delta_y))) * 2
          si delta_y > 0 alors inst.y += overlap 
          sinon inst.y -= overlap
      fin si
    fin si
  fin méthode
fin implémentation

var couleurs = ["bleu", "rouge", "orange", "vert", "blanc"]
var carrés: liste<Carré> = []

fonction init()
  var nb = 30
  var div = 800 / nb
  pour chaque i dans suite(0, nb) faire
    carrés.ajouter(Carré.créer(i * 30 + i * div, i * 30 + i * div, couleurs[i % tailleDe(couleurs)]))
  fin pour
fin fonction

fonction update()
  pour chaque carré dans carrés faire
    pour chaque autre_carré dans carrés 
      si carré == autre_carré alors continuer
      carré.rebondir(autre_carré)
    fin pour
    carré.bouger()
  fin pour
fin fonction

fonction dessiner()
  Graphique.dessinerFond("noir")
  pour chaque carré dans carrés faire
    carré.dessiner()
  fin pour
fin fonction
