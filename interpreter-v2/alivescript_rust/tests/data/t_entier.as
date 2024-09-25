utiliser Test {affirmer, affirmerEgal, affirmerEstInstance}

affirmerEgal(1, 1.0)
affirmerEgal(0, -0)
affirmerEgal(0, -0.0)

L = [
        ('0', 0),
        ('1', 1),
        ('9', 9),
        ('10', 10),
        ('99', 99),
        ('100', 100),
        ('314', 314),
        (' 314', 314),
        ('314 ', 314),
        ('  \t\t  314  \t\t  ', 314),
        ('  1x', "erreur"),
        ('  1  ', 1),
        ('  1\02  ', "erreur"),
        ('', "erreur"),
        (' ', "erreur"),
        ('  \t\t  ', "erreur"),
        ("\u0200", "erreur")
]

affirmerEgal(entier(314), 314)
affirmerEgal(entier(3.14), 3)
# Check that conversion from float truncates towards zero
affirmerEgal(entier(-3.14), -3)
affirmerEgal(entier(3.9), 3)
affirmerEgal(entier(-3.9), -3)
affirmerEgal(entier(3.5), 3)
affirmerEgal(entier(-3.5), -3)
affirmerEgal(entier("-3"), -3)
affirmerEgal(entier(" -3 "), -3)
affirmerEgal(entier("\N{EM SPACE}-3\N{EN SPACE}"), -3)
# Different base:
affirmerEgal(entier("10",16), 16)
# Test conversion from strings and various anomalies
pour s, v dans L faire
    pour sign dans "", "+", "-" faire
        pour prefix dans "", " ", "\t", "  \t\t  " faire
            ss = prefix + sign + s
            vv = v
            si sign == "-" et v != "erreur":
                vv = -v
            fin si
            result, err = essayer affirmerEgal(entier(ss), vv)
            si err != nul alors
            fin si
        fin pour
    fin pour
fin pour

assertIsInstance(x, entier)

# should return entier
x = entier(1e100)
assertIsInstance(x, entier)
x = entier(-1e100)
assertIsInstance(x, entier)

x = entier('1' * 600)
assertIsInstance(x, entier)

assertRaises(TypeError, entier, 1, 12)

affirmerEgal(entier('0o123', 0), 83)
affirmerEgal(entier('0x123', 16), 291)

# Bug 1679: "0x" is not a valid hex literal
assertRaises(ValueError, entier, "0x", 16)
assertRaises(ValueError, entier, "0x", 0)

assertRaises(ValueError, entier, "0o", 8)
assertRaises(ValueError, entier, "0o", 0)

assertRaises(ValueError, entier, "0b", 2)
assertRaises(ValueError, entier, "0b", 0)

# SF bug 1334662: entier(string, base) wrong answers
# Various representations of 2**32 evaluated to 0
# rather than 2**32 in previous versions

affirmerEgal(entier('100000000000000000000000000000000', 2), 4294967296)
affirmerEgal(entier('102002022201221111211', 3), 4294967296)
affirmerEgal(entier('10000000000000000', 4), 4294967296)
affirmerEgal(entier('32244002423141', 5), 4294967296)
affirmerEgal(entier('1550104015504', 6), 4294967296)
affirmerEgal(entier('211301422354', 7), 4294967296)
affirmerEgal(entier('40000000000', 8), 4294967296)
affirmerEgal(entier('12068657454', 9), 4294967296)
affirmerEgal(entier('4294967296', 10), 4294967296)
affirmerEgal(entier('1904440554', 11), 4294967296)
affirmerEgal(entier('9ba461594', 12), 4294967296)
affirmerEgal(entier('535a79889', 13), 4294967296)
affirmerEgal(entier('2ca5b7464', 14), 4294967296)
affirmerEgal(entier('1a20dcd81', 15), 4294967296)
affirmerEgal(entier('100000000', 16), 4294967296)
affirmerEgal(entier('a7ffda91', 17), 4294967296)
affirmerEgal(entier('704he7g4', 18), 4294967296)
affirmerEgal(entier('4f5aff66', 19), 4294967296)
affirmerEgal(entier('3723ai4g', 20), 4294967296)
affirmerEgal(entier('281d55i4', 21), 4294967296)
affirmerEgal(entier('1fj8b184', 22), 4294967296)
affirmerEgal(entier('1606k7ic', 23), 4294967296)
affirmerEgal(entier('mb994ag', 24), 4294967296)
affirmerEgal(entier('hek2mgl', 25), 4294967296)
affirmerEgal(entier('dnchbnm', 26), 4294967296)
affirmerEgal(entier('b28jpdm', 27), 4294967296)
affirmerEgal(entier('8pfgih4', 28), 4294967296)
affirmerEgal(entier('76beigg', 29), 4294967296)
affirmerEgal(entier('5qmcpqg', 30), 4294967296)
affirmerEgal(entier('4q0jto4', 31), 4294967296)
affirmerEgal(entier('4000000', 32), 4294967296)
affirmerEgal(entier('3aokq94', 33), 4294967296)
affirmerEgal(entier('2qhxjli', 34), 4294967296)
affirmerEgal(entier('2br45qb', 35), 4294967296)
affirmerEgal(entier('1z141z4', 36), 4294967296)

