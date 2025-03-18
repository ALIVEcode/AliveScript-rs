# Modules

AliveScript organise les bibliothèques de code qu'il dispose en `module`s.

Chaque module possède un nom et peut être `utiliser`.

## La commande `utiliser`

Pour ajouter un module dans un programme, on utilise la commande `utiliser`.

### Utiliser un module au complet

Syntaxe: <code>utiliser _nomModule_</code>

Ex:

```as
utiliser Math

afficher Math.PI
afficher Math.sin(1)
```

### Ajouter tous le contenu d'un module dans le programme

Syntaxe: <code>utiliser _nomModule_ {\*}</code>

Ex:

```as
utiliser Math {*}

afficher PI
afficher sin(1)
```

### Utiliser une partie d'un module

Syntaxe: <code>utiliser _nomModule_ {_nom1_, _nom2_, etc.}</code>

Ex:

```as
utiliser Math {PI, sin}

afficher PI
afficher sin(1)
afficher cos(1) # Erreur
```

### Utiliser un module avec alias

Syntaxe: <code>utiliser _nomModule_ _alias_.{\*}</code>

Ex:

```as
utiliser Math M.{*}

afficher M.PI
afficher M.sin(1)
```

### Utiliser un module avec alias et juste certains éléments

Syntaxe: <code>utiliser _nomModule_ _alias_.{_nom1_, _nom2_, etc.}</code>

Ex:

```as
utiliser Math M.{PI, sin}

afficher M.PI
afficher M.sin(1)
afficher M.cos(1) # Erreur
```
