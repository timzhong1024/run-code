pub const SKILL_MD: &str = include_str!("../skills/run-code-snippet/SKILL.md");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_skill_has_the_expected_identity() {
        assert!(SKILL_MD.starts_with("---\nname: run-code-snippet\n"));
        assert!(SKILL_MD.contains("run-code TOOLCHAIN[@VERSION]"));
    }
}
