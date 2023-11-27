utiliser Math

(-:
    La fonction additionVecteur prend deux vecteurs de même taille et retourne
    un vecteur qui est la somme des deux vecteurs passés en paramètre.

    @param v1: liste de nombres
    @param v2: liste de nombres

    @retourne liste de nombres ou nul si les vecteurs n'ont pas la même taille

    Exemple:
        additionVecteur([1, 2, 3], [4, 5, 6]) retourne [5, 7, 9]
:-)
fonction additionVecteur(v1: liste, v2: liste) -> liste?
    retourner nul si tailleDe(v1) != tailleDe(v2)

    pour i dans 0..tailleDe(v1)
        v1[i] = v1[i] + v2[i]
    fin pour

    retourner v1
fin fonction

afficher 3..1

