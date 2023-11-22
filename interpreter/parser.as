utiliser Math

fonction additionVecteur(v1: liste, v2: liste) -> liste?
    retourner nul si tailleDe(v1) != tailleDe(v2)

    pour i dans 0..tailleDe(v1)
        v1[i] = v1[i] + v2[i]
    fin pour

    retourner v1
fin fonction


afficher additionVecteur([1, 2, 3], [4, 5, 6])
afficher Math.sin(Math.PI / 2)

