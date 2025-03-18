# Commentaires

AliveScript supporte plusieurs types de commentaires différents.

### Commentaires simples lignes

Le symbole `#` dénote le début d'un commentaire simple ligne

Syntaxe:

```as
# commentaire
```

On peut aussi ajouter un commentaire simples lignes après une instruction

```as
afficher 12 # cette ligne affiche la valeur 12
```

### Commentaires multilignes

Alivescript supporte deux types de commentaires multilignes.

Syntaxe du premier type:

- ouverture: `(:`
- fermeture: `:)`

```as
(:
 Salut
 je
 suis
 un
 commentaire multiligne!
:)
```

Syntaxe du deuxième type (style C):

- ouverture: `/*`
- fermeture: `*/`

```as
/*
 Salut
 je
 suis
 un
 commentaire multiligne!
*/
```

### Commentaires de documentation

AliveScript supporte des commentaires spéciaux de documentation qui peuvent
être ajoutés au-dessus de la déclaration d'une fonction ou d'une classe.

Syntaxe:

- ouverture: `(-:`
- fermeture: `:-)`

```as
(-:
 - Cette fonction additionne deux nombres et retourne le résultat
 - @param num1: le premier nombre
 - @param num2: le deuxième nombre
 - @retourne la somme deux deux nombres
:-)
fonction additionner(num1: nombre, num2: nombre) -> nombre
  retourner num1 + num2
fin fonction
```
