utiliser Test {affirmerEgal, affirmerPasEgal}

affirmerEgal(1..10, [1,2,3,4,5,6,7,8,9])
affirmerEgal(1..2, [1])
affirmerEgal(1..1, [])
affirmerEgal(2..1, [])
affirmerEgal(-1..1, [-1,0])


affirmerEgal(-1..=1, [-1,0,1])
affirmerEgal(1..=1, [1])
affirmerEgal(1..=2, [1,2])
affirmerPasEgal(1..=2, [1,2,3])



affirmerEgal(1..10 bond 2, [1,3,5,7,9])
affirmerEgal(1..2 bond 2, [1])
affirmerEgal(10..3 bond -1, [10,9,8,7,6,5,4])
affirmerEgal(10..=3 bond -1, [10,9,8,7,6,5,4,3])

