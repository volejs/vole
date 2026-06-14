pub mod url_class;
pub mod url_pattern;
pub mod url_search_params;

use rquickjs::{Class, Ctx, Result};
use url::Url;
use vole_core::primordials::{BasePrimordials, Primordial};

use self::url_class::URL;
use self::url_pattern::URLPattern;
use self::url_search_params::URLSearchParams;

/// Returns whether the given scheme is a WHATWG special scheme.
pub fn is_special_scheme(scheme: &str) -> bool {
    matches!(scheme, "http" | "https" | "ftp" | "ws" | "wss" | "file")
}

/// Encode trailing space as `%20` in opaque paths before a setter runs.
pub fn convert_trailing_space(url: &mut Url) {
    if is_special_scheme(url.scheme()) {
        return;
    }

    let path = url.path();
    let has_remaining = url.fragment().is_some() || url.query().is_some();

    if path.ends_with(' ') && has_remaining {
        let new_path = [&path[..path.len() - 1], "%20"].concat();
        url.set_path(&new_path);
    }
}

/// Preserve non-empty hosts in `file://HOST/...` URLs where the `url` crate
/// normalizes the host away for Windows-drive-looking paths.
pub fn preserve_file_url_host(source: &str, mut url: Url) -> Url {
    if url.scheme() != "file" {
        return url;
    }
    if url.host_str().is_some_and(|h| !h.is_empty()) {
        return url;
    }
    let Some(rest) = source.strip_prefix("file://") else {
        return url;
    };
    let Some((host, _)) = rest.split_once('/') else {
        return url;
    };
    if host.is_empty() {
        return url;
    }
    let _ = url.set_host(Some(host));
    url
}

/// Restore file URL host lost by `Url::join` for Windows drive-letter paths.
pub fn restore_file_url_host(base: &Url, joined: &mut Url) {
    if base.scheme() != "file" || joined.scheme() != "file" {
        return;
    }
    let Some(base_host) = base.host_str() else {
        return;
    };
    if base_host.is_empty() {
        return;
    }
    if joined.host_str().is_some_and(|h| !h.is_empty()) {
        return;
    }

    let base_path = base.path();
    let is_drive_letter_first_seg = base_path
        .as_bytes()
        .get(1)
        .is_some_and(|b| b.is_ascii_alphabetic())
        && base_path.as_bytes().get(2) == Some(&b':')
        && matches!(base_path.as_bytes().get(3), Some(&b'/') | None);
    if !is_drive_letter_first_seg {
        return;
    }
    let _ = joined.set_host(Some(base_host));
}

pub fn normalize_windows_drive_letter(url: &mut Url) {
    if url.scheme() != "file" {
        return;
    }
    let path = url.path();
    let bytes = path.as_bytes();
    if bytes.len() < 4
        || bytes[0] != b'/'
        || !bytes[1].is_ascii_alphabetic()
        || bytes[2] != b'|'
        || bytes[3] != b'/'
    {
        return;
    }
    let new_path = ["/", &path[1..2], ":", &path[3..]].concat();
    url.set_path(&new_path);
}

/// Per WHATWG URL spec, a non-special URL with an empty host can have its path erased.
pub fn erase_empty_host_path(url: &mut Url) {
    if is_special_scheme(url.scheme()) {
        return;
    }
    if url.path() != "/" {
        return;
    }
    let serialized = url.as_str();
    let Some(scheme_end) = serialized.find("://") else {
        return;
    };
    let authority_and_path = &serialized[scheme_end + 3..];
    if authority_and_path != "/" {
        return;
    }
    let stripped = &serialized[..serialized.len() - 1];
    if let Ok(reparsed) = Url::parse(stripped) {
        *url = reparsed;
    }
}

pub fn strip_path_sentinel(url: &mut Url) {
    if is_special_scheme(url.scheme()) {
        return;
    }
    if !url.path().starts_with("//") {
        return;
    }
    let serialized = url.as_str();
    let Some(auth_start) = serialized.find("://") else {
        return;
    };
    let after_auth = auth_start + 3;
    let Some(path_start_rel) = serialized[after_auth..].find('/') else {
        return;
    };
    let path_idx = after_auth + path_start_rel;
    if serialized[path_idx..].starts_with("/./") {
        let stripped = [&serialized[..path_idx], &serialized[path_idx + 2..]].concat();
        if let Ok(reparsed) = Url::parse(&stripped) {
            *url = reparsed;
        }
    }
}

pub fn init(ctx: &Ctx) -> Result<()> {
    BasePrimordials::init(ctx)?;

    let globals = ctx.globals();
    Class::<URLSearchParams>::define(&globals)?;
    Class::<URL>::define(&globals)?;
    Class::<URLPattern>::define(&globals)?;

    Ok(())
}
