// TODO: move all of this into easy-to-use crate

use lazy_static::lazy_static;
use log::info;
use std::{
    fs::{create_dir_all, rename, File},
    io::{Read, Seek, Write},
    path::Path,
};
use walkdir::WalkDir;
use xtask_wasm::{anyhow::Result, clap};
use zip::{result::ZipError, write::FileOptions};

#[derive(clap::Parser)]
struct Opt {
    #[clap(long = "log", default_value = "Info")]
    log_level: log::LevelFilter,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(clap::Parser)]
enum Command {
    Dist(Build),
}

#[derive(clap::Parser)]
struct Build {
    #[clap(flatten)]
    base: xtask_wasm::Dist,
    /// The package to build
    #[clap(index = 1)]
    package: Option<String>,
}

fn main() -> Result<()> {
    let opt: Opt = clap::Parser::parse();

    env_logger::builder()
        .filter(Some("xtask"), opt.log_level)
        .init();

    match opt.cmd {
        Command::Dist(mut arg) => {
            let package_name = arg.package.as_ref().unwrap_or_else(|| {
                &cargo_data()
                    .root_package()
                    .expect(
                        "No root crate, please provide rom crate name or run from rom subdirectory",
                    )
                    .name
            });
            let workspace_root = &cargo_data().workspace_root;
            let dist_root = format!("{workspace_root}/dist");

            info!("Generating package: {package_name}...");

            arg.base.release = true;

            let dist_dir = format!("{dist_root}/{package_name}");

            let dist_result = arg.base.run(package_name)?;

            xtask_wasm::WasmOpt::level(3)
                .shrink(3)
                .optimize(&dist_result.wasm)?;

            info!("Creating dist dir");
            create_dir_all(&dist_dir)?;

            rename(&dist_result.wasm, format!("{dist_dir}/main.wasm"))?;

            let file = File::create(format!("{dist_root}/{package_name}.ultra.zip"))?;

            zip_dir(&dist_dir, &dist_dir, file, zip::CompressionMethod::Stored)?;
        }
    }

    Ok(())
}

fn zip_dir<T>(
    src_dir: &str,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter().filter_map(|e| e.ok());

    let mut zip = zip::ZipWriter::new(writer);

    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            info!("adding file {:?} as {:?} ...", path, name);
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            info!("adding dir {:?} as {:?} ...", path, name);
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}

pub fn cargo_data() -> &'static cargo_metadata::Metadata {
    lazy_static! {
        static ref METADATA: cargo_metadata::Metadata = cargo_metadata::MetadataCommand::new()
            .exec()
            .expect("cannot get crate's metadata");
    }

    &METADATA
}
