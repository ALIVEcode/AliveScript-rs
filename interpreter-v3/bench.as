# Configuration constant (used to control benchmark length)
# Set this to a value that takes a noticeable amount of time (e.g., 30 or 35)
var N = 30

(: 
--------------------------------------------------------------------------
I. CLOSURE & UPVALUE STRESS TEST
This section tests the overhead of creating and managing many upvalues.
-------------------------------------------------------------------------- 
:)

fonction make_adder(n)
    # Captures 'n' as an Upvalue from the enclosing scope
    retourner fonction(x)
        retourner x + n
    fin fonction
fin fonction

# Create a series of closures (100 unique objects).
#var adders_list = []
#var i = 0
#tant que i < 100
#    # The 'make_adder(i)' call creates a new closure referencing 'i'
#    adders_list = adders_list + [make_adder(i)]
#    i = i + 1
#fin tant que

# Run the last adder to ensure the objects were created and are working
#var closure_result = adders_list[99](5) # Should equal 99 + 5 = 104

(:
--------------------------------------------------------------------------
II. RECURSIVE FUNCTION & STACK STRESS TEST (Fibonacci)
This section tests function call overhead and stack frame management.
--------------------------------------------------------------------------
:)

fonction fib(n)
    si n < 2 alors
        retourner n
    sinon
        # Heavy recursion: stresses stack frames and function calls
        retourner fib(n - 1) + fib(n - 2)
    fin si
fin fonction

# Run the core calculation
var fib_result = fib(N)

(:
--------------------------------------------------------------------------
III. HEAVY IMPERATIVE LOOP (Simple Math)
This section tests the raw dispatch speed of simple instructions.
-------------------------------------------------------------------------- 
:)

var j = 0
var loop_accumulator = 0
var limit = 100000 # A large loop to test simple VM dispatch speed

tant que j < limit
    # Simple arithmetic/local variable access stress
    loop_accumulator = loop_accumulator + (j * 2 - 1)
    j = j + 1
fin tant que


(:
--------------------------------------------------------------------------
IV. FINAL OUTPUT & CHECK
Ensure the compiler/VM doesn't optimize away the calculations.
-------------------------------------------------------------------------- 
:)

# Display Fibonacci result
si fib_result == fib_result alors # Always true check
    afficher fib_result
sinon
    afficher "Erreur"
fin si

# Display closure and loop results
afficher loop_accumulator
