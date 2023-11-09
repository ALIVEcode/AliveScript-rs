#![allow(unused_variables, dead_code)]

use std::env;

mod client_rpc;

use alivescript_rust::run_script;
use client_rpc::ClientRPC;

fn main() {
    let script = env::args().nth(1).expect("A script to execute") + "\n";

    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout();
    let mut interpretor_io = ClientRPC::new(&mut stdin, &mut stdout);
    run_script(script, &mut interpretor_io);
}
