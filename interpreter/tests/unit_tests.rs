mod common;

use common::run_test;

#[test]
fn entiers() {
    run_test("tests/test_entier.as", &vec![]);
}

#[test]
fn suite() {
    run_test("tests/test_suite.as", &vec![]);
}
