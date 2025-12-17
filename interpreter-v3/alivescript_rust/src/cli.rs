use clap::{Args, Parser, Subcommand};
use pest::Parser as _;
use std::path::PathBuf;

use crate::{AlivescriptParser, Rule, compiler::Compiler, runtime::vm::VM};

// --- Utility Functions for Unimplemented Features ---

// Helper function to handle the unimplemented REPL start.
fn start_repl() {
    // Possible implementation:
    // 1. Initialize the VM state (e.g., globals, standard library).
    // 2. Loop: read line from stdin, compile, execute, print result.
    todo!("Implement the AliveScript Read-Eval-Print Loop (REPL).");
}

// Helper function to handle the unimplemented file execution.
fn run_file(path: &PathBuf, debug_infos: Option<&DebugInfo>, run: bool) {
    let script = std::fs::read_to_string(path).unwrap();
    evaluate_string(&script, debug_infos, run);
}

// Helper function to handle the unimplemented string evaluation.
fn evaluate_string(code: &str, debug_infos: Option<&DebugInfo>, run: bool) {
    let result_stmts = AlivescriptParser::parse(Rule::script, &code);

    match result_stmts {
        Ok(stmts) => {
            if debug_infos.is_some_and(|di| di.show_tokens()) {
                println!("{:#?}", stmts);
            }

            let compiler = Compiler::new(&code);
            let closure = if debug_infos.is_some_and(|di| di.show_bytecode()) {
                compiler.compile_debug(stmts)
            } else {
                compiler.compile(stmts)
            };

            let closure = match closure {
                Ok(c) => c,
                Err(err) => panic!("{}", err),
            };

            if run {
                let mut vm = VM::new();
                let result = vm.run(closure).unwrap();
            }
            // println!("{:#?}", vm.stack);
            // println!("{:?}", result);
        }
        Err(err) => panic!(
            "ErreurSyntaxe: {}\n{:#?}",
            err.to_string(),
            err.parse_attempts()
        ),
    };
}

// Helper function to handle the complex debug output.
fn print_debug_info(path: &PathBuf, infos: &str) {
    // Possible implementation:
    // 1. Read file contents.
    // 2. Instantiate a Lexer/Scanner and print tokens if 't' is in infos.
    // 3. Instantiate a Compiler.
    // 4. Compile the source code.
    // 5. If 'b' is in infos, print simple bytecode instructions.
    // 6. If 'B' is in infos, print detailed bytecode including locals, constants, and upvalues.
    // 7. If 'a' is in infos, print all of the above.

    let path_str = path.display();
    println!("\n--- Debugging {} ---", path_str);
    println!("Requested debug flags: '{}'", infos);

    if infos.contains('t') || infos.contains('a') {
        println!("Token Stream Debug requested.");
        todo!("Implement tokenizing and printing the token stream.");
    }

    if infos.contains('b') || infos.contains('B') || infos.contains('a') {
        println!("Bytecode Debug requested.");
        todo!("Implement compilation and printing of bytecode based on flags ('b'/'B'/'a').");
    }
}

// --- Clap Struct Definitions ---

/// The AliveScript programming language command-line interface.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct AliveCli {
    #[command(subcommand)]
    command: Option<AliveCommands>,

    /// Evaluate a single string as AliveScript code.
    /// Example: alive -e "var x = 10; afficher x * 2"
    #[arg(short = 'e', long)]
    evaluate: Option<String>,

    /// Optional file path to run if no subcommand is used.
    /// This captures the 'alive <FILE>' case.
    #[arg(default_value = None)]
    file_path: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum AliveCommands {
    /// Prints debug info of the AliveScript code to the console AND runs the file.
    /// INFOS is a string with characters meaning:
    /// 'b': simple bytecode
    /// 'B': detailed bytecode (locals, constants, upvalues)
    /// 't': tokens
    /// 'a': all available debug info
    #[command(name = "-d")]
    DebugAndRun(DebugInfo),

    /// Prints debug info of the AliveScript code to the console BUT DOESN'T run the file.
    /// INFOS is a string with the same flags as -d.
    #[command(name = "-D")]
    DebugOnly(DebugInfo),
}

#[derive(Args, Debug)]
struct DebugInfo {
    /// Debug flags: 'b' (simple bytecode), 'B' (detailed bytecode), 't' (tokens), 'a' (all).
    infos: String,

    /// The file containing the AliveScript code to debug.
    file: PathBuf,
}

impl DebugInfo {
    fn show_bytecode(&self) -> bool {
        self.infos.contains(['b', 'B', 'a'])
    }

    fn show_tokens(&self) -> bool {
        self.infos.contains(['t', 'a'])
    }
}

// --- Main Execution Logic ---

pub fn run_cli() {
    let cli = AliveCli::parse();

    match cli.command {
        // Case: alive -d <INFOS> <FILE>
        Some(AliveCommands::DebugAndRun(args)) => {
            run_file(&args.file, Some(&args), true);
        }

        // Case: alive -D <INFOS> <FILE>
        Some(AliveCommands::DebugOnly(args)) => {
            run_file(&args.file, Some(&args), false);
        }

        // Case: alive -e <STR>
        None if cli.evaluate.is_some() => {
            let code = cli.evaluate.as_ref().unwrap();
            evaluate_string(code, None, true);
        }

        // Case: alive <FILE>
        None if cli.file_path.is_some() => {
            let path = cli.file_path.as_ref().unwrap();
            run_file(path, None, true);
        }

        // Case: alive (REPL)
        None => {
            start_repl();
        }
    }
}
