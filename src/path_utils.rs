/// Strip leading and trailing slashes from a release path prefix.
pub fn normalize_prefix(path: &str) -> String {
    path.trim_matches('/').to_string()
}

/// Build the S3 object key from a path prefix and filename.
pub fn object_key(path: &str, filename: &str) -> String {
    let prefix = normalize_prefix(path);
    if prefix.is_empty() {
        filename.to_string()
    } else {
        format!("{}/{}", prefix, filename)
    }
}

/// Build the full public URL for a file, encoding special characters in the filename.
pub fn build_public_url(base: &str, path: &str, filename: &str) -> String {
    use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

    // Encode everything except unreserved chars and common filename-safe chars
    const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'_')
        .remove(b'.')
        .remove(b'~');

    let encoded = utf8_percent_encode(filename, ENCODE_SET).to_string();
    let trimmed_base = base.trim_end_matches('/');
    let prefix = normalize_prefix(path);
    if prefix.is_empty() {
        format!("{}/{}", trimmed_base, encoded)
    } else {
        format!("{}/{}/{}", trimmed_base, prefix, encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_prefix_strips_leading_and_trailing_slashes() {
        assert_eq!(normalize_prefix("/release/v1/"), "release/v1");
        assert_eq!(normalize_prefix("release/v1"), "release/v1");
        assert_eq!(normalize_prefix("/release/v1"), "release/v1");
    }

    #[test]
    fn normalize_prefix_empty_string() {
        assert_eq!(normalize_prefix(""), "");
    }

    #[test]
    fn normalize_prefix_just_slashes() {
        assert_eq!(normalize_prefix("///"), "");
    }

    #[test]
    fn normalize_prefix_deeply_nested() {
        assert_eq!(normalize_prefix("/a/b/c/d/e/f/"), "a/b/c/d/e/f");
    }

    #[test]
    fn object_key_combines_prefix_and_filename() {
        assert_eq!(object_key("/release/v1", "app.zip"), "release/v1/app.zip");
        assert_eq!(object_key("release/v1/", "app.zip"), "release/v1/app.zip");
    }

    #[test]
    fn object_key_with_empty_path() {
        assert_eq!(object_key("", "app.zip"), "app.zip");
    }

    #[test]
    fn object_key_filename_with_special_chars() {
        assert_eq!(
            object_key("/release/v1", "my app (v2.0).tar.gz"),
            "release/v1/my app (v2.0).tar.gz"
        );
    }

    #[test]
    fn build_public_url_trims_slashes_correctly() {
        assert_eq!(
            build_public_url("https://dl.agora.build", "/release/v1/", "app.zip"),
            "https://dl.agora.build/release/v1/app.zip"
        );
        assert_eq!(
            build_public_url("https://dl.agora.build/", "release/v1", "app.zip"),
            "https://dl.agora.build/release/v1/app.zip"
        );
    }

    #[test]
    fn build_public_url_with_empty_path() {
        assert_eq!(
            build_public_url("https://dl.agora.build", "", "app.zip"),
            "https://dl.agora.build/app.zip"
        );
    }

    #[test]
    fn build_public_url_with_port() {
        assert_eq!(
            build_public_url("https://localhost:9000", "/release/v1", "app.zip"),
            "https://localhost:9000/release/v1/app.zip"
        );
    }

    #[test]
    fn build_public_url_path_just_slash() {
        assert_eq!(
            build_public_url("https://dl.agora.build", "/", "file.zip"),
            "https://dl.agora.build/file.zip"
        );
    }

    #[test]
    fn build_public_url_encodes_special_chars() {
        assert_eq!(
            build_public_url("https://dl.agora.build", "/release/v1", "my app (v2.0).tar.gz"),
            "https://dl.agora.build/release/v1/my%20app%20%28v2.0%29.tar.gz"
        );
    }

    #[test]
    fn object_key_deeply_nested_path() {
        assert_eq!(
            object_key("/org/product/v2/nightly/", "build.tar.gz"),
            "org/product/v2/nightly/build.tar.gz"
        );
    }
}
