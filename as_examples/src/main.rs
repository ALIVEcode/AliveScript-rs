use raylib::prelude::*;
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use alivescript_rust::{
    as_module_fonction,
    compiler::{
        obj::Value,
        value::{ASModule, ArcModule, Type},
    },
    runtime::{module::LazyModule, vm::VM},
};

#[derive(Debug, Default)]
struct AppState {
    instructions: Vec<Instruction>,
    screen: (f32, f32),
    frame_time: f32,
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

fn get_couleur(couleur: &str) -> Color {
    use raylib::color::Color as C;
    match couleur {
        "bleu" => C::BLUE,
        "noir" => C::BLACK,
        "orange" => C::ORANGE,
        "vert" => C::GREEN,
        _ => C::WHITE,
    }
}

#[derive(Debug, Default)]
struct GraphiqueModule {
    ctx: Arc<RwLock<AppState>>,
}

impl LazyModule for GraphiqueModule {
    fn load(&self) -> ArcModule {
        let ctx = &self.ctx;

        ASModule::from_iter(
            "Graphique",
            [
                as_module_fonction! {
                    sin(x: Type::nombre()): Type::Nul => {
                        let val = x.as_decimal().unwrap();

                        Ok(Some(Value::Decimal(val.sin())))
                    }
                },
                as_module_fonction! {
                    {
                        let app = Arc::clone(&ctx);
                    }
                    tailleÉcran(): Type::Liste => {
                        let (w, h) = app.read().unwrap().screen;
                        Ok(Some(Value::liste(vec![
                            Value::Decimal(w as f64),
                            Value::Decimal(h as f64)
                        ])))
                    }
                },
                as_module_fonction! {
                    {
                        let app = Arc::clone(&ctx);
                    }
                    dessinerFond(couleur: Type::Texte): Type::Nul => {
                        let couleur = couleur.as_texte().unwrap();

                        let color = get_couleur(couleur);

                        app.write().unwrap().instructions.push(Instruction::SetBgColor(color));

                        Ok(None)
                    }
                },
                as_module_fonction! {
                        {
                            let ctx = Arc::clone(&ctx);
                        }
                        dessinerRect(
                            x: Type::Decimal,
                            y: Type::Decimal,
                            w: Type::Decimal,
                            h: Type::Decimal,
                            couleur: Type::Texte
                        ): Type::Nul => {
                        let couleur = couleur.as_texte().unwrap();

                        let color = get_couleur(couleur);

                        ctx.write().unwrap().instructions.push(Instruction::DrawRect{
                            x: x.as_decimal().unwrap() as f32,
                            y: y.as_decimal().unwrap() as f32,
                            w: w.as_decimal().unwrap() as f32,
                            h: h.as_decimal().unwrap() as f32,
                            color
                        });

                        Ok(None)
                    }
                },
            ],
        )
    }
}

fn init_script(file: &str, ctx: Arc<RwLock<AppState>>) -> (VM, ArcModule) {
    let mut vm = VM::new(file.to_string());

    vm.insert_module("Graphique", Arc::new(GraphiqueModule { ctx }));

    let result = vm.run_file_to_module(&file);
    let module = match result {
        Ok(module) => module,
        Err(err) => {
            println!("{}\n", err);
            panic!("--- STACK ---\n{}", vm.dump_stack());
        }
    };

    (vm, module)
}

fn main() {
    // 1. Initialize the window and the raylib handle
    let (mut rl, thread) = raylib::init()
        .size(800, 450)
        .resizable()
        .title("Démonstration d'AliveScript dans un jeu")
        .build();

    // 2. Set the target frames per second
    rl.set_target_fps(60);

    let file = "./screen-saver-v3.as";

    let ctx = Arc::new(RwLock::new(AppState::default()));
    let (mut vm, module) = init_script(file, Arc::clone(&ctx));

    if let Ok(Value::Function(init_func)) = module.read().unwrap().get_member("init") {
        vm.run_fn(vec![], &init_func).unwrap();
    }

    let update_func =
        if let Ok(Value::Function(update_func)) = module.read().unwrap().get_member("update") {
            Some(update_func)
        } else {
            None
        };
    let dessiner_func =
        if let Ok(Value::Function(dessiner_func)) = module.read().unwrap().get_member("dessiner") {
            Some(dessiner_func)
        } else {
            None
        };

    // 3. Main game loop
    while !rl.window_should_close() {
        ctx.write().unwrap().screen = (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
        if let Some(ref update_func) = update_func {
            vm.run_fn(vec![], update_func).unwrap();
        }

        if let Some(ref dessiner_func) = dessiner_func {
            vm.run_fn(vec![], dessiner_func).unwrap();
        }

        // Begin drawing context
        let mut d = rl.begin_drawing(&thread);
        d.draw_fps(0, 0);

        for inst in ctx.write().unwrap().instructions.drain(..) {
            match inst {
                Instruction::SetBgColor(color) => d.clear_background(color),
                Instruction::DrawRect { x, y, w, h, color } => {
                    d.draw_rectangle_v(Vector2::new(x, y), Vector2::new(w, h), color);
                }
            }
        }
    }
}
