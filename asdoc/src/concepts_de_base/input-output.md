# Entrées et Sorties (I/O)

## Sorties

### Afficher

Pour produire des sorties, on utilise la fonction `afficher`, que l'on peut appeler
sans parenthèses.

Ex:

```as
afficher 12

var x = "Monde"
afficher "Bonjour, " + x + "!"
```

> Attention, en cas d'ambiguïté, il faut mettre la valeur à afficher entre parenthèse
> Ex:
>
> ```as
> afficher [1] # Erreur, car ambiguë.
> afficher([1]) # Ok
> ```

## Entrées

### Lire

La commande `lire` d'AliveScript permet de lire une entrée fournie par l'utilisateur
et de la stocker dans une variable. Elle possède plusieurs formes décritent ci-dessous.

> À noter: si la variable n'a pas été déclarée avant la commande `lire`, il faut
> utiliser `var variable` plutôt que seulement `variable`.
> Ex:
>
> ```as
> var x
> lire x # Ok
>
> lire var y # Ok, car on déclare la variable y en même temps
>
> lire z # Erreur, la variable z n'a pas été déclarée avant
> ```

#### Forme de base:

La forme la plus simple de lire:

```as
lire var x
```

Si on veut un message personnalisé:

```as
lire var x, "Mon message"
```

Si on veut transformer le type de la valeur résultante avant l'affectation:

```as
lire entier dans var age

fonction split(s: texte) -> liste
    var mots = [""]
    pour var lettre dans s
        si lettre == " "
            mots += ""
        sinon
            mots[-1] += lettre
        fin si
    fin pour

    retourner mots
fin fonction

lire split dans var liste_mots

# Si on entre `un mot est si vite arrivé`, on obtiendra la valeur
# `["un", "mot", "est", "si", "vite", "arrivé"]` dans liste_mots
```

Si on veut transformer le type de la valeur résultante avant l'affectation et mettre un message personnalisé:

```as
lire entier dans var age, "Entrez votre âge:"
```
