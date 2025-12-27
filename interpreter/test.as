utiliser Test

(: 
   SUITE DE TESTS EXHAUSTIVE - VERSION 2.0
   Nouveautés : var/const, type 'rien', flèche de retour '->', boucles 'faire'
:)

# --- 1. TEST: DES VARIABLES ET ARITHMÉTIQUE ---
var a: entier = 10
const b: entier = 3  # b ne peut plus être modifié
var c: décimal = 2.5

Test.affirmerÉgaux(a + b, 13, "Addition entière échouée")
Test.affirmerÉgaux(a // b, 3, "Division entière (tronquée) échouée")
Test.affirmerÉgaux(a / b, 10 / 3, "Division échouée")
Test.affirmerÉgaux(c * 2.0, 5.0, "Multiplication réelle échouée")
Test.affirmer(10 > 5, "Comparaison logique échouée")

# --- 2. TEST: DES CHAÎNES ET CONVERSIONS ---
var s : texte = "42"
Test.affirmerÉgaux("Hello " + "World", "Hello World", "Concaténation échouée")
Test.affirmerÉgaux(entier(s), 42, "Conversion entier() échouée")
Test.affirmerÉgaux(texte(100), "100", "Conversion texte() échouée")

# --- 3. TEST: DES STRUCTURES ET MÉTHODES ---
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

var mon_robot : Robot = Robot.creer("R2D2")
Test.affirmerÉgaux(mon_robot.nom, "R2D2", "Attribut de structure incorrect")

mon_robot.recharger(50)
Test.affirmerÉgaux(mon_robot.energie, 150, "Échec de l'appel de méthode -> rien")

# --- 4. TEST: DES LISTES ET BOUCLES (Syntaxe 'faire') ---
var nombres : liste<entier> = [1, 2, 3, 4]
var somme : entier = 0

pour chaque n dans nombres faire
    somme = somme + n
fin pour

Test.affirmerÉgaux(somme, 10, "Somme de liste via 'faire' échouée")

# --- 5. TEST: DU MOT-CLÉ 'LIRE' ET 'SINON' ---
# Simulation du flux de contrôle du nouveau mot-clé lire
var entree_utilisateur : texte = "abc"
var age : entier = 0
var erreur_detectee : booleen = faux

# Syntaxe cible : lire entier dans age, "msg" sinon ...
# Ici on teste la logique que la VM exécutera
si entree_utilisateur.estNumérique() alors
    age = entier(entree_utilisateur)
sinon
    erreur_detectee = vrai
    age = -1
fin si

utiliser Texte
si Texte.estNumérique(entree_utilisateur) alors
    age = entier(entree_utilisateur)
sinon
    erreur_detectee = vrai
    age = -1
fin si

Test.affirmer(erreur_detectee, "L'échec de conversion n'a pas été détecté")

# --- 6. TEST: DE PORTÉE ET RETOUR 'RIEN' ---
var globale : entier = 1

fonction modifier_globale(valeur : entier) -> rien
    globale = globale + valeur
fin fonction

modifier_globale(9)
Test.affirmerÉgaux(globale, 10, "Le retour 'rien' ou la portée globale a échoué")

afficher "Tous les tests de base ont été passés avec succès !"

# --- 7. TEST: DE LA BOUCLE RÉPÉTER (N FOIS) ---
var compteur_fixe : entier = 0
répéter 5
    compteur_fixe = compteur_fixe + 1
fin répéter

Test.affirmerÉgaux(compteur_fixe, 5, "La boucle répéter N fois n'a pas tourné 5 fois")

# --- 8. TEST: DE LA BOUCLE RÉPÉTER (INFINIE + SORTIR) ---
var total_infini : entier = 0
répéter
    total_infini = total_infini + 1
    si total_infini == 10 alors
        sortir
    fin si
fin répéter

Test.affirmerÉgaux(total_infini, 10, "La boucle répéter infinie avec sortir a échoué")

# --- 9. TEST: TANT QUE (CONTINUER / SORTIR) ---
var i : entier = 0
var somme_paires : entier = 0

tant que i < 10 faire
    i = i + 1
    
    # On saute les nombres impairs
    si (i % 2) != 0 alors
        continuer
    fin si
    
    somme_paires = somme_paires + i
    
    # On arrête si on dépasse 6
    si i == 6 alors
        sortir
    fin si
fin tant que

# Devrait être : 2 + 4 + 6 = 12
Test.affirmerÉgaux(somme_paires, 12, "Logique continuer/sortir dans tant que échouée")

# --- 10. TEST: LOGIQUES COMPLEXES (ET, OU, NON) ---
const age : entier = 20
const a_permis : booleen = vrai
const est_fatigue : booleen = faux

var peut_conduire : booleen = faux

# Test SI / SINON SI / SINON avec logique combinée
si age >= 18 et a_permis et non est_fatigue alors
    peut_conduire = vrai
sinon si age >= 18 et non a_permis alors
    peut_conduire = faux
sinon
    peut_conduire = faux
fin si

Test.affirmer(peut_conduire, "Échec de la condition complexe et/non")

# Test de priorité et parenthèses
var logique_mixte : booleen = (5 > 10) ou (2 == 2 et non (1 == 0))
Test.affirmer(logique_mixte, "Échec de la priorité des opérateurs logiques")

# --- 11. TEST: POUR CHAQUE AVEC SORTIR ---
var somme_liste : entier = 0
const nombres : liste<entier> = [10, 20, 30, 40]

pour chaque n dans nombres faire
    si n > 25 alors
        sortir
    fin si
    somme_liste = somme_liste + n
fin pour

Test.affirmerÉgaux(somme_liste, 30, "Sortir dans une boucle pour chaque a échoué")

afficher "Tous les tests de contrôle et de logique sont validés !"


# --- 12. TEST : CLOSURE SIMPLE (CAPTURE DE VARIABLE) ---
fonction creer_ajouteur(base : entier) -> fonction
    # La fonction interne 'ajoute' capture 'base'
    fonction ajoute(valeur : entier) -> entier
        retourner base + valeur
    fin fonction
    
    retourner ajoute
fin fonction

const ajoute_dix = creer_ajouteur(10)
const ajoute_vingt = creer_ajouteur(20)

Test.affirmerÉgaux(ajoute_dix(5), 15, "La closure n'a pas capturé 10 correctement")
Test.affirmerÉgaux(ajoute_vingt(5), 25, "La deuxième instance de closure interfère avec la première")


# --- 13. TEST : CLOSURES INTRIQUÉES (MULTI-NIVEAUX) ---
fonction generateur_puissance(exposant : entier) -> fonction
    # Niveau 1 : capture 'exposant'
    fonction prefixeur(prefixe : texte) -> fonction
        # Niveau 2 : capture 'prefixe' ET 'exposant'
        fonction calcul(n : entier) -> texte
            var resultat : entier = 1
            répéter exposant
                resultat = resultat * n
            fin répéter
            retourner prefixe + texte(resultat)
        fin fonction
        
        retourner calcul
    fin fonction
    
    retourner prefixeur
fin fonction

const au_carre = generateur_puissance(2)
const label_carre = au_carre("Résultat : ")

const au_cube = generateur_puissance(3)
const label_cube = au_cube("Cube : ")

Test.affirmerÉgaux(label_carre(4), "Résultat : 16", "Capture multi-niveau (carré) échouée")
Test.affirmerÉgaux(label_cube(2), "Cube : 8.0", "Capture multi-niveau (cube) échouée")


# --- 14. TEST : ÉTAT PARTAGÉ (MUTATION DANS UNE CLOSURE) ---
fonction creer_compteur(depart : entier) -> fonction
    var compte : entier = depart
    
    # Cette closure modifie une variable 'var' de son parent
    fonction incrementer() -> entier
        compte = compte + 1
        retourner compte
    fin fonction
    
    retourner incrementer
fin fonction

const mon_compteur = creer_compteur(0)
Test.affirmerÉgaux(mon_compteur(), 1, "Premier appel du compteur échoué")
Test.affirmerÉgaux(mon_compteur(), 2, "La closure n'a pas conservé son état interne (mutation)")

const mon_compteur2 = creer_compteur(0)
Test.affirmerÉgaux(mon_compteur2(), 1, "Deuxieme compteur reprend de zéro")
Test.affirmerÉgaux(mon_compteur(), 3, "Le premier compteur n'a pas perdu son état interne")


# --- 15. TEST : CLOSURES MULTIPLES SUR LE MÊME SCOPE ---
fonction creer_banque(solde_initial : entier) -> liste<fonction>
    var solde : entier = solde_initial
    
    fonction deposer(montant : entier) -> rien
        solde = solde + montant
    fin fonction
    
    fonction voir_solde() -> entier
        retourner solde
    fin fonction
    
    retourner [deposer, voir_solde]
fin fonction

var compte_bancaire : liste<fonction> = creer_banque(100)
var depose = compte_bancaire[0]
var voir = compte_bancaire[1]

depose(50)
Test.affirmerÉgaux(voir(), 150, "Deux closures partageant la même variable ont échoué")

afficher "Tests des closures intriquées et de l'état partagé terminés !"

var x = essayer "a" / 2 sinon 12
Test.affirmerÉgaux(x, 12, "Essayer erreur avec sinon expression")


var y = essayer "a" / 2 sinon 
  voir()
  44
fin essayer

Test.affirmerÉgaux(y, 44, "Essayer erreur avec sinon bloc")

var z = essayer 2 / 2 sinon 
  44
fin essayer

Test.affirmerÉgaux(z, 1.0, "Essayer valide avec sinon bloc")

afficher "Tests des blocs 'essayer' terminés !"
