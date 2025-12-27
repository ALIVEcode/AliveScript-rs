utiliser ES

structure JSONParser
  compteur: entier = 0
  txt: texte
fin structure

impl JSONParser
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


fonction test()
  utiliser Test

  src = '{
    "metadata": {
      "version": "2.4.0",
      "generated_at": "2025-12-27T13:43:03Z",
      "environment": "production-test",
      "checksum": "a7b8c9d0e1f2g3h4i5j6",
      "total_records": 1500
    },
    "organization": {
      "name": "Nexus Dynamics Global",
      "ticker": "NXDG",
      "headquarters": {
        "address": "101 Innovation Way",
        "city": "San Francisco",
        "state": "CA",
        "postal_code": "94105",
        "coordinates": {
          "lat": 37.7749,
          "lng": -122.4194
        }
      },
      "departments": [
        {
          "id": "dept_001",
          "name": "Research & Development",
          "budget": 4500000.50,
          "active_projects": [
            {
              "project_id": "PRJ-992",
              "title": "Quantum Neural Link",
              "status": "in_progress",
              "tags": ["AI", "Quantum", "Bio-tech"],
              "milestones": [
                {"id": 1, "task": "Initial Prototyping", "completed": true},
                {"id": 2, "task": "Beta Testing Phase I", "completed": false}
              ]
            }
          ]
        },
        {
          "id": "dept_002",
          "name": "Infrastructure",
          "budget": 2100000.00,
          "active_projects": []
        }
      ]
    },
    "users": [
      {
        "id": "u_88219",
        "guid": "550e8400-e29b-41d4-a716-446655440000",
        "is_active": true,
        "balance": "$3,450.12",
        "picture": "http://placehold.it/32x32",
        "age": 34,
        "eye_color": "green",
        "name": {
          "first": "Elena",
          "last": "Rodriguez"
        },
        "company": "NEXUS",
        "email": "elena.rodriguez@nexus.io",
        "phone": "+1 (985) 555-0192",
        "address": "789 Maple Terrace, Gotham, NY 10001",
        "about": "Expert in distributed systems and cloud architecture with over 10 years of experience.",
        "registered": "2018-05-12T10:45:00 -08:00",
        "permissions": ["admin", "editor", "billing"],
        "login_history": [
          {"timestamp": "2025-12-20T08:00:00Z", "ip": "192.168.1.1", "device": "macOS"},
          {"timestamp": "2025-12-25T14:22:10Z", "ip": "10.0.0.45", "device": "iOS"}
        ]
      },
      {
        "id": "u_88220",
        "guid": "721f9120-f30c-52e5-b827-557766551111",
        "is_active": false,
        "balance": "$0.00",
        "picture": "http://placehold.it/32x32",
        "age": 29,
        "eye_color": "blue",
        "name": {
          "first": "Marcus",
          "last": "Chen"
        },
        "company": "NEXUS",
        "email": "marcus.chen@nexus.io",
        "phone": "+1 (985) 555-0432",
        "address": "456 Oak Avenue, Metropolis, IL 60601",
        "about": "Junior developer focusing on frontend accessibility and UX.",
        "registered": "2022-01-15T09:12:33 -08:00",
        "permissions": ["user"],
        "login_history": []
      }
    ],
    "system_logs": [
      {
        "id": "log_771",
        "level": "INFO",
        "message": "System heartbeat detected",
        "source": "server-node-04"
      },
      {
        "id": "log_772",
        "level": "WARN",
        "message": "High memory usage on cluster 5",
        "source": "monitor-daemon"
      },
      {
        "id": "log_773",
        "level": "ERROR",
        "message": "Database connection timeout",
        "source": "db-proxy-01",
        "stack_trace": "Error: Timeout at Socket.<anonymous> (node:net:321:12)"
      }
    ],
    "configuration_flags": {
      "feature_flags": {
        "enable_dark_mode": true,
        "beta_access": false,
        "max_upload_size_mb": 500
      },
      "regional_settings": [
        {"region": "US", "currency": "USD", "language": "en-US"},
        {"region": "EU", "currency": "EUR", "language": "de-DE"},
        {"region": "JP", "currency": "JPY", "language": "ja-JP"}
      ]
    }
  }'
  val = {
    "system_logs": [
        {"level": "INFO", "id": "log_771", "source": "server-node-04", "message": "Systemheartbeatdetected"},
        {"source": "monitor-daemon", "level": "WARN", "message": "Highmemoryusageoncluster5", "id": "log_772"},
        {"id": "log_773", "source": "db-proxy-01", "stack_trace": "Error:TimeoutatSocket.<anonymous>(node:net:321:12)", "message": "Databaseconnectiontimeout", "level": "ERROR"}
      ],
    "organization": {
      "name": "NexusDynamicsGlobal",
      "headquarters": {
        "state": "CA",
        "city": "SanFrancisco",
        "address": "101InnovationWay",
        "postal_code": "94105",
        "coordinates": {"lat": 37.7749, "lng": -122.4194}
      }, 
      "ticker": "NXDG", 
      "departments": [
        {"name": "Research&Development", "id": "dept_001", "budget": 4500000.5, "active_projects": [{"status": "in_progress", "tags": ["AI", "Quantum", "Bio-tech"], "milestones": [{"completed": vrai, "id": 1, "task": "InitialPrototyping"}, {"id": 2, "task": "BetaTestingPhaseI", "completed": faux}], "project_id": "PRJ-992", "title": "QuantumNeuralLink"}]},
        {"active_projects": [], "budget": 2100000.0, "name": "Infrastructure", "id": "dept_002"}
      ]
    }, 
    "configuration_flags": {
      "regional_settings": [
        {"region": "US", "currency": "USD", "language": "en-US"},
        {"currency": "EUR", "region": "EU", "language": "de-DE"},
        {"language": "ja-JP", "region": "JP", "currency": "JPY"}
      ], 
      "feature_flags": {"beta_access": faux, "enable_dark_mode": vrai, "max_upload_size_mb": 500}
    }, 
    "metadata": {"environment": "production-test", "checksum": "a7b8c9d0e1f2g3h4i5j6", "total_records": 1500, "version": "2.4.0", "generated_at": "2025-12-27T13:43:03Z"},
    "users": [
      {
        "picture": "http://placehold.it/32x32", 
        "registered": "2018-05-12T10:45:00-08:00",
        "age": 34,
        "phone": "+1(985)555-0192",
        "guid": "550e8400-e29b-41d4-a716-446655440000",
        "is_active": vrai,
        "name": {"first": "Elena", "last": "Rodriguez"}, 
        "email": "elena.rodriguez@nexus.io", 
        "address": "789MapleTerrace,Gotham,NY10001", 
        "about": "Expertindistributedsystemsandcloudarchitecturewithover10yearsofexperience.", 
        "permissions": ["admin", "editor", "billing"],
        "company": "NEXUS",
        "balance": "$3,450.12", 
        "id": "u_88219", 
        "eye_color": "green", 
        "login_history": [{"device": "macOS", "timestamp": "2025-12-20T08:00:00Z", "ip": "192.168.1.1"}, {"timestamp": "2025-12-25T14:22:10Z", "device": "iOS", "ip": "10.0.0.45"}]
      },
      {
        "guid": "721f9120-f30c-52e5-b827-557766551111",
        "registered": "2022-01-15T09:12:33-08:00",
        "email": "marcus.chen@nexus.io",
        "id": "u_88220",
        "age": 29,
        "name": {"first": "Marcus", "last": "Chen"}, 
        "login_history": [],
        "permissions": ["user"],
        "is_active": faux,
        "eye_color": "blue",
        "about": "JuniordeveloperfocusingonfrontendaccessibilityandUX.",
        "address": "456OakAvenue,Metropolis,IL60601", 
        "company": "NEXUS", 
        "picture": "http://placehold.it/32x32", 
        "phone": "+1(985)555-0432", 
        "balance": "$0.00"
      }
    ]
  }

  obj = JSONParser { txt: src }.parseValeur()
  
  Test.affirmerÉgaux(obj, val, "Même chose")
fin fonction


test()
