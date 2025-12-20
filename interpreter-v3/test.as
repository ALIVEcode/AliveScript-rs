utiliser Test

(: 
   SUITE DE TESTS EXHAUSTIVE - VERSION 2.0
   Nouveautés : var/const, type 'rien', flèche de retour '->', boucles 'faire'
:)

# --- 1. TESTS DES VARIABLES ET ARITHMÉTIQUE ---
var a : entier = 10
const b : entier = 3  # b ne peut plus être modifié
var c : reel = 2.5

Test.affirmerÉgaux(a + b, 13, "Addition entière échouée")
Test.affirmerÉgaux(a / b, 3, "Division entière (tronquée) échouée")
Test.affirmerÉgaux(c * 2.0, 5.0, "Multiplication réelle échouée")
Test.affirmer(10 > 5, "Comparaison logique échouée")

# --- 2. TESTS DES CHAÎNES ET CONVERSIONS ---
var s : chaine = "42"
Test.affirmerÉgaux("Hello " + "World", "Hello World", "Concaténation échouée")
Test.affirmerÉgaux(entier(s), 42, "Conversion entier() échouée")
Test.affirmerÉgaux(chaine(100), "100", "Conversion chaine() échouée")

# --- 3. TESTS DES STRUCTURES ET MÉTHODES ---
structure Robot
    nom : chaine
    energie : entier
fin structure

implementation Robot
    # Constructeur retournant le type Robot
    methode creer(n : chaine) -> Robot
        retourner Robot { nom: n, energie: 100 }
    fin methode

    # Méthode ne retournant rien (procédure)
    methode recharger(inst, montant : entier) -> rien
        inst.energie = inst.energie + montant
    fin methode
fin implementation

var mon_robot : Robot = Robot.creer("R2D2")
Test.affirmerÉgaux(mon_robot.nom, "R2D2", "Attribut de structure incorrect")

# Ajout dynamique (Shape transition)
mon_robot.version = 1.0 
mon_robot.recharger(50)
Test.affirmerÉgaux(mon_robot.energie, 150, "Échec de l'appel de méthode -> rien")

# --- 4. TESTS DES LISTES ET BOUCLES (Syntaxe 'faire') ---
var nombres : liste<entier> = [1, 2, 3, 4]
var somme : entier = 0

pour chaque n dans nombres faire
    somme = somme + n
fin pour

Test.affirmerÉgaux(somme, 10, "Somme de liste via 'faire' échouée")

# --- 5. TESTS DU MOT-CLÉ 'LIRE' ET 'SINON' ---
# Simulation du flux de contrôle du nouveau mot-clé lire
var entree_utilisateur : chaine = "abc"
var age : entier = 0
var erreur_detectee : booleen = faux

# Syntaxe cible : lire entier dans age, "msg" sinon ...
# Ici on teste la logique que la VM exécutera
si est_numerique(entree_utilisateur) alors
    age = entier(entree_utilisateur)
sinon
    erreur_detectee = vrai
    age = -1
fin si

Test.affirmer(erreur_detectee, "L'échec de conversion n'a pas été détecté")

# --- 6. TESTS DE PORTÉE ET RETOUR 'RIEN' ---
var globale : entier = 1

fonction modifier_globale(valeur : entier) -> rien
    globale = globale + valeur
fin fonction

modifier_globale(9)
Test.affirmerÉgaux(globale, 10, "Le retour 'rien' ou la portée globale a échoué")

afficher "Tous les tests de base ont été passés avec succès !"

utiliser Test

# --- 7. TESTS DE LA BOUCLE RÉPÉTER (N FOIS) ---
var compteur_fixe : entier = 0
répéter 5
    compteur_fixe = compteur_fixe + 1
fin répéter

Test.affirmerÉgaux(compteur_fixe, 5, "La boucle répéter N fois n'a pas tourné 5 fois")

# --- 8. TESTS DE LA BOUCLE RÉPÉTER (INFINIE + SORTIR) ---
var total_infini : entier = 0
répéter
    total_infini = total_infini + 1
    si total_infini == 10 alors
        sortir
    fin si
fin répéter

Test.affirmerÉgaux(total_infini, 10, "La boucle répéter infinie avec sortir a échoué")

# --- 9. TESTS TANT QUE (CONTINUER / SORTIR) ---
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

# --- 10. TESTS LOGIQUES COMPLEXES (ET, OU, NON) ---
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

# --- 11. TEST POUR CHAQUE AVEC SORTIR ---
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
