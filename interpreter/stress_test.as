utiliser Test

structure Robot
    nom : texte
    energie : entier
fin structure

implementation Robot
    # Constructeur retournant le type Robot
    methode creer(n : texte) -> Robot
        retourner Robot { nom: n, energie: 100 }
    fin methode

    # Méthode ne retournant rien (procédure)
    methode recharger(inst, montant : entier) -> rien
        inst.energie = inst.energie + montant
    fin methode
fin implementation
# --- TEST : SATURATION DU TAS ---
var liste_infinie : liste<Robot> = []

#répéter
#    # On ajoute des instances à l'infini dans une liste globale
#    liste_infinie.ajouter(Robot.creer("Destructeur"))
#    afficher tailleDe(liste_infinie)
#fin répéter


# --- TEST : ACCÈS À UNE VARIABLE ÉVAPORÉE ---
fonction obtenir_danger() -> fonction
    var x : entier = 42
    fonction interne() -> entier
        retourner x
    fin fonction
    # Ici 'x' devrait être déplacé sur le tas, sinon 'interne' pointera vers du vide
    retourner interne
fin fonction


