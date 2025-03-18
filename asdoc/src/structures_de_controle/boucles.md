# Boucles

## Mots clefs de contrôle

- `sortir`: met fin à la boucle immédiatement <small>(break en java/python)</small>
- `continuer`: remonte en haut de la boucle et passe à la prochaine itération <small>(continue en java/python)</small>

## Boucle `repeter`

La boucle `repeter` (ou `répéter`) permet d'itérer un certain nombre de fois prédéfini. S'il est
important de connaître la valeur de la variable d'itération, utilisez la boucle
`pour`.

Exemple où on affiche `Boujour!` 4 fois:

```as
repeter 4
    afficher "Bonjour!"
fin repeter
```

Syntaxe:

- ouverture: <code>repeter _valeurEntier_</code>
- fermeture: `fin repeter`

## Boucle `tant que`

Le corps de la boucle `tant que` va s'exécuter tant que la condition fournie ne sera
pas remplie.

Exemple où on exécute la boucle tant que la valeur de `i` n'est pas 5:

```as
var i = 0

tant que i != 5
    i = aleatoire(0..10)
    afficher i
fin tant que

afficher "fin"
```

Syntaxe:

- ouverture: <code>tant que _condition_ [faire]</code>
- fermeture: `fin tant que`

## Boucle `pour`

La boucle `pour` permet d'itérer sur tous les éléments d'une valeur itérable.

Exemples:

```as
pour var i dans 0..10
    afficher i
fin pour

pour var lettre dans "abcdef"
    afficher lettre
fin pour

pour var element dans [1, 2, 3, vrai, 4]
    afficher element
fin pour
```

Syntaxe:

- ouverture:
  - La variable itérée existe déjà:
    - <code>pour _variable_ dans _valeurIterable_</code>
  - La variable itérée n'existe pas:
    - <code>pour var _variable_ dans _valeurIterable_</code>
  - La variable itérée n'existe pas **ET** on veut empêcher la réaffectation dans le corps de la boucle:
    - <code>pour const _variable_ dans _valeurIterable_</code>
- fermeture: `fin pour`
