#[cfg(target_os = "linux")]
use std::env;

/// Keep reqwest's system-proxy support enabled for environment and platform
/// settings. KDE stores its manual proxy separately from the process
/// environment, so use kioslaverc as a Linux desktop fallback when no proxy
/// environment variable is present.
pub fn configure_client(builder: reqwest::ClientBuilder) -> Result<reqwest::ClientBuilder, String> {
    #[cfg(target_os = "linux")]
    {
        if !proxy_environment_configured() {
            if let Some(settings) = read_kde_proxy_settings()? {
                return apply_kde_proxy(builder, settings);
            }
        }
    }

    Ok(builder)
}

#[cfg(target_os = "linux")]
#[derive(Debug, PartialEq, Eq)]
struct KdeProxySettings {
    http_proxy: Option<String>,
    https_proxy: Option<String>,
    no_proxy: Option<String>,
}

#[cfg(target_os = "linux")]
fn proxy_environment_configured() -> bool {
    [
        "HTTP_PROXY",
        "http_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "ALL_PROXY",
        "all_proxy",
    ]
    .iter()
    .any(|name| env::var_os(name).is_some_and(|value| !value.is_empty()))
}

#[cfg(target_os = "linux")]
fn read_kde_proxy_settings() -> Result<Option<KdeProxySettings>, String> {
    use std::{fs, io::ErrorKind, path::PathBuf};

    let config_path = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .map(|path| path.join("kioslaverc"))
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".config/kioslaverc"))
        });
    let Some(config_path) = config_path else {
        return Ok(None);
    };

    let contents = match fs::read_to_string(config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("无法读取 KDE 代理设置（kioslaverc）".into()),
    };

    Ok(parse_kioslaverc(&contents))
}

#[cfg(target_os = "linux")]
fn parse_kioslaverc(contents: &str) -> Option<KdeProxySettings> {
    let mut in_proxy_settings = false;
    let mut proxy_type = None;
    let mut http_proxy = None;
    let mut https_proxy = None;
    let mut no_proxy = None;

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_proxy_settings = &line[1..line.len() - 1] == "Proxy Settings";
            continue;
        }
        if !in_proxy_settings {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "ProxyType" => proxy_type = Some(value),
            "httpProxy" => http_proxy = normalize_proxy_url(value),
            "httpsProxy" => https_proxy = normalize_proxy_url(value),
            "NoProxyFor" => no_proxy = non_empty(value).map(str::to_owned),
            _ => {}
        }
    }

    if proxy_type != Some("1") {
        return None;
    }

    if http_proxy.is_none() && https_proxy.is_none() {
        return None;
    }
    let https_proxy = https_proxy.or_else(|| http_proxy.clone());
    Some(KdeProxySettings {
        http_proxy,
        https_proxy,
        no_proxy,
    })
}

#[cfg(target_os = "linux")]
fn normalize_proxy_url(value: &str) -> Option<String> {
    let value = non_empty(value)?;
    if value.contains("://") {
        return Some(value.to_owned());
    }

    let mut parts = value.split_whitespace();
    let host = parts.next()?;
    match (parts.next(), parts.next()) {
        (Some(port), None) => Some(format!("http://{host}:{port}")),
        (None, None) => Some(format!("http://{value}")),
        _ => Some(format!("http://{value}")),
    }
}

#[cfg(target_os = "linux")]
fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

#[cfg(target_os = "linux")]
fn apply_kde_proxy(
    builder: reqwest::ClientBuilder,
    settings: KdeProxySettings,
) -> Result<reqwest::ClientBuilder, String> {
    let no_proxy = combined_no_proxy(settings.no_proxy.as_deref());
    let mut builder = builder;

    if let Some(proxy_url) = settings.http_proxy {
        let proxy = reqwest::Proxy::http(proxy_url.as_str())
            .map_err(|_| "KDE 的 HTTP 代理地址无效（httpProxy）")?
            .no_proxy(no_proxy.clone());
        builder = builder.proxy(proxy);
    }
    if let Some(proxy_url) = settings.https_proxy {
        let proxy = reqwest::Proxy::https(proxy_url.as_str())
            .map_err(|_| "KDE 的 HTTPS 代理地址无效（httpsProxy）")?
            .no_proxy(no_proxy);
        builder = builder.proxy(proxy);
    }

    Ok(builder)
}

#[cfg(target_os = "linux")]
fn combined_no_proxy(kde_no_proxy: Option<&str>) -> Option<reqwest::NoProxy> {
    let environment_no_proxy = env::var("NO_PROXY")
        .ok()
        .or_else(|| env::var("no_proxy").ok());
    let entries = [environment_no_proxy.as_deref(), kde_no_proxy]
        .into_iter()
        .filter_map(|value| value.and_then(non_empty))
        .collect::<Vec<_>>()
        .join(",");

    reqwest::NoProxy::from_string(&entries)
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::{parse_kioslaverc, KdeProxySettings};

    #[test]
    fn reads_manual_kde_proxy_and_bypass_list() {
        let settings = parse_kioslaverc(
            "[Proxy Settings]\nProxyType=1\nhttpProxy=127.0.0.1 8080\nhttpsProxy=http://127.0.0.1:8081\nNoProxyFor=localhost, .internal\n",
        );

        assert_eq!(
            settings,
            Some(KdeProxySettings {
                http_proxy: Some("http://127.0.0.1:8080".into()),
                https_proxy: Some("http://127.0.0.1:8081".into()),
                no_proxy: Some("localhost, .internal".into()),
            })
        );
    }

    #[test]
    fn reuses_http_proxy_for_https_when_kde_has_no_https_proxy() {
        let settings =
            parse_kioslaverc("[Proxy Settings]\nProxyType=1\nhttpProxy=http://127.0.0.1:8080\n");

        assert_eq!(
            settings,
            Some(KdeProxySettings {
                http_proxy: Some("http://127.0.0.1:8080".into()),
                https_proxy: Some("http://127.0.0.1:8080".into()),
                no_proxy: None,
            })
        );
    }

    #[test]
    fn accepts_https_only_kde_proxy() {
        assert_eq!(
            parse_kioslaverc("[Proxy Settings]\nProxyType=1\nhttpsProxy=http://127.0.0.1:8081\n"),
            Some(KdeProxySettings {
                http_proxy: None,
                https_proxy: Some("http://127.0.0.1:8081".into()),
                no_proxy: None,
            })
        );
    }

    #[test]
    fn ignores_disabled_or_automatic_kde_proxy_modes() {
        assert_eq!(
            parse_kioslaverc("[Proxy Settings]\nProxyType=0\nhttpProxy=http://127.0.0.1:8080\n"),
            None
        );
        assert_eq!(
            parse_kioslaverc("[Proxy Settings]\nProxyType=3\nhttpProxy=http://127.0.0.1:8080\n"),
            None
        );
    }
}
