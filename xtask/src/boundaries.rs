//! Check actual Cargo manifests rather than relying on architectural convention.

use std::{collections::BTreeSet, error::Error};

pub(crate) fn check() -> Result<(), Box<dyn Error>> {
    let output = super::cargo_command()
        .args([
            "metadata",
            "--offline",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let data: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let packages = data["packages"]
        .as_array()
        .ok_or("metadata packages are missing")?;
    for (name, allowed) in [
        ("ecad-domain", BTreeSet::new()),
        ("ecad-app", BTreeSet::from(["ecad-domain"])),
        (
            "ecad-bridge",
            BTreeSet::from([
                "ecad-domain",
                "ecad-app",
                "ecad-adapters",
                "serde",
                "serde_json",
                "tempfile",
            ]),
        ),
    ] {
        let package = packages
            .iter()
            .find(|package| package["name"] == name)
            .ok_or_else(|| format!("missing crate {name}"))?;
        for dependency in package["dependencies"]
            .as_array()
            .ok_or("dependencies are missing")?
        {
            let dependency = dependency["name"]
                .as_str()
                .ok_or("dependency name is missing")?;
            if !allowed.contains(dependency) {
                return Err(format!(
                    "architecture boundary: {name} must not depend on {dependency}"
                )
                .into());
            }
        }
    }
    println!("Core dependency boundaries passed.");
    Ok(())
}
