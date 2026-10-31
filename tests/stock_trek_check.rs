use std::{
    fs,
    path::{Path, PathBuf},
};

fn algorithm_files(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
    {
        let path = entry
            .unwrap_or_else(|error| {
                panic!(
                    "failed to read an entry in {}: {error}",
                    directory.display()
                )
            })
            .path();
        if path.is_dir() {
            files.extend(algorithm_files(&path));
        } else {
            files.push(path);
        }
    }
    files.sort();
    files
}

#[test]
fn algorithms_pass_stock_trek_check() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/algorithms");
    let files = algorithm_files(&directory);
    assert!(
        !files.is_empty(),
        "no algorithm files found in {}",
        directory.display()
    );

    let failures = files
        .iter()
        .filter_map(|path| {
            stock_trek_check::verify::verify(path.to_string_lossy().into_owned())
                .err()
                .map(|error| format!("{}:\n{error}", path.display()))
        })
        .collect::<Vec<_>>();

    assert!(
        failures.is_empty(),
        "stock-trek-check rejected algorithm files:\n{}",
        failures.join("\n\n")
    );
}
