# Valeurs et Types de données

Toutes les données/valeurs qu'il est possible d'écrire en AliveScript ont ce que
l'on appelle un `type`. Un type, c'est comme une étiquette que l'on appose sur
chaque valeur afin de décrire sa nature et, ainsi, pouvoir la regrouper avec les
autres valeurs partageant les mêmes caractéristiques. Par exemple, en
AliveScript, toutes les valeurs numériques (nombres entiers et nombre décimaux)
ont le type `nombre`.

Une valeur peut même satisfaire _plusieurs_ types en même temps. Par exemple, un
nombre entier (`12`, `18`, `-36`, `0`, etc.) est de type `nombre`, mais aussi de
type **`entier`**.

## Les types primitifs

Les types de données _primitifs_ représentent les valeurs de base qui composent
AliveScript. Ils sont au compte de 7: `nul`, `entier`, `décimal`, `texte`,
`booléen`, `fonction` et `structure`.

| Type de données     | Caractéristiques                                                                        | Exemple                          |
| ------------------- | --------------------------------------------------------------------------------------- | -------------------------------- |
| `nul`               | La seule valeur de ce type est la valeur `nul`. Représente souvent l'absence de valeurs | `nul`                            |
| `entier`            | `-2^63` &leq; x < `2^63`                                                                | `-1`, `34324`, `0`, `-10212`     |
| `décimal`/`decimal` | Nombre flottant double précision IEEE-754                                               | `23.2`, `-0.1212`, `9823.2223`   |
| `texte`             | Chaîne de caractères UTF-8                                                              | `""`, `"école"`, `"bonjour! 😃"` |
| `booléen`/`booleen` | Valeurs représentant vrai ou faux                                                       | `vrai`, `faux`                   |
| `fonction`          | Valeurs encapsulant un comportement et pouvant être appelées                            | Voir [définition de Fonctions]() |
| `structure`         | Type englobant toutes les structures                                                    | Voir [les Structures]()          |

## Structures de données primitives

Les structures de données primitives sont les structures de données définies à
même le langage. Il y en a 3: `liste`, `paire` et `dict`.

| Type de données | Caractéristiques                                               | Exemple                                                                             |
| --------------- | -------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `liste`         | Une liste d'éléments, hétérogène et de taille dynamique        | `[1, "a", vrai]`, `[]`, `["allo", [2, [3], x]]`                                     |
| `paire`         | Une association entre un élément `clef` et un élément `valeur` | `"couleur": "rouge"`, `"age": 28`, `[1, 2]: [3, 4]`                                 |
| `dict`          | Un ensemble de `paire`s qui respecte l'ordre d'insertion       | `{"nom": "Mathis", "age": 21}`, `{}`, `{"infos": {"couleur": "rouge", "prix": 23}}` |
