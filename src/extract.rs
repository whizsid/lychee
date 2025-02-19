use linkify::LinkFinder;

use std::{collections::HashSet, fmt::Display};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Uri {
    Website(Url),
    Mail(String),
}

impl Uri {
    pub fn as_str(&self) -> &str {
        match self {
            Uri::Website(url) => url.as_str(),
            Uri::Mail(address) => address.as_str(),
        }
    }

    pub fn scheme(&self) -> Option<String> {
        match self {
            Uri::Website(url) => Some(url.scheme().to_string()),
            Uri::Mail(_address) => None,
        }
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Use LinkFinder here to offload the actual link searching
fn find_links(input: &str) -> Vec<linkify::Link> {
    let finder = LinkFinder::new();
    finder.links(input).collect()
}

pub(crate) fn extract_links(input: &str) -> HashSet<Uri> {
    let links = find_links(input);
    // Only keep legit URLs. This sorts out things like anchors.
    // Silently ignore the parse failures for now.
    let mut uris = HashSet::new();
    for link in links {
        match Url::parse(link.as_str()) {
            Ok(url) => uris.insert(Uri::Website(url)),
            Err(_) => uris.insert(Uri::Mail(link.as_str().to_owned())),
        };
    }
    debug!("Found: {:#?}", uris);
    uris
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn test_extract_markdown_links() {
        let input = "This is [a test](https://endler.dev).";
        let links = extract_links(input);
        assert_eq!(
            links,
            HashSet::from_iter(
                [Uri::Website(Url::parse("https://endler.dev").unwrap())]
                    .iter()
                    .cloned()
            )
        )
    }

    #[test]
    fn test_skip_markdown_anchors() {
        let input = "This is [a test](#lol).";
        let links = extract_links(input);
        assert_eq!(links, HashSet::new())
    }

    #[test]
    fn test_skip_markdown_internal_urls() {
        let input = "This is [a test](./internal).";
        let links = extract_links(input);
        assert_eq!(links, HashSet::new())
    }

    #[test]
    fn test_non_markdown_links() {
        let input =
            "https://endler.dev and https://hello-rust.show/foo/bar?lol=1 at test@example.com";
        let links = extract_links(input);
        let expected = HashSet::from_iter(
            [
                Uri::Website(Url::parse("https://endler.dev").unwrap()),
                Uri::Website(Url::parse("https://hello-rust.show/foo/bar?lol=1").unwrap()),
                Uri::Mail("test@example.com".to_string()),
            ]
            .iter()
            .cloned(),
        );
        assert_eq!(links, expected)
    }

    #[test]
    #[ignore]
    // TODO: Does this escaping need to work properly?
    // See https://github.com/tcort/markdown-link-check/issues/37
    fn test_md_escape() {
        let input = r#"http://msdn.microsoft.com/library/ie/ms535874\(v=vs.85\).aspx"#;
        let links = find_links(input);
        let expected = "http://msdn.microsoft.com/library/ie/ms535874(v=vs.85).aspx)";
        assert!(links.len() == 1);
        assert_eq!(links[0].as_str(), expected);
    }
}
