utiliser ES


structure JSONParser
  compteur: entier = 0
  txt: texte
fin structure

impl JSONParser
  méthode parseValeur(inst)
    tant que non inst.estFini() faire
      inst.passer_espace()
      prochain = inst.inspecter()
      si prochain == "{" alors retourner inst.parseObj()
      sinon si prochain == "[" alors retourner inst.parseTableau()
      sinon si prochain == '"' alors retourner inst.parseTexte()
      sinon retourner inst.parseLiteral()
    fin tant que
  fin méthode

  méthode parseObj(inst)
    var obj = {}
    # consumme soit "{", soit "," ou soit "}"
    tant que inst.prochain() != "}"
      clé = inst.parseTexte()
      # on passe le ":"
      inst.prochain()
      val = inst.parseValeur()

      obj[clé] = val
    fin tant que

    retourner obj
  fin méthode

  méthode parseTableau(inst)
    var tab = []
    # consumme soit "[", soit "," ou soit "]"
    tant que inst.prochain() != "]"
      tab.ajouter(inst.parseValeur())
    fin tant que

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
    si premier.estNumérique() alors 
      chiffres = premier
      tant que inst.inspecter().estNumérique()
        chiffres += inst.prochain()
      fin tant que

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


src = '
{
  "first_name": "John",
  "last_name": "Smith",
  "is_alive": true,
  "age": 27,
  "address": {
    "street_address": "21 2nd Street",
    "city": "New York",
    "state": "NY",
    "postal_code": "10021-3100"
  },
  "phone_numbers": [
    {
      "type": "home",
      "number": "212 555-1234"
    },
    {
      "type": "office",
      "number": "646 555-4567"
    }
  ],
  "children": [
    "Catherine",
    "Thomas",
    "Trevor"
  ],
  "spouse": null
}
'

parseur = JSONParser {txt: src}

d = parseur.parseValeur()
afficher d

afficher d.children


