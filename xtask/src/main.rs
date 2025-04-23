use anyhow::{anyhow, Result};
use clap::Parser;
use shared::cli::LoginData;
use xshell::{cmd, Shell};

fn main() -> Result<()> {
    let is_debug_build = {
        #[cfg(not(debug_assertions))]
        let res = false;

        #[cfg(debug_assertions)]
        let res = true;

        res
    };

    match shared::cli::Command::parse() {
        shared::cli::Command::Post { login_data, src } => {
            compile_all(is_debug_build)?;

            println!("Posting contents of {src} as an atpage website...");
            assemble(publish(login_data, src)?)?;

            println!("Website posted! Now publish the contents of the `public` folder somewhere and have fun :)");

            Ok(())
        }
        shared::cli::Command::Nuke(ld) => nuke(ld),
        shared::cli::Command::Compile { at_uri } => {
            compile_all(is_debug_build)?;
            if !at_uri.starts_with("at://") {
                return Err(anyhow!("aturi argument must be a valid AT URI"));
            }

            Ok(assemble(at_uri)?)
        }
    }
}

fn compile_all(release: bool) -> Result<()> {
    let sh = Shell::new()?;

    let render_targets = [("web", "mod"), ("no-modules", "nomod")];

    let opt_target = {
        match release {
            true => "release",
            false => "dev",
        }
    };

    for (rt, dir) in render_targets {
        cmd!(sh, "wasm-pack build --{opt_target} --no-typescript --target {rt} atpage_renderer")
        .run()?;

        let destdir = format!("public/{}", dir);
        sh.create_dir(destdir.clone())?;

        for i in sh.read_dir("atpage_renderer/pkg")? {
            sh.copy_file(i, destdir.clone())?;
        }
    }

    Ok(())
}

fn publish(ld: LoginData, src: String) -> Result<String> {
    let sh = Shell::new()?;

    // compile atpage_publisher
    cmd!(sh, "cargo build --release --package atpage_publisher").run()?;

    let (username, password, pds) = (ld.username, ld.password, ld.pds);
    let res = cmd!(
        sh,
        "target/release/atpage_publisher post --username {username} --password {password} --src {src} --pds {pds}"
    )
    .read()?;

    Ok(res.trim_start_matches("ATPage index URI: ").to_string())
}

fn nuke(ld: LoginData) -> Result<()> {
    let sh = Shell::new()?;

    // compile atpage_publisher
    cmd!(sh, "cargo build --release --package atpage_publisher").run()?;

    let (username, password, pds) = (ld.username, ld.password, ld.pds);
    Ok(cmd!(
        sh,
        "target/release/atpage_publisher nuke --username {username} --password {password} --pds {pds}"
    )
    .run()?)
}

fn assemble(at_uri: String) -> Result<()> {
    let at_uri = at_uri.replace("at://", "/at/");

    let sh = Shell::new()?;

    for i in sh.read_dir("template")? {
        sh.copy_file(i, "public/")?;
    }

    let ijs = sh.read_file("template/index.js")?;

    let ijs = ijs.replace("REPLACE_ME", &at_uri);

    sh.write_file("public/index.js", ijs)?;

    Ok(())
}
