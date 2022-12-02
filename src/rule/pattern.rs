use wildmatch::WildMatch;

#[derive(Debug)]
pub struct PathPattern<'a> {
    pattern: &'a str,
}

impl<'a> From<&'a str> for PathPattern<'a> {
    fn from(pattern: &'a str) -> Self {
        Self { pattern }
    }
}

impl Into<WildMatch> for PathPattern<'_> {
    fn into(self) -> WildMatch {
        if let Some(pattern) = self.pattern.strip_suffix("::") {
            WildMatch::new(pattern)
        } else {
            WildMatch::new(self.pattern)
        }
    }
}

impl PathPattern<'_> {
    pub fn matches_struct_path(self, path: &str) -> bool {
        let Some((path, _item)) = path.rsplit_once("::") else {
            return false;
        };

        <PathPattern<'_> as Into<WildMatch>>::into(self).matches(path)
    }

    pub fn matches_module_path(self, path: &str) -> bool {
        <PathPattern<'_> as Into<WildMatch>>::into(self).matches(path)
    }
}

#[cfg(test)]
mod test {
    use speculoos::prelude::*;

    use crate::rule::pattern::PathPattern;

    #[test]
    fn wildcard_only_should_match() {
        let pattern = PathPattern::from("*");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_true();
    }

    #[test]
    fn wildcard_suffix_should_match() {
        let pattern = PathPattern::from("archunit_rs::*");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_true();

        let pattern = PathPattern::from("archunit_rs::rule::*");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_true();
    }

    #[test]
    fn wildcard_prefix_should_match() {
        let pattern = PathPattern::from("*::rule::*");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_true();

        let pattern = PathPattern::from("*::modules");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_true();
    }

    #[test]
    fn inner_wildcard_should_match() {
        let pattern = PathPattern::from("archunit_rs::*::modules");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_true();
    }

    #[test]
    fn inner_wildcard_should_fail() {
        let pattern = PathPattern::from("archunit_rs::*::mod");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_false();
    }

    #[test]
    fn wild_card_does_not_match_when_after_direct_parent() {
        let pattern = PathPattern::from("archunit_rs::rule::modules::*");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_false();
    }

    #[test]
    fn should_match_exact() {
        let pattern = PathPattern::from("archunit_rs::rule::modules");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_true();
    }

    #[test]
    fn should_not_match() {
        let pattern = PathPattern::from("archunit_rs");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_false();

        let pattern = PathPattern::from("archunit_rs::rule");
        assert_that!(pattern.matches_struct_path("archunit_rs::rule::modules::Modules")).is_false();
    }

    #[test]
    fn match_module_path() {
        let pattern = PathPattern::from("archunit_rs::rule::modules::*");
        assert_that!(pattern.matches_module_path("archunit_rs::rule::modules::module_test"))
            .is_true();
    }
}
