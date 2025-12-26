# benchmark_equivalent.py

import time

# Configuration constant (used to control benchmark length)
# Set this to a value that takes a noticeable amount of time (e.g., 30 or 35)
N = 30

# --------------------------------------------------------------------------
# I. CLOSURE & UPVALUE STRESS TEST
# --------------------------------------------------------------------------

def make_adder(n):
    # Captures 'n' as an Upvalue (a free variable in Python closure scope)
    def inner_function(x):
        return x + n
    return inner_function

# Create a series of closures (100 unique objects).
adders_list = []
i = 0
while i < 100:
    # The 'make_adder(i)' call creates a new closure referencing a unique 'n' value
    adders_list.append(make_adder(i))
    i = i + 1

# Run the last adder to ensure the objects were created and are working
closure_result = adders_list[99](5) # Should equal 99 + 5 = 104

# --------------------------------------------------------------------------
# II. RECURSIVE FUNCTION & STACK STRESS TEST (Fibonacci)
# --------------------------------------------------------------------------

def fib(n):
    if n < 2:
        return n
    else:
        # Heavy recursion: stresses stack frames and function calls
        return fib(n - 1) + fib(n - 2)

# Run the core calculation
start_fib = time.perf_counter()
fib_result = fib(N)
end_fib = time.perf_counter()

# --------------------------------------------------------------------------
# III. HEAVY IMPERATIVE LOOP (Simple Math)
# --------------------------------------------------------------------------

j = 0
loop_accumulator = 0
limit = 100000 # A large loop to test simple dispatch speed

# Using a while loop to closely mimic the 'tant que' structure
while j < limit:
    # Simple arithmetic/local variable access stress
    loop_accumulator = loop_accumulator + (j * 2 - 1)
    j = j + 1


# --------------------------------------------------------------------------
# IV. FINAL OUTPUT & CHECK
# --------------------------------------------------------------------------

# Display Fibonacci result (equivalent of 'si fib_result == fib_result alors afficher fib_result fin si')
if fib_result == fib_result:
    print(f"Fibonacci Result (N={N}): {fib_result}")

# Display closure and loop results
print(f"Closure Result: {closure_result}")
print(f"Loop Accumulator Result: {loop_accumulator}")

# Optional: Print the execution time for the Fibonacci part (since it's the main computational stress)
print(f"Fibonacci Calculation Time: {end_fib - start_fib:.4f} seconds")
