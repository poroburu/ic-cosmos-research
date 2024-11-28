use std::{error::Error, fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Cluster {
    Testnet,
    Mainnet,
    Custom(String, String),
}

#[derive(Debug)]
pub enum ClusterError {
    InvalidCluster,
    UrlParseError(url::ParseError),
    SetPortError,
    SetSchemeError,
}

impl fmt::Display for ClusterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClusterError::InvalidCluster => write!(f, "Invalid cluster"),
            ClusterError::UrlParseError(e) => write!(f, "URL parse error: {}", e),
            ClusterError::SetPortError => write!(f, "Unable to set port"),
            ClusterError::SetSchemeError => write!(f, "Unable to set scheme"),
        }
    }
}

impl Error for ClusterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClusterError::UrlParseError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<url::ParseError> for ClusterError {
    fn from(e: url::ParseError) -> Self {
        ClusterError::UrlParseError(e)
    }
}

impl FromStr for Cluster {
    type Err = ClusterError;

    fn from_str(s: &str) -> Result<Cluster, ClusterError> {
        match s.to_ascii_lowercase().as_str() {
            "t" | "testnet" => Ok(Cluster::Testnet),
            "m" | "mainnet" => Ok(Cluster::Mainnet),
            _ if s.starts_with("http") => {
                let mut ws_url = Url::parse(s)?;
                if let Some(port) = ws_url.port() {
                    ws_url
                        .set_port(Some(port + 1))
                        .map_err(|_| ClusterError::SetPortError)?;
                }
                if ws_url.scheme() == "https" {
                    ws_url
                        .set_scheme("wss")
                        .map_err(|_| ClusterError::SetSchemeError)?;
                } else {
                    ws_url
                        .set_scheme("ws")
                        .map_err(|_| ClusterError::SetSchemeError)?;
                }
                Ok(Cluster::Custom(s.to_string(), ws_url.to_string()))
            }
            _ => Err(ClusterError::InvalidCluster),
        }
    }
}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let cluster_str = match self {
            Cluster::Testnet => "testnet",
            Cluster::Mainnet => "mainnet",
            Cluster::Custom(url, _ws_url) => url,
        };
        write!(f, "{cluster_str}")
    }
}

impl Cluster {
    pub fn url(&self) -> &str {
        match self {
            Cluster::Testnet => "https://neutron-testnet-rpc.polkachu.com",
            Cluster::Mainnet => "https://neutron-rpc.polkachu.com/",
            Cluster::Custom(url, _ws_url) => url,
        }
    }
    pub fn ws_url(&self) -> &str {
        match self {
            Cluster::Testnet => "https://neutron-testnet-rpc.polkachu.com",
            Cluster::Mainnet => "https://neutron-rpc.polkachu.com/",
            Cluster::Custom(_url, ws_url) => ws_url,
        }
    }
    pub fn host_str(&self) -> Option<String> {
        Url::parse(self.url())
            .ok()
            .and_then(|url| url.host_str().map(|s| s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cluster(name: &str, cluster: Cluster) {
        assert_eq!(Cluster::from_str(name).unwrap(), cluster);
    }

    #[test]
    fn test_cluster_parse() {
        test_cluster("testnet", Cluster::Testnet);
        test_cluster("mainnet", Cluster::Mainnet);
    }

    #[test]
    #[should_panic]
    fn test_cluster_bad_parse() {
        let bad_url = "httq://my_custom_url.test.net";
        Cluster::from_str(bad_url).unwrap();
    }
    #[test]
    fn test_http_port() {
        let url = "http://my-url.com:7000/";
        let ws_url = "ws://my-url.com:7001/";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), ws_url.to_string()),
            cluster
        );
    }
    #[test]
    fn test_http_no_port() {
        let url = "http://my-url.com/";
        let ws_url = "ws://my-url.com/";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), ws_url.to_string()),
            cluster
        );
    }
    #[test]
    fn test_https_port() {
        let url = "https://my-url.com:7000/";
        let ws_url = "wss://my-url.com:7001/";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), ws_url.to_string()),
            cluster
        );
    }
    #[test]
    fn test_https_no_port() {
        let url = "https://my-url.com/";
        let ws_url = "wss://my-url.com/";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), ws_url.to_string()),
            cluster
        );
    }

    #[test]
    fn test_upper_case() {
        let url = "http://my-url.com/FooBar";
        let ws_url = "ws://my-url.com/FooBar";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), ws_url.to_string()),
            cluster
        );
    }
}