# tests with base 0
# this fails on 3.0, but in 2.x the old octal syntax is allowed
affirmerEgal(entier(' 0o123  ', 0), 83)
affirmerEgal(entier(' 0o123  ', 0), 83)
affirmerEgal(entier('000', 0), 0)
affirmerEgal(entier('0o123', 0), 83)
affirmerEgal(entier('0x123', 0), 291)
affirmerEgal(entier('0b100', 0), 4)
affirmerEgal(entier(' 0O123   ', 0), 83)
affirmerEgal(entier(' 0X123  ', 0), 291)
affirmerEgal(entier(' 0B100 ', 0), 4)
with assertRaises(ValueError):
    entier('010', 0)

# without base still base 10
affirmerEgal(entier('0123'), 123)
affirmerEgal(entier('0123', 10), 123)

# tests with prefix and base != 0
affirmerEgal(entier('0x123', 16), 291)
affirmerEgal(entier('0o123', 8), 83)
affirmerEgal(entier('0b100', 2), 4)
affirmerEgal(entier('0X123', 16), 291)
affirmerEgal(entier('0O123', 8), 83)
affirmerEgal(entier('0B100', 2), 4)

# the code has special checks for the first character after the
#  type prefix
assertRaises(ValueError, entier, '0b2', 2)
assertRaises(ValueError, entier, '0b02', 2)
assertRaises(ValueError, entier, '0B2', 2)
assertRaises(ValueError, entier, '0B02', 2)
assertRaises(ValueError, entier, '0o8', 8)
assertRaises(ValueError, entier, '0o08', 8)
assertRaises(ValueError, entier, '0O8', 8)
assertRaises(ValueError, entier, '0O08', 8)
assertRaises(ValueError, entier, '0xg', 16)
assertRaises(ValueError, entier, '0x0g', 16)
assertRaises(ValueError, entier, '0Xg', 16)
assertRaises(ValueError, entier, '0X0g', 16)

# SF bug 1334662: entier(string, base) wrong answers
# Checks for proper evaluation of 2**32 + 1
affirmerEgal(entier('100000000000000000000000000000001', 2), 4294967297)
affirmerEgal(entier('102002022201221111212', 3), 4294967297)
affirmerEgal(entier('10000000000000001', 4), 4294967297)
affirmerEgal(entier('32244002423142', 5), 4294967297)
affirmerEgal(entier('1550104015505', 6), 4294967297)
affirmerEgal(entier('211301422355', 7), 4294967297)
affirmerEgal(entier('40000000001', 8), 4294967297)
affirmerEgal(entier('12068657455', 9), 4294967297)
affirmerEgal(entier('4294967297', 10), 4294967297)
affirmerEgal(entier('1904440555', 11), 4294967297)
affirmerEgal(entier('9ba461595', 12), 4294967297)
affirmerEgal(entier('535a7988a', 13), 4294967297)
affirmerEgal(entier('2ca5b7465', 14), 4294967297)
affirmerEgal(entier('1a20dcd82', 15), 4294967297)
affirmerEgal(entier('100000001', 16), 4294967297)
affirmerEgal(entier('a7ffda92', 17), 4294967297)
affirmerEgal(entier('704he7g5', 18), 4294967297)
affirmerEgal(entier('4f5aff67', 19), 4294967297)
affirmerEgal(entier('3723ai4h', 20), 4294967297)
affirmerEgal(entier('281d55i5', 21), 4294967297)
affirmerEgal(entier('1fj8b185', 22), 4294967297)
affirmerEgal(entier('1606k7id', 23), 4294967297)
affirmerEgal(entier('mb994ah', 24), 4294967297)
affirmerEgal(entier('hek2mgm', 25), 4294967297)
affirmerEgal(entier('dnchbnn', 26), 4294967297)
affirmerEgal(entier('b28jpdn', 27), 4294967297)
affirmerEgal(entier('8pfgih5', 28), 4294967297)
affirmerEgal(entier('76beigh', 29), 4294967297)
affirmerEgal(entier('5qmcpqh', 30), 4294967297)
affirmerEgal(entier('4q0jto5', 31), 4294967297)
affirmerEgal(entier('4000001', 32), 4294967297)
affirmerEgal(entier('3aokq95', 33), 4294967297)
affirmerEgal(entier('2qhxjlj', 34), 4294967297)
affirmerEgal(entier('2br45qc', 35), 4294967297)
affirmerEgal(entier('1z141z5', 36), 4294967297)
