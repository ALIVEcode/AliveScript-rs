mod common;

use common::TestIO;

use alivescript_rust::run_script;

#[test]
fn numbers() {
    let mut test_io = TestIO::default();
    run_script(String::from("1za"), &mut test_io);
    dbg!("{:#?}", test_io.outputs());
}
