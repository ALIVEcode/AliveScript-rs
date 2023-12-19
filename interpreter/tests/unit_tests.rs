mod common;

use alivescript_rust::data::Data;
use common::run_test;

#[test]
fn entiers() {
    run_test("tests/data/t_entier.as", &vec![]);
}

#[test]
fn suite() {
    run_test("tests/data/t_suite.as", &vec![]);
}

#[test]
fn booleen() {
    run_test("tests/data/t_bool.as", &vec![]);
}

#[test]
fn assign() {
    run_test("tests/stmt/t_assign.as", &vec![]);
}

#[test]
fn test_classe1() {
    run_test("tests/test1.as", &vec![]);
}

#[test]
fn test_print_recursive() {
    run_test(
        "tests/t_print_recursive.as",
        &vec![
            Data::Afficher("<1>@[1, [<1>], 3]".into()),
            Data::Afficher(r#"<1>@{"a": 1, "b": 2, "c": {<1>}}"#.into()),
        ],
    )
}
