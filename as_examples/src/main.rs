use macroquad::{miniquad::window::screen_size, prelude::*};
use std::{
    collections::HashMap,
    fs,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use anyhow::{Result, anyhow};

use alivescript_rust::{
    as_fonction_native,
    compiler::{
        Compiler,
        obj::Value,
        value::{ASModule, ArcModule, Type},
    },
    runtime::vm::VM,
};

#[derive(Debug, Default)]
struct AppState {
    instructions: Vec<Instruction>,
    screen: (f32, f32),
}

#[derive(Debug)]
enum Instruction {
    SetBgColor(Color),
    DrawRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Color,
    },
}

fn add_macroquad_func(vm: &mut VM, ctx: Arc<RwLock<AppState>>) {
    let sin_func = as_fonction_native! {
        sin(x: Type::nombre()): Type::Nul => {
            let val = x.as_decimal().unwrap();

            Ok(Some(Value::Decimal(val.sin())))
        }
    };

    let attendre = as_fonction_native! {
        attendre(ms: Type::Entier): Type::Nul => {
            thread::sleep(Duration::from_millis(ms.as_entier().unwrap() as u64));
            Ok(None)
        }
    };

    let app = Arc::clone(&ctx);
    let set_background = as_fonction_native! {
        changerFond(couleur: Type::Texte): Type::Nul => {
            let couleur = couleur.as_texte().unwrap();

            let color = match couleur {
                "bleu" => BLUE,
                "noir" => BLACK,
                "orange" => ORANGE,
                "vert" => GREEN,
                _ => WHITE,
            };

            app.write().unwrap().instructions.push(Instruction::SetBgColor(color));

            Ok(None)
        }
    };

    let app = Arc::clone(&ctx);
    let draw_rect = as_fonction_native! {
        dessinerRect(
            x: Type::Decimal,
            y: Type::Decimal,
            w: Type::Decimal,
            h: Type::Decimal,
            couleur: Type::Texte
        ): Type::Nul => {
            let couleur = couleur.as_texte().unwrap();

            let color = match couleur {
                "bleu" => BLUE,
                "noir" => BLACK,
                "orange" => ORANGE,
                "vert" => GREEN,
                _ => WHITE,
            };

            app.write().unwrap().instructions.push(Instruction::DrawRect{
                x: x.as_decimal().unwrap() as f32,
                y: y.as_decimal().unwrap() as f32,
                w: w.as_decimal().unwrap() as f32,
                h: h.as_decimal().unwrap() as f32,
                color
            });

            Ok(None)
        }
    };

    let app = Arc::clone(&ctx);
    let get_screen_size = as_fonction_native! {
        tailleÉcran(): Type::Liste => {
            let (w, h) = app.read().unwrap().screen;
            Ok(Some(Value::liste(vec![
                Value::Decimal(w as f64),
                Value::Decimal(h as f64)
            ])))
        }
    };

    let module = ArcModule::new(RwLock::new(ASModule::new(
        "Graphique",
        HashMap::from_iter([get_screen_size, draw_rect, attendre, set_background]),
    )));

    vm.insert_module("Graphique", module);
}

fn run_in_thread(ctx: Arc<RwLock<AppState>>) {
    thread::spawn(move || {
        let file = "./screen-saver.as".to_string();
        let script = fs::read_to_string(&file).unwrap();
        let closure = Compiler::new(&script, file.clone())
            .parse_and_compile()
            .unwrap();

        let mut vm = VM::new(file);

        add_macroquad_func(&mut vm, ctx);

        let result = vm.run(closure).map_err(|err| anyhow!("{}", err));
        if let Err(err) = result {
            println!("{}\n", err);
            println!("--- STACK ---\n{}", vm.dump_stack());
        }
    });
}

#[macroquad::main("test alivescript gui")]
async fn main() -> Result<()> {
    let instructions = Arc::new(RwLock::new(AppState::default()));

    run_in_thread(Arc::clone(&instructions));

    let mut current_color = BLACK;
    loop {
        clear_background(current_color);

        instructions.write().unwrap().screen = screen_size();

        while let Some(instruction) = instructions.write().unwrap().instructions.pop() {
            match instruction {
                Instruction::SetBgColor(color) => current_color = color,
                Instruction::DrawRect { x, y, w, h, color } => draw_rectangle(x, y, w, h, color),
            }
        }

        next_frame().await;
    }
}
