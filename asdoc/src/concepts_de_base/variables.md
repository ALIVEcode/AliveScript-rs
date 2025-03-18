# Les variables

Les variables sont un outil essentiel dans tout langage de programmation.
Nous verrons ici comment il est possible de déclarer et de réaffecter des
variables en AliveScript.

## Nom de variable

Les noms de variables peuvent posséder des lettres (toutes les lettres unicodes), des underscordes (`_`)
et des chiffres, mais _ne peuvent pas commencer par un chiffre_.

## Déclaration

Pour déclarer une _nouvelle_ variable, on utilise le mot clef `var` suivi par le
nom de la variable puis du symbole d'affectation (`=` ou `<-`) puis de la valeur.

Ex:

```as
var estValide = vrai
# OU
var items <- ["chandelier", "chaise", "manteau"]
```

> Si le type de la variable n'est _pas_ spécifiée, la variable est de type `tout`
> et on peut alors on peut changer la valeur de la variable peut importe le type.
>
> ```as
> var x = 12
> x = 88 # Ok
> x = "hey!" # Ok, x est de type `tout`
> ```

Il est aussi possible de spécifier le **type** de valeur acceptée par la variable
en ajoutant le symbole `:` suivi du type après le nom de la variable.
Ex:

```as
var age: entier = 24
# OU
var couleur: texte <- "rouge"
```

> Si le type de la variable est spécifiée, essayer d'affecter une valeur incompatible
> à la variable causera une erreur dans le programme.
>
> ```as
> var x: entier = 12
> x = 88 # Ok
> x = "hey!" # ERREUR, mauvais type!
> ```

### Constantes

Il est aussi possible de déclarer une variable comme une constante, c.-à-d. que
sa valeur ne pourra pas être changé après sa définition. La syntaxe est la même
que pour la déclaration de variable, mais on va utiliser `const` au lieu de `var`.

Ex:

```as
# Sans type
const PI = 3.14159265
const DEBUG <- faux

# Avec type
const TAXES: décimal = 14.975
const CHIFFRES: texte = "1234567890"
```

> Note: par convention, on nomme les constantes en lettres majuscules.

## Affectation

Une fois une variable déclarée, on peut changer la valeur qu'elle contient grâce
à une affectation (ou une réaffectation).

Pour affecter une variable à une nouvelle valeur, on utilise la forme:

```as
variable = valeur
# OU
variable <- valeur
```

Ex:

```as
var x = 12
x = "salut"
x <- 88
```

### Affectation avec opérateur arithmétique

Il est aussi possible d'utiliser la forme <code>_variable_ **operation**= _valeur_</code>
pour remplacer la forme plus longue <code>_variable_ = _variable_ **operation** _valeur_</code>

- <code>_variable_ += _valeur_</code>
- <code>_variable_ -= _valeur_</code>
- <code>_variable_ \*= _valeur_</code>
- <code>_variable_ /= _valeur_</code>
- <code>_variable_ //= _valeur_</code>
- <code>_variable_ %= _valeur_</code>
- <code>_variable_ ^= _valeur_</code> ou <code>_variable_ \*\*= _valeur_</code>
