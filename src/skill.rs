pub const SKILL_MD: &str = include_str!("../skills/run-code-snippet/SKILL.md");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_skill_has_the_expected_identity() {
        let mut lines = SKILL_MD.lines();
        assert_eq!(lines.next(), Some("---"));
        assert_eq!(lines.next(), Some("name: run-code-snippet"));
        assert!(SKILL_MD.contains("run-code TOOLCHAIN[@VERSION]"));
    }
}
