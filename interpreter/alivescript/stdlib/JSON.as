utiliser ES

structure _JSONParser
  compteur: entier = 0
  txt: texte
fin structure

impl _JSONParser
  méthode parseValeur(inst)
    si inst.estFini() alors retourner nul

    inst.passer_espace()
    prochain = inst.inspecter()
    si prochain == "{" alors retourner inst.parseObj()
    sinon si prochain == "[" alors retourner inst.parseTableau()
    sinon si prochain == '"' alors retourner inst.parseTexte()
    sinon retourner inst.parseLiteral()
  fin méthode

  méthode parseObj(inst)
    var obj = {}
    # consumme "{"
    inst.prochain()

    tant que inst.inspecter() != "}"
      clé = inst.parseTexte()
      # on passe le ":"
      inst.prochain()
      val = inst.parseValeur()

      obj[clé] = val
      inst.passer_prochain_si(",")
    fin tant que

    # consumme "}"
    inst.prochain()
    retourner obj
  fin méthode

  méthode parseTableau(inst)
    var tab = []
    # consumme "["
    inst.prochain()

    tant que inst.inspecter() != "]"
      tab.ajouter(inst.parseValeur())
      inst.passer_prochain_si(",")
    fin tant que

    # consumme "]"
    inst.prochain()
    retourner tab
  fin méthode

  méthode parseTexte(inst)
    # on passe le '"'
    inst.prochain()
    txt = ""
    tant que inst.inspecter() != '"' faire
      car = inst.prochain()
      si car == "\\" alors
        var flag = inst.prochain()
        si flag == "n" alors car = "\n"
        sinon si flag == "t" alors car = "\t"
        sinon si flag == "\\" alors car = "\\"
        sinon si flag == "r" alors car = "\r"
        sinon si flag == '"' alors car = '"'
        sinon si flag == "'" alors car = "'"
        sinon erreur("Flag inconnu '" + flag + "'")
      fin si
      txt += car
    fin tant que

    # on passe le '"'
    inst.prochain()
    retourner txt
  fin méthode

  méthode parseLiteral(inst)
    premier = inst.prochain()
    si premier.estNumérique() ou premier == "-" alors 
      chiffres = premier
      tant que inst.inspecter().estNumérique() ou inst.inspecter() == "."
        chiffres += inst.prochain()
      fin tant que

      si "." dans chiffres alors retourner décimal(chiffres)
      retourner entier(chiffres)

    sinon si premier == "t" alors 
      répéter 3
        # rue
        inst.prochain()
      fin répéter

      retourner vrai
    sinon si premier == "f" alors 
      répéter 4
        # alse
        inst.prochain()
      fin répéter

      retourner faux
    sinon si premier == "n" alors 
      répéter 3
        # ull
        inst.prochain()
      fin répéter

      retourner nul

    sinon
      erreur("caractère inconnu: " + premier)
    fin si
  fin méthode

  méthode inspecter(inst)
    si inst.estFini() alors retourner nul

    retourner inst.txt[inst.compteur]
  fin méthode

  méthode passer_espace(inst)
    tant que inst.inspecter() dans ["\n", " "]
      inst.compteur += 1
    fin tant que
  fin méthode

  méthode passer_prochain_si(inst, car: texte)
    inst.passer_espace()
    si inst.estFini() alors retourner nul

    val = inst.txt[inst.compteur]
    si val == car alors inst.compteur += 1
  fin méthode

  méthode prochain(inst)
    inst.passer_espace()
    si inst.estFini() alors retourner nul

    val = inst.txt[inst.compteur]
    inst.compteur += 1
    retourner val
  fin méthode

  méthode estFini(inst)
    retourner tailleDe(inst.txt) == inst.compteur
  fin méthode

  méthode estDernier(inst)
    retourner tailleDe(inst.txt) == inst.compteur + 1
  fin méthode

fin impl

fonction textifier(val) -> texte
  quand val
    est texte faire
    sinon -> !
  fin quand
fin fonction

fonction charger(txtJSON: texte)
  retourner _JSONParser {txt: txtJSON}.parseValeur()
fin fonction
