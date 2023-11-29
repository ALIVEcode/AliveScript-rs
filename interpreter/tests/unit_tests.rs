mod common;

use common::run_test;

#[test]
fn entiers() {
    run_test("tests/test_data/test_entier.as", &vec![]);
}

#[test]
fn suite() {
    run_test("tests/test_suite.as", &vec![]);
}

#[test]
fn booleen() {
    run_test("tests/test_data/test_bool.as", &vec![]);
}

#[test]
fn test_classe1() {
    run_test("tests/test_classe/test1.as", &vec![]);
}
