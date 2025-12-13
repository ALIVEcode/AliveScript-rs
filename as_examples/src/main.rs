use macroquad::{miniquad::window::screen_size, prelude::*};
use std::{
    fs,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use anyhow::{Result, anyhow};

use alivescript_rust::{
    as_fonction_native,
    as_obj::ASType,
    compiler::{Compiler, obj::Value, vm::VM},
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
        sin(x: ASType::nombre()): ASType::Nul => {
            let val = x.as_decimal().unwrap();

            Ok(Some(Value::Decimal(val.sin())))
        }
    };

    let attendre = as_fonction_native! {
        attendre(ms: ASType::Entier): ASType::Nul => {
            thread::sleep(Duration::from_millis(ms.as_entier().unwrap() as u64));
            Ok(None)
        }
    };

    let app = Arc::clone(&ctx);
    let set_background = as_fonction_native! {
        changerFond(couleur: ASType::Texte): ASType::Nul => {
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
            x: ASType::Decimal,
            y: ASType::Decimal,
            w: ASType::Decimal,
            h: ASType::Decimal,
            couleur: ASType::Texte
        ): ASType::Nul => {
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
        tailleÉcran(): ASType::Liste => {
            let (w, h) = app.read().unwrap().screen;
            Ok(Some(Value::liste(vec![
                Value::Decimal(w as f64),
                Value::Decimal(h as f64)
            ])))
        }
    };

    vm.insert_global(get_screen_size);
    vm.insert_global(draw_rect);
    vm.insert_global(attendre);
    vm.insert_global(sin_func);
    vm.insert_global(set_background);
}

fn run_in_thread(ctx: Arc<RwLock<AppState>>) {
    thread::spawn(move || {
        let script = fs::read_to_string("./screen-saver.as").unwrap();
        let closure = Compiler::new(&script).parse_and_compile().unwrap();

        let mut vm = VM::new();

        add_macroquad_func(&mut vm, ctx);

        let result = vm
            .run_shared_closure(closure)
            .map_err(|err| anyhow!("{}", err));
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
