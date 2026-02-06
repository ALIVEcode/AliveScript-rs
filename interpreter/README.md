# AliveScript

AliveScript est un langage de programmation moderne avec une syntaxe française, conçu pour être expressif et facile à apprendre. Il combine des concepts de programmation fonctionnelle et impérative dans une syntaxe élégante et lisible.

## Installation

### Prérequis

- Rust (version 1.70+)
- Git

### Compiler depuis les sources

```bash
# Cloner le dépôt
git clone https://github.com/votre-utilisateur/AliveScript-rs.git
cd AliveScript-rs/interpreter

# Compiler et installer
make install

# Ou manuellement :
cargo install --path alivescript
```

### Vérifier l'installation

```bash
alivec --version
```

## Premiers pas

### Hello World

Créez un fichier `bonjour.as` :

```alivescript
afficher "Bonjour, monde !"
```

Exécutez-le :

```bash
alivec bonjour.as
```

### Mode interactif (REPL)

Lancez le REPL pour expérimenter avec le langage :

```bash
alivec
```

### Utiliser le gestionnaire de projet `alive`

Assurez-vous d'avoir installer avec `make install`

```bash
# Créer un nouveau projet
mkdir mon_projet
cd mon_projet
alive init


# Exécuter le projet
alive exec

# Ajouter une dépendance à votre projet
alive ajouter <URL.git> <Nom>
```

## Fondamentaux du langage

### Variables et constantes

```alivescript
# Variables mutables
var age: entier = 25
var nom: texte = "Alice"

# Constantes immutables  
const PI: décimal = 3.14159
const VRAI: booléen = vrai
```

### Types de base

```alivescript
# Types numériques
var entier: entier = 42
var decimal: décimal = 3.14

# Texte
var message: texte = "Bonjour"

# Booléens
var est_vrai: booléen = vrai
var est_faux: booléen = faux

# Nul
var valeur_nulle = nul
```

### Opérations arithmétiques

```alivescript
var a = 10
var b = 3

afficher a + b     # 13 (addition)
afficher a - b     # 7 (soustraction)
afficher a * b     # 30 (multiplication)
afficher a / b     # 3.333... (division réelle)
afficher a // b    # 3 (division entière)
afficher a % b     # 1 (modulo)
afficher a ^ 2     # 100 (puissance)
```

### Chaînes de caractères

```alivescript
var prenom = "Alice"
var nom = "Martin"

# Concaténation
var nom_complet = prenom + " " + nom

# Conversion
var nombre_texte = "42"
var nombre = entier(nombre_texte)  # 42

# Méthodes de chaîne
var txt = "Bonjour monde"
afficher txt.taille()        # 12
afficher txt.enMajuscule()   # "BONJOUR MONDE"
afficher txt.crier()   # "BONJOUR MONDE"
afficher "monde" dans txt    # vrai
```

## Structures de contrôle

### Conditions

```alivescript
var age = 18

# Simple si
si age >= 18 alors
    afficher "Majeur"
fin si

# Simple si sur une ligne
si age >= 18 alors afficher "Majeur"

# Si/sinon
si age >= 18 alors
    afficher "Majeur"
sinon
    afficher "Mineur"
fin si

# Si/sinon si/sinon
si age < 13 alors
    afficher "Enfant"
sinon si age < 18 alors
    afficher "Adolescent"
sinon
    afficher "Adulte"
fin si
```

### Logique combinée

```alivescript
var age = 25
var a_permis = vrai
var est_fatigue = faux

# Opérateurs logiques
si age >= 18 et a_permis et non est_fatigue alors
    afficher "Peut conduire"
fin si

# Priorité et parenthèses
var condition = (age >= 18 et a_permis) ou (est_fatigue et age >= 21)
```

### Boucles

#### Boucle `pour chaque`

```alivescript
var nombres = 1 jusqu'à 5  # Suite inclusive de 1 à 5
var somme = 0

pour chaque n dans nombres faire
    somme = somme + n
fin pour

afficher somme  # 15
```

#### Boucle `répéter`

```alivescript
# Répéter N fois
var compteur = 0
répéter 5
    compteur += 1
fin répéter

# Boucle infinie avec sortie
var total = 0
répéter
    total += 1
    si total == 10 alors
        sortir
    fin si
fin répéter
```

#### Boucle `tant que`

```alivescript
var i = 0
tant que i < 10 faire
    afficher i
    i += 1
    
    si i == 5 alors
        continuer  # saute à l'itération suivante
    fin si
    
    si i == 8 alors
        sortir     # sort de la boucle
    fin si
fin tant que
```

### Contrôle avancé

#### Expression `quand`

