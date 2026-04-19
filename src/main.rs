use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use unicode_normalization::UnicodeNormalization;
use walkdir::WalkDir;

/// Rename files from NFC (Windows default) to NFD (Apple default) so the iOS
/// system document picker can select CJK-named files. Only files whose NFD form
/// actually differs from NFC are touched; everything else is left alone.
#[derive(Parser, Debug)]
#[command(name = "nfc2nfd", version, about, long_about = None)]
struct Cli {
    /// Directory to process.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Recurse into subdirectories.
    #[arg(short, long)]
    recursive: bool,

    /// Print planned renames without touching the filesystem.
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Also rename directory names (default: files only).
    #[arg(short = 'D', long = "include-dirs")]
    include_dirs: bool,

    /// Print entries that are already NFD as well.
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Default)]
struct Stats {
    renamed: usize,
    already_nfd: usize,
    collisions: usize,
    errors: usize,
    skipped_non_utf8: usize,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if !cli.path.exists() {
        eprintln!("error: path not found: {}", cli.path.display());
        return ExitCode::from(2);
    }
    if !cli.path.is_dir() {
        eprintln!("error: not a directory: {}", cli.path.display());
        return ExitCode::from(2);
    }

    let mut stats = Stats::default();

    // `contents_first(true)` yields directory contents before the directory
    // itself, so when --include-dirs is on we can safely rename a parent after
    // its children without invalidating child paths.
    let walker = WalkDir::new(&cli.path)
        .min_depth(1)
        .max_depth(if cli.recursive { usize::MAX } else { 1 })
        .contents_first(true);

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[walk-error] {e}");
                stats.errors += 1;
                continue;
            }
        };

        let is_dir = entry.file_type().is_dir();
        if is_dir && !cli.include_dirs {
            continue;
        }

        let path = entry.path();
        let Some(file_name) = path.file_name() else {
            continue;
        };
        let Some(name_str) = file_name.to_str() else {
            eprintln!("[skip-non-utf8] {}", path.display());
            stats.skipped_non_utf8 += 1;
            continue;
        };

        let nfd: String = name_str.nfd().collect();
        if nfd == name_str {
            stats.already_nfd += 1;
            if cli.verbose {
                println!("[ok] {}", path.display());
            }
            continue;
        }

        let new_path = match path.parent() {
            Some(p) => p.join(&nfd),
            None => continue,
        };

        if new_path.exists() {
            eprintln!(
                "[collision] {}\n            target exists: {}",
                path.display(),
                new_path.display()
            );
            stats.collisions += 1;
            continue;
        }

        if cli.dry_run {
            println!("[dry-run] {}", path.display());
            stats.renamed += 1;
            continue;
        }

        match fs::rename(path, &new_path) {
            Ok(()) => {
                println!("[rename] {}", path.display());
                stats.renamed += 1;
            }
            Err(e) => {
                eprintln!("[error] {}: {}", path.display(), e);
                stats.errors += 1;
            }
        }
    }

    eprintln!();
    eprintln!(
        "Summary: {} {}, {} already NFD, {} collisions, {} errors{}",
        stats.renamed,
        if cli.dry_run { "would rename" } else { "renamed" },
        stats.already_nfd,
        stats.collisions,
        stats.errors,
        if stats.skipped_non_utf8 > 0 {
            format!(", {} non-UTF-8 skipped", stats.skipped_non_utf8)
        } else {
            String::new()
        },
    );

    if stats.errors > 0 || stats.collisions > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
