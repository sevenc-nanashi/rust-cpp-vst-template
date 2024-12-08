use clap::{Parser, Subcommand};
use colored::Colorize;
use notify::Watcher;
use std::io::Write;

macro_rules! green_log {
    ($subject:expr, $($args:tt)+) => {
        println!("{:>12} {}", $subject.bold().green(), &format!($($args)*));
    };
}
macro_rules! blue_log {
    ($subject:expr, $($args:tt)+) => {
        println!("{:>12} {}", $subject.bold().cyan(), &format!($($args)*));
    };
}
macro_rules! red_log {
    ($subject:expr, $($args:tt)+) => {
        println!("{:>12} {}", $subject.bold().red(), &format!($($args)*));
    };
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    /// Generate bridge files.
    #[command(version, about, long_about = None)]
    GenerateBridge,

    /// Build the project.
    #[command(version, about, long_about = None)]
    Build(BuildArgs),

    /// Build Installer for Windows.
    #[command(version, about, long_about = None)]
    GenerateInstaller,

    /// Watch logs.
    #[command(version, about, long_about = None)]
    WatchLog,
}

#[derive(Parser, Debug)]
struct BuildArgs {
    /// Whether to build in release mode.
    #[clap(short, long)]
    release: bool,
    /// Whether to enable logging.
    #[clap(short, long)]
    log: Option<bool>,
}

fn print_cmd(process: &std::process::Command) -> std::io::Result<()> {
    blue_log!("Running", "{:?}", process);
    Ok(())
}

