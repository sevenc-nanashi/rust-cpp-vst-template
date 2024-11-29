use clap::{Parser, Subcommand};
use colored::Colorize;
use notify::Watcher;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

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
    /// C++のヘッダーファイルを生成する。
    #[command(version, about, long_about = None)]
    GenerateHeader,

    /// プラグインをビルドする。
    #[command(version, about, long_about = None)]
    Build(BuildArgs),

    /// Windows用のインストーラーを生成する。
    #[command(version, about, long_about = None)]
    GenerateInstaller,

    /// ログを確認する。
    #[command(version, about, long_about = None)]
    WatchLog,
}

#[derive(Parser, Debug)]
struct BuildArgs {
    /// Releaseビルドを行うかどうか。
    #[clap(short, long)]
    release: bool,
    /// logs内にVST内のログを出力するかどうか。
    #[clap(short, long)]
    log: Option<bool>,
}

fn generate_header() {
    let main_crate = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let bindings = cbindgen::generate(&main_crate).unwrap();
    let destination_path = main_crate.join("src/rust.generated.hpp");
    bindings.write_to_file(&destination_path);

    green_log!("Finished", "generated to {:?}", destination_path,);
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
    .before_spawn(|command| {
        blue_log!("Running", "{:?}", command);

        Ok(())
    })
    .dir(main_crate)
    .run()
    .unwrap();
    duct::cmd!("cmake", "--build", &destination_path)
        .dir(main_crate)
        .before_spawn(|command| {
            blue_log!("Running", "{:?}", command);

            Ok(())
        })
        .full_env(envs)
        .run()
        .unwrap();

    let elapsed = current.elapsed();
    green_log!(
        "Finished",
        "built in {}.{:03}s",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
    green_log!("", "destination: {:?}", &destination_path);
    green_log!("", "plugin: {:?}", destination_path.join("bin"),);
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
        .before_spawn(|command| {
            blue_log!("Running", "{:?}", command);

            Ok(())
        })
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
        SubCommands::GenerateHeader => {
            generate_header();
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
