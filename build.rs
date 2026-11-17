//! Build script: embeds the icon and version details into the Windows executable.

/// Does nothing unless the target is Windows.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=packaging/windows/abstractions.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed();
    }
}

/// Compiles the icon, the product name and the package version into a resource linked into the executable.
#[cfg(windows)]
fn embed() {
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon("packaging/windows/abstractions.ico")
        .set("ProductName", "Abstractions")
        .set("FileDescription", "Abstractions");
    resource.compile().expect("the Windows resource compiles");
}

/// Skips the resource: the resource compiler crate is only a build dependency on Windows hosts.
#[cfg(not(windows))]
fn embed() {
    println!("cargo:warning=the executable icon is only embedded when the build runs on Windows");
}
