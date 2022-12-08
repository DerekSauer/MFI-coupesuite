use coupesuite_shared::settings::Settings;
use serde::Deserialize;
use std::{fs::File, io::Read, path::PathBuf};

/// Root structure of the workspace's `Cargo.toml`.
#[derive(Deserialize, Debug)]
struct WorkspaceCargo {
    /// Settings specific to the workspace.
    workspace: Workspace,
}

/// Settings specific to the workspace.
#[derive(Deserialize, Debug)]
#[serde(rename = "workspace")]
struct Workspace {
    /// List of workspace members.
    members: Vec<String>,
}

/// Package build artifacts of this project into a `dist` directory
/// that can be installed on other machines. This application must be called
/// from the root directory of the workspace.
fn main() -> anyhow::Result<()> {
    let app_settings = Settings::load(&std::env::current_dir()?.join("coupesuite.toml"))?;

    let root_directory = std::env::current_dir()?;

    // Create and/or empty the `dist` directory
    fs_extra::dir::create("./dist/", true)?;

    // Load the `Cargo.toml` file from the workspace root
    let mut cargo_toml = String::new();
    File::open(&root_directory.join("Cargo.toml"))
        .and_then(|mut file| file.read_to_string(&mut cargo_toml))?;

    // Parse the workspace section of the Cargo manifest
    // to get the name of each sub-project
    let mut cargo_toml: WorkspaceCargo = toml::from_str(&cargo_toml)?;

    // Remove the 'shared' project as it has no executables and is statically linked
    // to other workspace members requiring it
    cargo_toml
        .workspace
        .members
        .retain(|member| *member != "shared");

    // Remove the `publish` project, its not needed for end users
    cargo_toml
        .workspace
        .members
        .retain(|member| *member != "publish");

    // Visit each project
    for project in cargo_toml.workspace.members {
        let target = &root_directory
            .join("target")
            .join("release")
            .join(format!("{}.exe", &project));

        // If the project's executable does not exist, build it
        if !&target.try_exists()? {
            println!("Executable for `{}` not found, building.", &project);

            std::process::Command::new("cargo")
                .arg("build")
                .arg(format!("--bin={}", &project))
                .arg("--release")
                .output()?;
        }

        // Copy the project's executable to the `dist` directory
        let destination = &root_directory
            .join("dist")
            .join(format!("{}.exe", &project));
        fs_extra::file::copy(target, destination, &fs_extra::file::CopyOptions::new())?;

        println!("{} -> {}", &target.display(), &destination.display());
    }

    // Copy the `www` directory to the `dist` directory
    fs_extra::dir::copy(
        &root_directory.join("www"),
        &root_directory.join("dist"),
        &fs_extra::dir::CopyOptions::new(),
    )?;

    println!(
        "{} -> {}",
        &root_directory.join("www").display(),
        &root_directory.join("dist").join("www").display()
    );

    // Finally, copy the settings file
    fs_extra::file::copy(
        &root_directory.join("coupesuite.toml"),
        &root_directory.join("dist").join("coupesuite.toml"),
        &fs_extra::file::CopyOptions::new(),
    )?;

    println!(
        "{} -> {}",
        &root_directory.join("coupesuite.toml").display(),
        &root_directory
            .join("dist")
            .join("coupesuite.toml")
            .display()
    );

    // Copy Ghostscript binaries to the workspace directory
    let mut gs_path = PathBuf::from(&app_settings.ghostscript.location);
    gs_path.pop();

    let copy_options = fs_extra::dir::CopyOptions {
        content_only: true,
        ..Default::default()
    };
    fs_extra::dir::copy(&gs_path, &root_directory.join("dist"), &copy_options)?;

    println!(
        "{} -> {}",
        &gs_path.display(),
        &root_directory.join("dist").display()
    );

    Ok(())
}
