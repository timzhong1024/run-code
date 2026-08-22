use std::fs;
use std::io::{self, Read};
use std::path::Path;

pub fn read(source: Option<&Path>) -> Result<String, String> {
    match source {
        Some(path) => fs::read_to_string(path)
            .map_err(|error| format!("failed to read source file {}: {error}", path.display())),
        None => {
            let mut code = String::new();
            io::stdin()
                .read_to_string(&mut code)
                .map_err(|error| format!("failed to read stdin: {error}"))?;
            Ok(code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn reads_utf8_source_file() {
        let mut source = tempfile::NamedTempFile::new().unwrap();
        write!(source, "print('copied')").unwrap();
        assert_eq!(read(Some(source.path())).unwrap(), "print('copied')");
    }

    #[test]
    fn reports_the_source_path_on_failure() {
        let path = Path::new("missing-snippet.py");
        let error = read(Some(path)).unwrap_err();
        assert!(error.contains("missing-snippet.py"));
    }
}
