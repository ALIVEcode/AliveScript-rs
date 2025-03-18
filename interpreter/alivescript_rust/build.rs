use lalrpop;

#[cfg(not(feature = "no-ast"))]
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::create_dir_all(format!("{}/stdlib", out_dir)).unwrap();
    for file in std::fs::read_dir("./stdlib/").unwrap() {
        let file = file.unwrap();
        let path = file.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        if name.ends_with(".as") {
            let out = format!("{}/stdlib/{}", out_dir, name);
            std::fs::copy(path, out).unwrap();
        }
    }
    lalrpop::Configuration::new().process_dir("./src/").unwrap();
}

#[cfg(feature = "no-ast")]
fn main() {
}
