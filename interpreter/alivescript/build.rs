const VERSION: &'static str = "0.1.0";

fn main() {
    let Some(home) = std::env::home_dir() else {
        return;
    };

    let out_dir = std::env::var("ALIVESCRIPT_LIB").unwrap_or(format!(
        "{}/.local/share/alivescript{}",
        home.display(),
        VERSION
    ));

    std::fs::create_dir_all(format!("{}/lib", out_dir)).unwrap();

    for file in std::fs::read_dir("./stdlib").unwrap() {
        let file = file.unwrap();
        let path = file.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        if name.ends_with(".as") {
            let out = format!("{}/lib/{}", out_dir, name);
            std::fs::copy(path, out).unwrap();
        }
    }
}
