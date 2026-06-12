use clap::{CommandFactory, Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;
use vole_runtime::Vm;

#[derive(Parser)]
#[command(name = "vole")]
#[command(about = "A vole ECMAScript runtime built with Rust and QuickJS", long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
#[command(disable_help_subcommand = true)]
struct Cli {
    /// File to execute
    file: Option<PathBuf>,

    /// Watch mode for file changes
    #[arg(short, long)]
    watch: bool,

    /// Print version information
    #[arg(short = 'v', long)]
    version: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a JavaScript file
    Run {
        /// File to execute
        file: PathBuf,
    },
    /// Evaluate JavaScript code directly
    Eval {
        /// JavaScript code to evaluate
        code: String,
    },
    /// Start an interactive REPL
    Repl,
    /// Print version information including dependencies
    Version,
    /// Run tests
    Test {
        /// File or directory to test
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Initialize a new project
    Init {
        /// Project name
        #[arg(default_value = "my-project")]
        name: String,
        /// Project directory
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
    /// Install dependencies
    Install {
        /// Packages to install (if empty, installs all from package.json)
        packages: Vec<String>,
    },
    /// Print help information
    Help {
        /// Command to get help for
        command: Option<String>,
    },
}

static CARGO_LOCK: &str = include_str!("../../../Cargo.lock");

const RELEVANT_DEPENDENCIES: &[&str] = &[
    "rquickjs",
    "rquickjs-serde",
    "tokio",
    "clap",
    "reqwest",
    "axum",
    "serde",
    "oxc_parser",
    "wasmi",
    "libffi",
    "libloading",
    "rsa",
    "aes",
    "sha1",
    "sha2",
    "notify",
    "tar",
    "flate2",
    "semver",
    "url",
    "base64",
];

fn print_dependency_versions() {
    let lock: toml::Value = CARGO_LOCK.parse().expect("Failed to parse Cargo.lock");
    let packages = lock
        .get("package")
        .and_then(|p| p.as_array())
        .expect("Invalid Cargo.lock");

    let mut versions: Vec<(&str, &str)> = Vec::new();
    for pkg in packages {
        let name = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let version = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("");
        if RELEVANT_DEPENDENCIES.contains(&name) {
            versions.push((name, version));
        }
    }

    // Preserve the order defined in RELEVANT_DEPENDENCIES.
    for &name in RELEVANT_DEPENDENCIES {
        if let Some(version) = versions.iter().find(|(n, _)| *n == name).map(|(_, v)| *v) {
            println!("{} {}", name, version);
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("vole {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    match cli.command {
        Some(Commands::Run { file }) => {
            if cli.watch {
                watch_and_run(&file).await;
                return;
            }
            let vm = Vm::new().await.expect("Failed to create VM");
            execute_file(&vm, &file).await;
        }
        Some(Commands::Eval { code }) => {
            let vm = Vm::new().await.expect("Failed to create VM");
            evaluate_code(&vm, &code).await;
        }
        Some(Commands::Repl) => {
            let vm = Vm::new().await.expect("Failed to create VM");
            start_repl(&vm).await;
        }
        Some(Commands::Version) => {
            println!("vole {}", env!("CARGO_PKG_VERSION"));
            print_dependency_versions();
        }
        Some(Commands::Test { path }) => {
            let files = find_test_files(&path);
            if files.is_empty() {
                println!("No test files found");
                return;
            }
            let vm = Vm::new().await.expect("Failed to create VM");
            match vm.run_tests(&files).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Init { name, dir }) => {
            let pm = vole_package::PackageManager::new();
            if let Err(e) = pm.init_project(&name, &dir) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Install { packages }) => {
            let pm = vole_package::PackageManager::new();
            let dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let result = if packages.is_empty() {
                pm.install_all(&dir).await
            } else {
                let refs: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
                pm.install_packages(&refs, &dir).await
            };
            if let Err(e) = result {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Help { command }) => {
            let mut cmd = Cli::command();
            match command {
                Some(name) => {
                    if let Some(mut sub) = cmd.find_subcommand(&name).cloned() {
                        sub.print_help().unwrap();
                        println!();
                    } else {
                        eprintln!("Error: unknown command '{}'", name);
                        std::process::exit(1);
                    }
                }
                None => {
                    cmd.print_help().unwrap();
                    println!();
                }
            }
        }
        None => {
            let file = cli.file.unwrap_or_else(|| {
                eprintln!("Error: no file specified");
                std::process::exit(1);
            });

            if cli.watch {
                watch_and_run(&file).await;
                return;
            }

            let vm = Vm::new().await.expect("Failed to create VM");
            execute_file(&vm, &file).await;
        }
    }
}

async fn execute_file(vm: &Vm, file: &PathBuf) {
    match vm.eval_file(file).await {
        Ok(result) => {
            if result != "undefined" {
                println!("{}", result);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn evaluate_code(vm: &Vm, code: &str) {
    match vm.eval(code).await {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn start_repl(vm: &Vm) {
    println!("vole v{} REPL", env!("CARGO_PKG_VERSION"));
    println!("Type 'exit' or 'quit' to exit\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            println!("Goodbye!");
            break;
        }

        match vm.eval(input).await {
            Ok(result) => {
                if result != "undefined" {
                    println!("{}", result);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

async fn watch_and_run(file: &PathBuf) {
    use notify::{Event, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        let _ = tx.send(res);
    })
    .expect("Failed to create file watcher");

    let watch_dir = file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .expect("Failed to watch file");

    loop {
        let vm = Vm::new().await.expect("Failed to create VM");
        execute_file(&vm, file).await;
        drop(vm);

        println!("\n[watch] Waiting for changes... (press Ctrl+C to stop)\n");

        loop {
            match rx.try_recv() {
                Ok(Ok(Event { kind, .. })) => {
                    if kind.is_access() {
                        continue;
                    }
                    break;
                }
                Ok(Err(e)) => {
                    eprintln!("[watch] error: {}", e);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    continue;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    eprintln!("[watch] watcher disconnected");
                    return;
                }
            }
        }
    }
}

fn find_test_files(path: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_test_files(path, &mut files);
    files
}

fn collect_test_files(path: &std::path::Path, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path
            .file_name()
            .map(|n| {
                let name = n.to_string_lossy();
                name.ends_with("_test.js") || name.ends_with("_test.ts")
            })
            .unwrap_or(false)
        {
            files.push(path.to_path_buf());
        }
        return;
    }
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default();
                if name == "target"
                    || name == "node_modules"
                    || name == ".git"
                    || name.starts_with('.')
                {
                    continue;
                }
                collect_test_files(&path, files);
            } else if path.is_file() {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default();
                if name.ends_with("_test.js") || name.ends_with("_test.ts") {
                    files.push(path);
                }
            }
        }
    }
}
