# Opérateurs

## Opérateurs arithmétiques

- addition: `+`

  - nombre + nombre -> nombre
    - ex: `2 + 2 = 4`
  - liste + tout -> ajoute le deuxième terme à la fin de la liste
    - ex: `[2, 3, 4] + "salut" = [2, 3, 4, "salut"]`
  - texte + tout -> concationne le texte avec la représentation en texte du deuxième terme
    - ex: `"salut " + vrai = "salut vrai"`
  - (tout sauf liste) + texte -> concationne le texte avec la représentation en texte du premier terme
    - ex: `12 + "hey" = "12hey"`

- soustraction: `-`

  - nombre - nombre -> nombre
    - ex: `12 - 2 = 10`
  - liste - tout -> retire tous les éléments de la liste qui sont égaux au deuxième terme
    - ex: `[1, vrai, "bonjour"] - vrai = [1, "bonjour"]`
  - texte - texte -> retire tous les sous-textes du premier terme matchant au deuxième terme
    - ex: `"salut je suis Mathis" - "is" = "salut je su Math`

- multiplication: `*`

  - nombre \* nombre -> nombre
    - ex: `4 * 12 = 48`
  - texte \* entier -> répète le texte un nombre de fois égal au deuxième terme
    - ex: `"salut " * 3 = "salut salut salut "`
  - liste \* entier -> répète la liste un nombre de fois égal au deuxième terme
    - ex: `[12, vrai, "h"] * 3 = [12, vrai, "h", 12, vrai, "h", 12, vrai, "h"]`

- division: `/`

  - nombre / nombre -> nombre
    - ex: `25 / 2 = 12.5`
  - liste / liste -> retire tous les éléments de la liste 1 présent dans la liste 2
    - ex: `[1, 1, 2, vrai, faux, 4, 5, "foo"] / [vrai, 2, 4, 1] = [faux, 5, "foo"]`

- division entière: `//`

  - nombre // nombre -> entier
    - ex: `25 / 2 = 12`

- exposant: `^` ou `**`

  - nombre ^ nombre -> nombre
    - ex: `3 ^ 4 = 81`

- modulo: `%`

  - entier % entier -> entier
    - ex: `7 % 3 = 1`

- pipe: `|`
  - liste | liste -> créé une nouvelle liste composé des éléments des deux listes sans modifier les listes originales
    - ex: `[1, 2, 3, vrai] | [3, faux, "salut"] = [1, 2, 3, vrai, 3, faux, "salut"]`

## Opérateurs de comparaisons

- égal: `==`
- pas égal: `!=`
- plus grand: `>`
- plus petit: `<`
- plus grand ou égal: `>=`
- plus petit ou égal: `<=`
- dans: _valeur_ `dans` _iterable_
- pas dans: _valeur_ `pas dans` _iterable_

## Opérations sur les itérables

- Indexation:

  - Obtenir valeur à l'index: <code>_variable_\[_index_]</code>
  - Affecter valeur à l'index: <code>_variable_\[_index_] = _valeur_</code>

- Sous section:
  - Obtenir valeurs dans le range: <code>_variable_\[_debut_:_fin_]</code>
  - Affecter valeurs dans le range: <code>_variable_\[_debut_:_fin_] = _valeurIterable_</code>

## Opérateurs de séries (range)

- Suite exclusive:

  - Syntaxe: `debut .. fin`
  - Ex:
    - `0 .. 10 == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]`
    - `10 .. 3 == [10, 9, 8, 7, 6, 5, 4]`
    - `"a" .. "f" == ["a", "b", "c", "d", "e"]`

- Suite inclusive:
  - Syntaxe: `debut ..= fin`
  - Ex:
    - `0 ..= 10 == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]`
    - `10 ..= 3 == [10, 9, 8, 7, 6, 5, 4, 3]`
    - `"a" ..= "f" == ["a", "b", "c", "d", "e", "f"]`

## Opérateurs logiques

- et:
  - syntaxe: _valeur1_ `et` _valeur2_
  - fonctionnement: retourne <code>_valeur1_</code> si <code>_valeur1_</code> est **faux**, sinon retourne <code>_valeur2_</code>
- ou:
  - syntaxe: _valeur1_ `ou` _valeur2_
  - fonctionnement: retourne <code>_valeur1_</code> si <code>_valeur1_</code> est **vrai**, sinon retourne <code>_valeur2_</code>

## Opérateurs unaires

- négation:

  - syntaxe: `-`_valeur_
  - fonctionnement: inverse le signe de _valeur_

- `pas` et `non`:
  - syntaxe: `pas` _valeur_ OU `non` _valeur_
  - fonctionnement: retourne la valeur booléenne **inverse** de <code>_valeur_</code>
