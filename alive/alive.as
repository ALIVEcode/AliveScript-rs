#!alivec

utiliser Chemin
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
const CHEMIN_CONFIG = Chemin.créer("config.as")
const VERSION_ALIVESCRIPT = "0.1.0"

fonction chargerConfig(chemin: texte)
    const mod = Module.configurer(chemin, {
      modulesPermis: ["Projet"],
      actionsPermises: ["écrireSortieStd"],
    }).charger()

    const config = mod.__AS_PROJET
    config.source = Chemin.créer(config.source)
fin fonction

fonction init()
  si non CHEMIN_CONFIG.existe() alors 
    var fichier = CHEMIN_CONFIG.ouvrir("écriture")

    var contenu = `
      utiliser Projet

      Projet.configurer({{
        nom: "{}",
        version: "{}",
        versionAliveScript: "{}",
        source: "{}",
      }})
      `

    var nom = Env.dossierDeTravail().nom()
    var version = "0.1.0"
    var source = "src/init.as"
    fichier.écrire(contenu.format([nom, version, VERSION_ALIVESCRIPT, source]))
  sinon
    afficher "Le fichier de configuration existe déjà."
  fin si

  config = chargerConfig(CHEMIN_CONFIG)
  source = config.source
  si non source.existe() alors
    const parent = chemin.parent()
    parent.créerDossier()

    var fichierSource = source.ouvrir("écriture")
    fichierSource.écrire(`
    afficher "Bonjour d'AliveScript !"
    `)
  fin si
fin fonction

fonction rechercheModule(chemin: texte)
  utiliser Chemin
  utiliser ES
  utiliser Env
  utiliser Système alias Sys
  utiliser Module
  utiliser Processus

  const fichier = Chemin.créer("modules") / chemin / "config.as"
  si fichier.existe() alors
    const config = essayer Module.configurer(fichier, {
      modulesPermis: ["Projet"],
      actionsPermises: ["écrireSortieStd"],
    }).charger().__AS_PROJET

    const source = "modules/{}/{}".format([chemin, config.source])
    retourner essayer Module.charger(source)
  fin si
fin fonction

fonction exec()
  # trouver config.as
  si non CHEMIN_CONFIG.existe() alors erreur("Impossible de trouver '{}'.".format([CHEMIN_CONFIG]))
  config = essayer chargerConfig(CHEMIN_CONFIG) sinon 
    erreur "Impossible de charger le fichier de configuration"
  fin essayer

  source = config.source

  const mod = Module.configurer(source, {})
  mod.rechercheModule(rechercheModule)

  mod.charger()
fin fonction

fonction gérerDepUrl(dep: dict)
  const url = dep.url
  const nom = dep.nom.raser()

  si nom == "" alors erreur "Le nom ne doit pas être vide"

  const dossierModule = Env.dossierDeTravail() / "modules" / nom

  var existe = faux
  si dossierModule.existe() alors existe = vrai

  si ".git" dans url alors
    afficher "| Installation de '{}' ('{}')".format([nom, url])

    const p = quand existe 
      vaut vrai -> Processus.créer("git", ["pull"], dossierModule)
      vaut faux -> Processus.créer("git", ["clone", url, dossierModule])
      sinon -> !
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
  config = chargerConfig(CHEMIN_CONFIG)

  ES.créerDossier("modules")

  afficher "Installation des dépendances de '{}'".format([config.nom])

  si "dépendances" dans config.clés() alors
    const deps = config.dépendances
    pour chaque dep dans deps 
      quand dep 
        est dict -> gérerDepUrl(dep)
        sinon -> erreur "Chaque dépendances doit être un dict avec les clés \"nom\" et \"url\""
      fin quand
    fin pour
  fin si

  afficher "Dépendances de '{}' installées !".format([config.nom])
fin fonction

fonction ajouter()
  # trouver config.as
  si non ES.existe(CHEMIN_CONFIG) alors erreur("Impossible de trouver 'config.as'.")
  const config = chargerConfig(CHEMIN_CONFIG)

  si tailleDe(args) < 4 alors erreur("Url manquante")
  const url = args[3]

  var nom = ""
  si tailleDe(args) > 4 alors 
    nom = args[4]
  sinon
    nom = url.diviser("/")[-1].sansSuffix(".git").raser()
  fin si

  afficher nom

  si "dépendances" dans config.clés() alors
    const deps = config.dépendances
    pour chaque dep dans deps 
      si dep.nom == nom alors 
        erreur "Une autre débendance a déjà le nom '{}' (url='{}')".format([nom, dep.url])
      fin si
      si dep.url == url alors 
        erreur "Cette débendance est déjà dans le projet '{}' (nom='{}')".format([nom, dep.url])
      fin si
    fin pour
  fin si

  const fichier = essayer ES.ouvrir(CHEMIN_CONFIG, "ajout") sinon
    erreur "Impossible d'ouvrir le fichier de configuration"
  fin essayer

  fichier.écrire('Projet.ajouterDépendance({{ nom: "{}", url: "{}" }})'.format([
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
    sinon avec autre -> erreur "Commande inconnue: {}".format([autre])
  fin quand
fin fonction


départ()
