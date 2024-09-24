use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    // Prepare what to copy and how
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let paths_to_copy = vec!["data"];

    // Determine the profile (debug or release)
    let profile = env::var("PROFILE")?; // Will be "debug" or "release"

    // Construct the path to the target directory
    let mut target_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    target_dir.push("target");
    target_dir.push(&profile); // target/debug or target/release

    // Create the target directory if it doesn't exist
    std::fs::create_dir_all(&target_dir)?;

    // Copy the items to the directory where the executable will be placed
    copy_items(&paths_to_copy, &target_dir, &copy_options)?;

    println!("cargo:rerun-if-changed=data/*"); // Ensure the build script runs if data changes

    Ok(())
}
