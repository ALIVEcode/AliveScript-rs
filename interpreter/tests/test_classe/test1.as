utiliser Test {affirmer, affirmerEgal}

classe Personne
    nom: texte
    age: entier
    ami: Personne? = nul
    ami2: Personne? = nul

    init(nom: texte, age: entier)
        inst.nom = nom
        inst.age = age
    fin init

    methode getNom()
        retourner inst.nom 
    fin methode

    methode getAge()
        retourner inst.age
    fin methode

    methode setAmi(ami: Personne)
        inst.ami = ami
    fin methode 

    methode getAmi()
        retourner inst.ami 
    fin methode

fin classe


mathis = Personne("Mathis", 20)
enric = Personne("Enric", 20)

afficher mathis.getAge()
mathis.age += 1
afficher mathis.getAge()

afficher mathis.getAmi()
mathis.setAmi(enric)
afficher mathis
afficher mathis.getAmi().getNom()
mathis.getAmi().setAmi(mathis)
mathis.ami2 = mathis
afficher enric.getAmi()
enric.ami2 = enric
afficher enric

x = [1, 2, 3]
x[2] = x
x[2][0] = 7
afficher x[2][2][2]

