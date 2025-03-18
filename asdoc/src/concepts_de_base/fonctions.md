# Fonctions

AliveScript permet de définir des fonctions

## Syntaxe de déclaration de fonction

- Ouverture <code>fonction _nom_(_param1_, _param2_, _etc._)</code>
- Fermeture: `fin fonction`

## Paramètres

Pour définir les paramètres d'une fonction, on ajoute son nom à la liste de
paramètres dans la déclaration de la fonction.

### Typer un paramètre

Comme dans la définition d'une variable, il est possible de typer un paramètre avec la forme: <code>_nom_: _type_</code>

- Si le type de la valeur passée en argument n'est pas le bon, une erreur est lancée.
- Si non spécifié, un paramètre a le type `tout`, c.-à-d. qu'il accepte n'importe quel valeur.

Ex:

```as
fonction somme(x: entier, y: entier)
  retourner x + y
fin fonction
```

### Valeur par défaut

Il est possible de définir une valeur par défaut à un paramètre qui sera utilisée si
aucune valeur ne lui ait passée en argument. Syntaxe: <code>_param_ = _valeurParDefaut_</code>

- Les paramètres possédant des valeurs par défaut doivent venir après ceux n'en possédant pas

Ex:

```as
fonction somme(x: entier, y: entier = 1)
  retourner x + y
fin fonction

afficher somme(10, 23) # 33
afficher somme(4) # 5
```

## Retourner

### Syntaxe

<code>retourner _valeur_</code>

### Typer le retour

On peut spécifier le type que doit retourner une fonction en ajoutant à la fin de la déclaration <code>-> _type_</code>.

- Si le type de la valeur retournée n'est pas le bon, une erreur est lancée.
- Si non spécifié, le type de retour est `tout`, c.-à-d. que la fonction peut retourner n'importe quel type de valeur.

Ex:

```as
fonction somme(x: entier, y: entier) -> entier
  retourner x + y
fin fonction
```
