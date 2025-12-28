#!alive

utiliser ES
utiliser Env
utiliser Système alias Sys
utiliser Module

fonction aide()
  afficher "Gestionnaire de Packets d'AliveScript

COMMANDES:
 init : initialise un projet AliveScript
 exec : exécute un projet AliveScript
"
fin fonction

const args = Sys.args()

si tailleDe(args) == 2 alors
  aide()
  retourner
fin si

const commande = args[2]

fonction init()
  cheminConfig = "config.as"
  #si ES.existe(cheminConfig) alors erreur("Le fichier de configuration existe déjà.")

  var fichier = ES.ouvrir(cheminConfig, "écriture")

  var contenu = `
    nom = "{}"
    version = "{}"
    
    src = "{}"
    versionAliveScript = "0.1.0"
    
    dépendances = []
    `

  var nom = Env.dossierActuel().diviser("/")[-1]
  var version = "0.1.0"
  var source = "src.as"
  fichier.écrire(contenu.format([nom, version, source]))

  si non ES.existe(source) alors
    var fichierSource = ES.ouvrir(source, "écriture")
    fichierSource.écrire(`
    afficher "Bonjour d'AliveScript !"
    `)
  fin si
  
fin fonction

fonction exec()
  # trouver config.as
  si non ES.existe("config.as") alors erreur("Impossible de trouver 'config.as'.")
  config = Module.charger("config.as")
  source = config.src

  Module.charger(source)
fin fonction()

si commande == "init" alors
  init()
sinon si commande == "exec" alors
  exec()
fin si



