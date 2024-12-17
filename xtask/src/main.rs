use anyhow::Result;
use clap::Parser;
use shared::cli::LoginData;
use xshell::{cmd, Shell};

fn main() -> Result<()> {
    match shared::cli::Command::parse() {
        shared::cli::Command::Post { login_data, src } => {
            compile_all()?;

            println!("Posting contents of {src} as an atpage website...");
            assemble(publish(login_data, src)?)?;

            println!("Website posted! Now publish the contents of the `public` folder somewhere and have fun :)");

            Ok(())
        }
        shared::cli::Command::Nuke(ld) => nuke(ld),
    }
}

fn compile_all() -> Result<()> {
    let sh = Shell::new()?;

    let render_targets = [("web", "mod"), ("no-modules", "nomod")];
    for (rt, dir) in render_targets {
        cmd!(
            sh,
            "wasm-pack build --release --no-typescript --target {rt} atpage_renderer"
        )
        .run()?;

        let destdir = format!("public/{}", dir);
        sh.create_dir(destdir.clone())?;

        for i in sh.read_dir("atpage_renderer/pkg")? {
            sh.copy_file(i, destdir.clone())?;
        }
    }

    // compile publish
    cmd!(sh, "cargo build --release --package publish").run()?;

    Ok(())
}

fn publish(ld: LoginData, src: String) -> Result<String> {
    let sh = Shell::new()?;

    let (username, password, pds) = (ld.username, ld.password, ld.pds);
    let res = cmd!(
        sh,
        "target/release/publish post --username {username} --password {password} --src {src} --pds {pds}"
    )
    .read()?;

    Ok(res.replace("at://", "/at/"))
}

fn nuke(ld: LoginData) -> Result<()> {
    let sh = Shell::new()?;

    let (username, password, pds) = (ld.username, ld.password, ld.pds);
    Ok(cmd!(
        sh,
        "target/release/publish nuke --username {username} --password {password} --pds {pds}"
    )
    .run()?)
}

fn assemble(at_uri: String) -> Result<()> {
    let sh = Shell::new()?;

    for i in sh.read_dir("template")? {
        sh.copy_file(i, "public/")?;
    }

    let ijs = sh.read_file("template/index.js")?;

    let ijs = ijs.replace("REPLACE_ME", &at_uri);

    sh.write_file("public/index.js", ijs)?;

    Ok(())
}