fn generate_bridge() {
    blue_log!("Running", "cbindgen");
    let main_crate = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let bindings = cbindgen::generate(&main_crate).unwrap();
    let mut cbindgen_binding = vec![];
    bindings.write(&mut cbindgen_binding);

    blue_log!("Generating", "rust_bridge.generated.hpp");
    let contents = std::str::from_utf8(&cbindgen_binding).unwrap();
    let re = lazy_regex::regex!(
        r#"EXPORT\s+(?<returns>[\w ]+\s+\*?)(?<name>\w+)\s*\((?<args>[^)]*)\);"#
    );
    let mut functions = vec![];
    for cap in re.captures_iter(&contents) {
        let returns = cap.name("returns").unwrap().as_str();
        let name = cap.name("name").unwrap().as_str();
        let args = cap.name("args").unwrap().as_str();
        let args = args
            .split(',')
            .map(|arg| {
                let arg = arg.trim();
                arg.split_whitespace()
                    .filter(|arg| !arg.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>();
        functions.push((returns, name, args));
    }

    let types = contents
        .lines()
        .skip_while(|line| !line.contains("namespace Rust {"))
        .skip(1)
        .take_while(|line| !line.contains("extern \"C\" {"))
        .collect::<Vec<_>>();
    let types = types.join("\n");
    assert!(!types.is_empty());

    let bridge_header_path = main_crate.join("src/rust_bridge.generated.hpp");
    let mut file = std::fs::File::create(&bridge_header_path).unwrap();
    writeln!(file, "#pragma once").unwrap();
    writeln!(file, "#include <choc/platform/choc_DynamicLibrary.h>").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "namespace Rust {{").unwrap();
    writeln!(file, "{}", types).unwrap();
    writeln!(file, "    choc::file::DynamicLibrary* loadRustDll();").unwrap();
    for (returns, name, args) in &functions {
        let args = args.join(", ");
        writeln!(file, "    {} {}({});", returns, name, args).unwrap();
        writeln!(file).unwrap();
    }
    writeln!(file, "}}").unwrap();

    blue_log!("Generating", "rust_bridge.generated.cpp");
    let bridge_path = main_crate.join("src/rust_bridge.generated.cpp");
    let mut file = std::fs::File::create(&bridge_path).unwrap();
    writeln!(file, "#include \"rust_bridge.generated.hpp\"").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "namespace Rust {{").unwrap();
    for (returns, name, args) in &functions {
        let args = args.join(", ");
        writeln!(file, "    typedef {} (*{}_t)({});", returns, name, args).unwrap();
        writeln!(file, "    {} {}({}) {{", returns, name, args).unwrap();
        writeln!(file, "        auto rust = Rust::loadRustDll();").unwrap();
        writeln!(
            file,
            "        auto fn = ({}_t)rust->findFunction(\"{}\");",
            name, name
        )
        .unwrap();

        let args_regex = lazy_regex::regex!(r"(?P<name>\w+)(?:,|$)");
        let mut arg_names = vec![];
        for cap in args_regex.captures_iter(&args) {
            let name = cap.name("name").unwrap().as_str();
            arg_names.push(name);
        }
        let arg_names = arg_names.join(", ");

        if *returns != "void" {
            writeln!(file, "        return fn({});", arg_names).unwrap();
        } else {
            writeln!(file, "        fn({});", arg_names).unwrap();
        }
        writeln!(file, "    }}").unwrap();
        writeln!(file).unwrap();
    }
    writeln!(file, "}}").unwrap();

    duct::cmd!("clang-format", "-i", &bridge_header_path)
        .before_spawn(|command| print_cmd(command))
        .run()
        .unwrap();
    duct::cmd!("clang-format", "-i", &bridge_path)
        .before_spawn(|command| print_cmd(command))
        .run()
        .unwrap();

    green_log!("Finished", "generated to:");
    green_log!("", "- {:?}", bridge_header_path);
    green_log!("", "- {:?}", bridge_path);
}
fn build(args: BuildArgs) {
    let main_crate = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();

    let enable_log = args.log.unwrap_or(!args.release);
    if args.release {
        if enable_log {
            panic!("Cannot enable logging in release mode");
        }
    }
    let mut envs = std::env::vars().collect::<std::collections::HashMap<_, _>>();
    if enable_log {
        envs.insert("RUST_VST_LOG".to_string(), "1".to_string());
    }

    if colored::control::SHOULD_COLORIZE.should_colorize() {
        envs.insert("CLICOLOR_FORCE".to_string(), "1".to_string());
    }

    let build_name = if args.release { "release" } else { "debug" };
    green_log!("Building", "log: {}, release: {}", enable_log, args.release);

    let destination_path = main_crate.join("build").join(build_name);

    let current = std::time::Instant::now();

    let build_type = format!(
        "-DCMAKE_BUILD_TYPE={}",
        if args.release { "Release" } else { "Debug" }
    );
    let build_dir = format!("-B{}", &destination_path.to_string_lossy());
    // _add_library causes infinite recursion somehow, so disable vcpkg toolchain file
    // https://github.com/microsoft/vcpkg/issues/11307
    if cfg!(windows) {
        duct::cmd!(
            "cmake",
            "-DCMAKE_TOOLCHAIN_FILE=OFF",
            &build_type,
            &build_dir
        )
    } else {
        duct::cmd!("cmake", &build_type, &build_dir)
    }
    .before_spawn(|command| print_cmd(command))
    .dir(main_crate)
    .run()
    .unwrap();
    duct::cmd!("cmake", "--build", &destination_path)
        .dir(main_crate)
        .before_spawn(|command| print_cmd(command))
        .full_env(envs)
        .run()
        .unwrap();

    // TODO: Do this in cmake as cmake knows more about the build
    blue_log!("Copying", "plugin dll to bin");
    let plugin_name = if cfg!(target_os = "windows") {
        "my_plugin_impl.dll"
    } else if cfg!(target_os = "macos") {
        "libmy_plugin_impl.dylib"
    } else if cfg!(target_os = "linux") {
        "libmy_plugin_impl.so"
    } else {
        panic!("Unsupported platform");
    };
    let plugin_path = destination_path.join(plugin_name);
    let vst_root = destination_path.join("bin");
    let vst_path = glob::glob(&format!("{}/*/**/*.vst3", vst_root.to_string_lossy()))
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
    let vst_path = vst_path.parent().unwrap();
    std::fs::copy(&plugin_path, vst_path.join(plugin_name)).unwrap();

    let elapsed = current.elapsed();
    green_log!(
        "Finished",
        "built in {}.{:03}s",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
    green_log!("", "destination: {:?}", &destination_path);
    green_log!("", "plugin: {:?}", destination_path.join("bin"));
    if enable_log {
        green_log!("", "logs: {:?}", main_crate.join("logs"));
    }
}

fn generate_installer() {
    let current = std::time::Instant::now();

    let main_crate = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let main_cargo_toml = main_crate.join("Cargo.toml");
    let main_cargo_toml = cargo_toml::Manifest::from_path(&main_cargo_toml).unwrap();

    let version: String = main_cargo_toml.package.unwrap().version.unwrap();

    let installer_base = main_crate
        .join("resources")
        .join("installer")
        .join("installer_base.nsi");
    let installer_dist = main_crate.join("installer.nsi");

    let installer_base = std::fs::read_to_string(&installer_base).unwrap();
    std::fs::write(
        &installer_dist,
        installer_base.replace("{version}", &version),
    )
    .unwrap();
    blue_log!("Building", "wrote nsis script to {:?}", installer_dist);

    duct::cmd!("makensis", &installer_dist, "/INPUTCHARSET", "UTF8")
        .dir(main_crate)
        .before_spawn(|command| print_cmd(command))
        .run()
        .unwrap();
    green_log!(
        "Finished",
        "built to {:?} in {}.{:03}s",
        installer_dist.with_extension("exe"),
        current.elapsed().as_secs(),
        current.elapsed().subsec_millis()
    );
}

fn watch_log() {
    let main_crate = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let logs = main_crate.join("logs");
    if !logs.exists() {
        panic!("Logs not found at {:?}", logs);
    }

    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(tx).unwrap();
    watcher
        .watch(&logs, notify::RecursiveMode::Recursive)
        .unwrap();
    let mut current_log = find_log(&logs);
    let mut current_log_process: Option<duct::Handle> = None;

    if let Some(ref current_log) = current_log {
        green_log!("Watching", "current log: {:?}", current_log);
        current_log_process = Some(duct::cmd!("tail", "-f", current_log).start().unwrap());
    } else {
        green_log!("Watching", "no log found");
    }

    for event in rx {
        let event = event.unwrap();
        match event.kind {
            notify::EventKind::Create(_) | notify::EventKind::Remove(_) => {
                let new_log = find_log(&logs);
                if new_log != current_log {
                    if let Some(ref mut current_log_process) = current_log_process {
                        current_log_process.kill().unwrap();
                    }
                    if let Some(ref new_log) = new_log {
                        green_log!("Watching", "new log: {:?}", new_log);
                        current_log_process =
                            Some(duct::cmd!("tail", "-f", new_log).start().unwrap());
                    } else {
                        green_log!("Watching", "no log found");
                    }
                    current_log = new_log;
                }

                if let Some(ref current_log) = current_log {
                    let panic_path = current_log.with_extension("panic");
                    if panic_path.exists() {
                        let panic = std::fs::read_to_string(&panic_path).unwrap();
                        red_log!("Panicked", "{}", panic);
                    }
                }
            }
            _ => {
                continue;
            }
        }
    }

    fn find_log(logs_dir: &std::path::Path) -> Option<std::path::PathBuf> {
        let mut current_logs = std::fs::read_dir(&logs_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.is_file()
                    && path.extension().unwrap_or_default() == "log"
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.split('.').next().unwrap().parse::<u64>().is_ok())
            })
            .collect::<Vec<_>>();
        current_logs.sort_by_key(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap()
                .split('.')
                .next()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        });

        current_logs.last().cloned()
    }
}

fn main() {
    let args = Args::parse();

    match args.subcommand {
        SubCommands::GenerateBridge => {
            generate_bridge();
        }
        SubCommands::Build(build_args) => {
            build(build_args);
        }
        SubCommands::GenerateInstaller => {
            generate_installer();
        }
        SubCommands::WatchLog => {
            watch_log();
        }
    }
}