```alivescript
var valeur = 42

quand valeur
    # utilisez 'faire' pour une plusieurs commandes
    vaut 1, 2, 3 faire
        afficher "Petit nombre"
        afficher "Très petit même"

    # utilisez '->' pour une seule commande sur la même ligne
    vaut 42 -> afficher "La réponse universelle"

    # `est` regarde pour le type de la valeur
    est entier faire
        afficher "C'est un entier"

    (: 
        `avec` permet d'affecter la valeur à une variable
        `si` permet d'ajouter une condition de guarde
    :) 
    sinon avec v si v > 20 faire
        afficher "Autre chose"

    (: 
        branche spéciale qui envoie une erreur
        nécessaire si le bloc quand est utilisé dans une expression
    :)
    sinon -> !
fin quand
```

#### Gestion d'erreurs avec `essayer`

```alivescript
# Essayer avec sinon simple
var resultat = essayer "a" / 2 sinon 12

# Essayer avec bloc sinon
var x = essayer "a" / 2 sinon
    afficher "Erreur de division"
    44
fin essayer
```

## Fonctions

### Définition de fonction

```alivescript
# Fonction simple
fonction carre(x: entier) -> entier
    retourner x * x
fin fonction

# Fonction sans retour (procédure)
fonction saluer(nom: texte) -> rien
    afficher "Bonjour, " + nom + " !"
fin fonction

# Fonction fléchée
var doubler = fonction(x) -> entier = x * 2
const plus3(x) = x + 3
```

### Appel de fonction

```alivescript
var resultat = carre(5)        # 25
saluer("Alice")                # Affiche le message

# Statement "commande" (appel sans parenthèse avec un arguement)
saluer "Alice"                 # Affiche le message 

var double = doubler(10)       # 20
```

### Fonctions comme valeurs

```alivescript
# Fonctions en paramètre
fonction appliquer(f: fonction, x: entier) -> entier
    retourner f(x)
fin fonction

var resultat = appliquer(carre, 3)  # 9
```

### Closures (fonctions imbriquées)

```alivescript
# Closure simple
fonction creer_ajouteur(base: entier) -> fonction
    fonction ajoute(valeur: entier) -> entier
        retourner base + valeur
    fin fonction
    retourner ajoute
fin fonction

var ajoute_dix = creer_ajouteur(10)
afficher ajoute_dix(5)  # 15

# Closures multi-niveaux
fonction generateur_puissance(exposant: entier) -> fonction
    fonction prefixeur(prefixe: texte) -> fonction
        fonction calcul(n: entier) -> texte
            var resultat = 1
            répéter exposant
                resultat *= n
            fin répéter
            retourner prefixe + texte(resultat)
        fin fonction
        retourner calcul
    fin fonction
    retourner prefixeur
fin fonction

var au_carre = generateur_puissance(2)
var label_carre = au_carre("Résultat : ")
afficher label_carre(4)  # "Résultat : 16"
```

## Structures de données

### Listes

```alivescript
# Création de listes
var nombres = [1, 2, 3, 4, 5]
var listeVide = []
var mixte = [1, "deux", 3.0, vrai]

# Accès aux éléments
afficher nombres[0]        # 1
afficher nombres[1]        # 2

# Modification
nombres[0] = 10

# Méthodes de liste
nombres.ajouter(6)
nombres.insérer(0, 0)
var element = nombres.retirer()
var taille = nombres.taille()
```

### Dictionnaires

```alivescript
# Création
var personne = {
    "nom": "Alice",
    "age": 25,
    "ville": "Paris"
}

# Accès
afficher personne["nom"]     # "Alice"
afficher personne.age        # 25 (accès par propriété)

# Modification
personne["age"] = 26
personne.ville = "Lyon"

# Ajout
personne["pays"] = "France"
```

### Structures personnalisées

```alivescript
# Définition de structure
structure Robot
    nom: texte
    energie: entier
fin structure

# Implémentation de méthodes
implémentation Robot
    méthode creer(n: texte) -> Robot
        retourner Robot { nom: n, energie: 100 }
    fin méthode

    méthode recharger(inst, montant: entier) -> rien
        inst.energie += montant
    fin méthode
fin implémentation

# Utilisation
var mon_robot = Robot.creer("R2D2")
afficher mon_robot.nom        # "R2D2"
afficher mon_robot.energie    # 100

mon_robot.recharger(50)
afficher mon_robot.energie    # 150
```

## Modules et imports

### Importer des modules

```alivescript
# Importer tout un module
utiliser Math

# Importer avec alias
utiliser Texte alias T

# Importer des membres spécifiques
utiliser Math { sin, cos }
```

### Modules standards

