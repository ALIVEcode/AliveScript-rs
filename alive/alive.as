#!alivec

utiliser ES
utiliser Env
utiliser Système alias Sys
utiliser Module
utiliser Processus

fonction aide()
  afficher "Gestionnaire de Projets d'AliveScript

COMMANDES:
 init : initialise un projet AliveScript
 exec : exécute un projet AliveScript
"
fin fonction

fonction aff(msg: texte) 
  ES.sortieStd().écrire(msg)
fin fonction

const args = Env.args()

si tailleDe(args) == 2 alors
  aide()
  retourner
fin si

const commande = args[2]
const CHEMIN_CONFIG = "config.as"

fonction init()
  si non ES.existe(CHEMIN_CONFIG) alors 
    var fichier = ES.ouvrir(CHEMIN_CONFIG, "écriture")

    var contenu = `
      nom = "{}"
      version = "{}"
      
      src = "{}"
      versionAliveScript = "0.1.0"
      
      dépendances = []
      `

    var nom = Env.dossierDeTravail().diviser("/")[-1]
    var version = "0.1.0"
    var source = "src/init.as"
    fichier.écrire(contenu.format([nom, version, source]))
  sinon
    afficher "Le fichier de configuration existe déjà."
  fin si

  config = Module.charger(CHEMIN_CONFIG)
  source = config.src
  si non ES.existe(source) alors
    var chemin = source.diviser("/")
    var parent = chemin[0..tailleDe(chemin) - 1]
    ES.créerDossier(parent.joindre("/"))

    var fichierSource = ES.ouvrir(source, "écriture")
    fichierSource.écrire(`
    afficher "Bonjour d'AliveScript !"
    `)
  fin si
fin fonction

fonction exec()
  # trouver config.as
  si non ES.existe(CHEMIN_CONFIG) alors erreur("Impossible de trouver '{}'.".format([CHEMIN_CONFIG]))
  config = essayer Module.charger(CHEMIN_CONFIG) sinon 
    erreur "Impossible de charger le fichier de configuration"
  fin essayer

  source = config.src

  const mod = Module.créer(source)

  mod.rechercheModule(fn(chemin)
    utiliser Module
    utiliser ES
    const fichier = "modules/{}/config.as".format([chemin])
    si ES.existe(fichier) alors
      const config = essayer Module.charger(fichier)
      const source = "modules/{}/{}".format([chemin, config.src])
      retourner essayer Module.charger(source)
    fin si
  fin fn)

  mod.charger()
fin fonction

fonction gérerDepUrl(dep: dict)
  const url = dep.url
  const nom = dep.nom.raser()

  si nom == "" alors erreur "Le nom ne doit pas être vide"

  const dossierModule = "{}/modules/{}".format([Env.dossierDeTravail(), nom])

  var existe = faux
  si ES.existe("modules/{}".format([nom])) alors existe = vrai

  si ".git" dans url alors
    afficher "| Installation de '{}' ('{}')".format([nom, url])

    const p = quand existe 
      vaut vrai -> Processus.créer("git", ["pull"], dossierModule)
      vaut faux -> Processus.créer("git", ["clone", url, dossierModule])
    fin quand

    out, err = p.execAvecSortie()
    si err alors aff "> " + err
    si out alors 
      aff "  > " + out
    fin si

    const exe = Env.cheminExec()
    const alive = args[1]
    const p = Processus.créer(exe, [alive, "installer"], dossierModule)
    out, err = p.execAvecSortie()
    si err alors aff err
    si out alors 
      afficher out.diviser("\n").map(fn(ln): "  " + ln).joindre("\n")
    fin si
    
  sinon
    erreur "L'url doit être un repo git"
  fin si
fin fonction

fonction installer()
  # trouver config.as
  si non ES.existe(CHEMIN_CONFIG) alors erreur("Impossible de trouver 'config.as'.")
  config = Module.charger(CHEMIN_CONFIG)
  deps = config.dépendances

  ES.créerDossier("modules")

  afficher "Installation des dépendances de '{}'".format([config.nom])
  pour chaque dep dans deps 
    quand dep 
      est dict -> gérerDepUrl(dep)
      sinon -> erreur "Chaque dépendances doit être un dict avec les clés \"nom\" et \"url\""
    fin quand
  fin pour

  afficher "Dépendances de '{}' installées !".format([config.nom])
fin fonction

fonction ajouter()
  # trouver config.as
  si non ES.existe(CHEMIN_CONFIG) alors erreur("Impossible de trouver 'config.as'.")
  const config = Module.charger(CHEMIN_CONFIG)
  const deps = config.dépendances

  si tailleDe(args) < 4 alors erreur("Url manquante")
  const url = args[3]

  var nom = ""
  si tailleDe(args) >= 4 alors 
    nom = args[4]
  sinon
    nom = (url.diviser("/")[-1]).sansSuffix(".git").raser()
  fin si

  afficher nom

  pour chaque dep dans deps 
    si dep.nom == nom alors 
      erreur "Une autre débendance a déjà le nom '{}' (url='{}')".format([nom, dep.url])
    fin si
  fin pour

  const fichier = essayer ES.ouvrir(CHEMIN_CONFIG, "ajout") sinon
    erreur "Impossible d'ouvrir le fichier de configuration"
  fin essayer

  fichier.écrire('\ndépendances.ajouter({{ nom: "{}", url: "{}" }})'.format([
    nom,
    url,
  ]))

  installer()
fin fonction

fonction départ()
  quand commande
    vaut "init" -> init()
    vaut "exec" -> exec()
    vaut "installer", "i" -> installer()
    vaut "ajouter", "a" -> ajouter()
  fin quand
fin fonction


départ()
