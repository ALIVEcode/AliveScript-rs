utiliser Système
utiliser Test

(:
   SUITE DE BENCHMARK DE PERFORMANCE
   Cible : Vitesse d'exécution, gestion des Shapes et Closures
:)

# --- UTILITAIRE DE MESURE ---
var _debut : entier = 0
fonction marquer_debut() -> rien
    _debut = Système.temps()
fin fonction

fonction marquer_fin(nom_test : texte) -> rien
    var _fin : entier = Système.temps()
    afficher("RESULTAT [" + nom_test + "] : " + texte(_fin - _debut) + " ms")
fin fonction

# --- 1. BENCHMARK : ARITHMÉTIQUE BRUTE (BOUCLE RÉPÉTER) ---
# Mesure la vitesse du cycle de fetch/decode/execute de la VM
marquer_debut()
var n : entier = 0
répéter 1000000
    n = n + 1
fin répéter
marquer_fin("Boucle simple 1M itérations")

# --- 2. BENCHMARK : APPELS DE FONCTIONS (RÉCURSION) ---
# Mesure le coût de création des "Stack Frames" (cadres de pile)
fonction fibonacci(n : entier) -> entier
    si n <= 1 alors retourner n
    retourner fibonacci(n - 1) + fibonacci(n - 2)
fin fonction

marquer_debut()
var res_fib : entier = fibonacci(25)
marquer_fin("Fibonacci récursif (25)")

# --- 3. BENCHMARK : ALLOCATION ET SHAPES (STRUCTURES) ---
# Mesure la rapidité de création d'objets sur le Tas et l'accès aux champs
structure Point
    x : entier
    y : entier
fin structure

marquer_debut()
var points : liste<Point> = []
var i : entier = 0
répéter 100000
    var p : Point = Point { x: i, y: i * 2 }
    points.ajouter(p)
    i = i + 1
    si i % 1000 == 0 alors afficher i
fin répéter
marquer_fin("Allocation 100k structures")

# --- 4. BENCHMARK : ACCÈS DYNAMIQUE VS STATIQUE ---
# Mesure l'efficacité du cache des "Shapes"
marquer_debut()
var somme : entier = 0
pour chaque p dans points faire
    somme = somme + p.x
fin pour
marquer_fin("Accès aux champs 100k objets")

# --- 5. BENCHMARK : GÉNÉRATION DE CLOSURES ---
# Mesure le coût de capture des variables (Upvalues)
marquer_debut()
var closures : liste<fonction> = []
var k : entier = 0
répéter 10000
    var local_k : entier = k
    closures.ajouter(fonction() -> entier: local_k * 2)
    k = k + 1
fin répéter
marquer_fin("Création 10k closures (capture)")

# --- 6. BENCHMARK : LOGIQUE ET BRANCHEMENTS ---
# Mesure l'efficacité des sauts (Jumps) et de la prédiction
marquer_debut()
var vrais : entier = 0
répéter 100000
    si (10 > 5 et non (2 == 3)) ou (1 == 1) alors
        vrais = vrais + 1
    fin si
fin répéter
marquer_fin("Logique complexe 100k itérations")

afficher "--- FIN DES BENCHMARKS ---"