- **Math** : fonctions mathématiques
- **Texte** : manipulation de chaînes
- **Liste** : manipulation de liste
- **Dict** : manipulation de dict
- **ES** : entrées/sorties
- **JSON** : parsing JSON
- **Iter** : itérateurs avancés
- **Système** : accès système
- **Module** : permet de charger et créer des modules dynamiquements
- **Chemin** : gérer les chemins systèmes
- **Env** : gérer l'environnement
- **Processus** : permet de démarrer et de gérer des sous-processus
- **Aléatoire** : permet de gérer les nombres aléatoires
- **Projet** : permet de configurer des projets AliveScript (utilisé dans config.as)

### Exemple avec module JSON

```alivescript
utiliser JSON

var texte_json = '{"nom": "Alice", "age": 25}'
var donnees = JSON.charger(texte_json)

afficher donnees.nom    # "Alice"
afficher donnees.age    # 25
```

## Entrées/Sorties

### Lecture utilisateur

```alivescript
# Lire avec conversion
var age: entier = 0
lire entier dans age, "Quel est votre âge ? " sinon
    afficher "Âge invalide"
fin lire

# Lire texte simple
lire texte dans var nom, "Votre nom : "
```

### Fichiers

```alivescript
utiliser ES

# ouvrir un fichier
var fichier = essayer ES.ouvrir("donnees.txt", "l") sinon
  afficher "Erreur lors de l'ouverture du fichier"
  quitter()
fin essayer

# Lire un fichier
var contenu = fichier.lireTout()

# Écrire dans un fichier
ES.écrireFichier("sortie.txt", "Bonjour monde")
```

## Tests et assertions

```alivescript
utiliser Test

# Assertions simples
Test.affirmer(vrai, "Ceci devrait être vrai")
Test.affirmerÉgaux(2 + 2, 4, "L'addition échoue")

# Tests complexes
var lst = [1, 2, 3]
Test.affirmerÉgaux(lst.taille(), 3, "Longueur incorrecte")
```

## Syntaxe avancée

### Déclarations multiples

```alivescript
# Déclaration multiple
var a, b, c = 1, 2, 3

# Déclaration avec typage
var x: entier, y: décimal = 10, 3.14

# Liste de déclarations
var [premier, deuxieme] = [1, 2]
```

### Assignations composées

```alivescript
var x = 10
x += 5    # x = x + 5 (15)
x -= 3    # x = x - 3 (12)
x *= 2    # x = x * 2 (24)
x /= 3    # x = x / 3 (8.0)
```

### Expressions de type

```alivescript
# Alias de type
type EntierOuTexte = entier | texte

# Types génériques
var liste_entiers: liste<entier> = [1, 2, 3]
var dico_texte: dict<texte, texte> = {"cle": "valeur"}
```

## Commentaires

```alivescript
# Commentaire en ligne

(: 
   Commentaire 
   multiligne
:)

# Documentation de fonction
(-:
Calcule le carré d'un nombre
@param x Le nombre à mettre au carré
@return Le carré de x
:-)
fonction carre(x: entier) -> entier
    retourner x * x
fin fonction
```

## Bonnes pratiques

1. **Noms explicites** : Utilisez des noms de variables et de fonctions clairs
2. **Typage** : Précisez les types quand c'est utile pour la lisibilité
3. **Constantes** : Utilisez `const` pour les valeurs qui ne changent pas
4. **Fonctions pures** : Privilégiez les fonctions sans effets de bord
5. **Documentation** : Documentez les fonctions complexes avec `(-: ... :-)`

## Exemples complets

### Calculatrice simple

```alivescript
fonction calculer(a: décimal, op: texte, b: décimal) -> décimal
    quand op
        vaut "+" -> retourner a + b
        vaut "-" -> retourner a - b
        vaut "*" -> retourner a * b
        vaut "/" faire
            si b == 0 alors
                afficher "Erreur : division par zéro"
                retourner 0
            fin si
            retourner a / b
        sinon faire
            afficher "Opération inconnue : " + op
            retourner 0
    fin quand
fin fonction

afficher "Calculatrice simple"

lire décimal dans var a, "Premier nombre : "
lire texte dans var op, "Opération (+, -, *, /) : "
lire décimal dans var b, "Deuxième nombre : "

var resultat = calculer(a, op, b)
afficher "Résultat : " + texte(resultat)
```

### Compteur avec closure

```alivescript
fonction creer_compteur(depart: entier) -> fonction
    var compte = depart
    
    fonction incrementer() -> entier
        compte = compte + 1
        retourner compte
    fin fonction
    
    retourner incrementer
fin fonction

var compteur1 = creer_compteur(0)
var compteur2 = creer_compteur(100)

afficher compteur1()  # 1
afficher compteur1()  # 2
afficher compteur2()  # 101
afficher compteur1()  # 3
```

## Ressources

- **Grammaire** : `alivescript/src/alivescript.pest`
- **Exemples** : Fichiers `.as` dans le projet
- **Modules standards** : `alivescript/stdlib/`
- **Tests** : `alivescript/tests/`
