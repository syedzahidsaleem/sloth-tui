//! Models for Stremio addon manifests, catalogs, metadata, and streams.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stremio addon resource definition (can be simple string or detailed object).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddonResource {
    /// Simple string resource name (e.g. "stream", "meta", "catalog").
    Simple(String),
    /// Detailed resource declaration with supported types and ID prefixes.
    Detailed {
        /// Name of the resource.
        name: String,
        /// Supported media types (e.g. "movie", "series").
        types: Option<Vec<String>>,
        /// Supported ID prefixes (e.g. "tt", "kitsu").
        #[serde(rename = "idPrefixes")]
        id_prefixes: Option<Vec<String>>,
    },
}

impl AddonResource {
    /// Returns the name of the resource.
    pub fn name(&self) -> &str {
        match self {
            Self::Simple(name) => name.as_str(),
            Self::Detailed { name, .. } => name.as_str(),
        }
    }
}

/// Extra query parameter configuration for an addon catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogExtra {
    /// Name of the extra parameter (e.g. "genre", "search", "skip").
    pub name: String,
    /// Whether this extra parameter is required to query the catalog.
    #[serde(rename = "isRequired", default)]
    pub is_required: bool,
    /// Predefined options for this parameter if applicable.
    pub options: Option<Vec<String>>,
}

/// Definition of a catalog provided by a Stremio addon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogDefinition {
    /// Content type of the catalog (e.g. "movie", "series").
    #[serde(rename = "type")]
    pub r#type: String,
    /// Catalog identifier (e.g. "top", "imdbRating").
    pub id: String,
    /// Display name of the catalog.
    pub name: Option<String>,
    /// Extra query parameters supported by the catalog.
    #[serde(default)]
    pub extra: Vec<CatalogExtra>,
}

/// Stremio addon manifest defining metadata, capabilities, and catalogs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddonManifest {
    /// Unique identifier of the addon.
    pub id: String,
    /// Display name of the addon.
    pub name: String,
    /// Version string of the addon.
    pub version: Option<String>,
    /// Short description of what the addon provides.
    pub description: Option<String>,
    /// List of resources supported by the addon.
    #[serde(default)]
    pub resources: Vec<AddonResource>,
    /// List of media types supported by the addon.
    #[serde(default)]
    pub types: Vec<String>,
    /// Catalogs provided by the addon.
    #[serde(default)]
    pub catalogs: Vec<CatalogDefinition>,
    /// ID prefixes handled by the addon (e.g. "tt").
    #[serde(rename = "idPrefixes", default)]
    pub id_prefixes: Vec<String>,
    /// URL to the addon logo image.
    pub logo: Option<String>,
    /// URL to background artwork for the addon.
    pub background: Option<String>,
}

impl AddonManifest {
    /// Checks if the manifest declares support for a specific resource name.
    pub fn provides_resource(&self, res_name: &str) -> bool {
        self.resources
            .iter()
            .any(|r| r.name().eq_ignore_ascii_case(res_name))
    }

    /// Checks if the addon provides stream links.
    pub fn provides_stream(&self) -> bool {
        self.provides_resource("stream")
    }

    /// Checks if the addon provides detailed metadata.
    pub fn provides_meta(&self) -> bool {
        self.provides_resource("meta")
    }

    /// Checks if the addon provides browsable catalogs.
    pub fn provides_catalog(&self) -> bool {
        self.provides_resource("catalog") || !self.catalogs.is_empty()
    }
}

fn de_opt_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrNumberVisitor;
    impl<'de> serde::de::Visitor<'de> for StringOrNumberVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string, number, or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            if v.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(v.trim().to_string()))
            }
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            if v.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(v.trim().to_string()))
            }
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}

fn de_opt_usize_or_string<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct UsizeOrStringVisitor;
    impl<'de> serde::de::Visitor<'de> for UsizeOrStringVisitor {
        type Value = Option<usize>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a usize, string, or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v as usize))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            if v >= 0 {
                Ok(Some(v as usize))
            } else {
                Ok(None)
            }
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
            if v >= 0.0 {
                Ok(Some(v as usize))
            } else {
                Ok(None)
            }
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.trim().parse::<usize>().ok())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(v.trim().parse::<usize>().ok())
        }
    }

    deserializer.deserialize_any(UsizeOrStringVisitor)
}

fn de_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrVecVisitor;

    impl<'de> serde::de::Visitor<'de> for StringOrVecVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string, sequence of strings, or sequence of objects")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            let mut list = Vec::new();
            while let Some(val) = seq.next_element::<serde_json::Value>()? {
                if let Some(s) = val.as_str() {
                    let s_trim = s.trim();
                    if !s_trim.is_empty() {
                        list.push(s_trim.to_string());
                    }
                } else if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                    let n_trim = name.trim();
                    if !n_trim.is_empty() {
                        list.push(n_trim.to_string());
                    }
                }
            }
            Ok(list)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![v.to_string()])
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![v.to_string()])
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![v.to_string()])
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Vec::new())
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Vec::new())
        }
    }

    deserializer.deserialize_any(StringOrVecVisitor)
}

/// Catalog metadata item returned in Stremio catalog listings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaItem {
    /// Item identifier (e.g. IMDb ID "tt1234567").
    pub id: String,
    /// Item content type (e.g. "movie", "series").
    #[serde(rename = "type", default)]
    pub r#type: String,
    /// Item display name.
    #[serde(default)]
    pub name: String,
    /// Alternative title field.
    #[serde(default)]
    pub title: Option<String>,
    /// URL to poster image.
    pub poster: Option<String>,
    /// URL to cover image.
    pub cover: Option<String>,
    /// Summary description.
    pub description: Option<String>,
    /// Item overview.
    pub overview: Option<String>,
    /// Detailed synopsis.
    pub synopsis: Option<String>,
    /// Release year or date information.
    #[serde(
        rename = "releaseInfo",
        default,
        deserialize_with = "de_opt_string_or_number"
    )]
    pub release_info: Option<String>,
    /// Year of release.
    #[serde(default, deserialize_with = "de_opt_string_or_number")]
    pub year: Option<String>,
    /// Release date string.
    #[serde(default, deserialize_with = "de_opt_string_or_number")]
    pub released: Option<String>,
    /// IMDb rating string (e.g. "8.5").
    #[serde(
        rename = "imdbRating",
        default,
        deserialize_with = "de_opt_string_or_number"
    )]
    pub imdb_rating: Option<String>,
    /// General rating string.
    #[serde(default, deserialize_with = "de_opt_string_or_number")]
    pub rating: Option<String>,
    /// Associated genres (list format).
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub genres: Vec<String>,
    /// Associated genre (single or comma-separated format).
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub genre: Vec<String>,
}

/// Stremio catalog API response wrapper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogResponse {
    /// List of metadata items matching the catalog query.
    #[serde(default)]
    pub metas: Vec<MetaItem>,
}

/// Episode or video metadata in detailed series metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaVideo {
    /// Unique video or episode identifier.
    #[serde(default)]
    pub id: Option<String>,
    /// Episode title.
    pub title: Option<String>,
    /// Alternative name field.
    pub name: Option<String>,
    /// Season number.
    #[serde(default, deserialize_with = "de_opt_usize_or_string")]
    pub season: Option<usize>,
    /// Episode number within season.
    #[serde(default, deserialize_with = "de_opt_usize_or_string")]
    pub episode: Option<usize>,
    /// Absolute episode number.
    #[serde(default, deserialize_with = "de_opt_usize_or_string")]
    pub number: Option<usize>,
    /// Air or release date.
    pub released: Option<String>,
    /// Thumbnail image URL.
    pub thumbnail: Option<String>,
    /// Episode overview synopsis.
    pub overview: Option<String>,
    /// Episode description.
    pub description: Option<String>,
}

/// Detailed metadata record for a movie or series.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaDetail {
    /// Media ID (e.g. IMDb tt ID).
    pub id: String,
    /// Content type (movie, series).
    #[serde(rename = "type", default)]
    pub r#type: String,
    /// Media title / name.
    #[serde(default)]
    pub name: String,
    /// Optional alternative title.
    #[serde(default)]
    pub title: Option<String>,
    /// Poster artwork URL.
    pub poster: Option<String>,
    /// Cover artwork URL.
    pub cover: Option<String>,
    /// Background artwork URL.
    pub background: Option<String>,
    /// Logo image URL.
    pub logo: Option<String>,
    /// Description synopsis.
    pub description: Option<String>,
    /// Overview synopsis.
    pub overview: Option<String>,
    /// Plot synopsis.
    pub synopsis: Option<String>,
    /// Release info string.
    #[serde(
        rename = "releaseInfo",
        default,
        deserialize_with = "de_opt_string_or_number"
    )]
    pub release_info: Option<String>,
    /// Release year.
    #[serde(default, deserialize_with = "de_opt_string_or_number")]
    pub year: Option<String>,
    /// Release date string.
    #[serde(default, deserialize_with = "de_opt_string_or_number")]
    pub released: Option<String>,
    /// IMDb rating score.
    #[serde(
        rename = "imdbRating",
        default,
        deserialize_with = "de_opt_string_or_number"
    )]
    pub imdb_rating: Option<String>,
    /// Rating score string.
    #[serde(default, deserialize_with = "de_opt_string_or_number")]
    pub rating: Option<String>,
    /// Associated genre names.
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub genres: Vec<String>,
    /// Alternative genre field.
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub genre: Vec<String>,
    /// Runtime duration string.
    #[serde(default, deserialize_with = "de_opt_string_or_number")]
    pub runtime: Option<String>,
    /// Cast list.
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub cast: Vec<String>,
    /// Starring actors.
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub stars: Vec<String>,
    /// Director list.
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub director: Vec<String>,
    /// Directors list.
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub directors: Vec<String>,
    /// Writer list.
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub writer: Vec<String>,
    /// Writers list.
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub writers: Vec<String>,
    /// Videos / episodes list for series content.
    #[serde(default)]
    pub videos: Vec<MetaVideo>,
}

/// Extra playback behavior hints provided in Stremio stream responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamBehaviorHints {
    /// Whether the stream requires desktop playback rather than web player.
    #[serde(rename = "notWebReady", default)]
    pub not_web_ready: bool,
    /// Custom HTTP headers required to stream this URL.
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    /// File size in bytes if known.
    #[serde(rename = "videoSize")]
    pub video_size: Option<u64>,
    /// Recommended file name for download/playback.
    pub filename: Option<String>,
}

/// Playable stream item returned by a Stremio stream endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamItem {
    /// Stream name (e.g. source/addon name or quality).
    pub name: Option<String>,
    /// Stream title describing quality, audio, and resolution.
    pub title: Option<String>,
    /// Stream description.
    pub description: Option<String>,
    /// Playable URL (direct video, HLS, or torrent info hash).
    pub url: Option<String>,
    /// Additional streaming behavior hints.
    #[serde(rename = "behaviorHints")]
    pub behavior_hints: Option<StreamBehaviorHints>,
}

/// Configuration record for an installed Stremio addon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstalledAddon {
    /// Addon manifest URL.
    pub manifest_url: String,
    /// Addon display name.
    pub name: String,
    /// Addon version string.
    pub version: Option<String>,
    /// Addon description.
    pub description: Option<String>,
    /// Whether the addon is currently active.
    pub enabled: bool,
    /// Whether the addon provides browsable catalogs.
    #[serde(default)]
    pub provides_catalog: bool,
    /// Whether the addon provides metadata details.
    #[serde(default)]
    pub provides_meta: bool,
    /// Whether the addon provides playable streams.
    #[serde(default)]
    pub provides_stream: bool,
    /// List of ID prefixes handled by this addon.
    #[serde(default)]
    pub id_prefixes: Vec<String>,
    /// List of media types handled by this addon.
    #[serde(default)]
    pub types: Vec<String>,
}

impl InstalledAddon {
    /// Canonical manifest URL for Cinemeta official metadata addon.
    pub const CINEMETA_MANIFEST: &'static str = "https://v3-cinemeta.strem.io/manifest.json";

    /// Default Cinemeta addon instance configured as core metadata source.
    pub fn cinemeta_default() -> Self {
        Self {
            manifest_url: Self::CINEMETA_MANIFEST.to_string(),
            name: "Cinemeta".to_string(),
            version: Some("3.0.14".to_string()),
            description: Some("Official Catalog and Metadata".to_string()),
            enabled: true,
            provides_catalog: true,
            provides_meta: true,
            provides_stream: false,
            id_prefixes: vec!["tt".to_string()],
            types: vec!["movie".to_string(), "series".to_string()],
        }
    }

    /// Checks if this addon is the built-in core Cinemeta addon.
    pub fn is_core(&self) -> bool {
        self.name.eq_ignore_ascii_case("cinemeta")
            || self.manifest_url.to_lowercase().contains("cinemeta")
    }

    /// Constructs an `InstalledAddon` from an endpoint URL and parsed manifest.
    pub fn from_manifest(manifest_url: String, manifest: &AddonManifest) -> Self {
        Self {
            manifest_url,
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            enabled: true,
            provides_catalog: manifest.provides_catalog(),
            provides_meta: manifest.provides_meta(),
            provides_stream: manifest.provides_stream(),
            id_prefixes: manifest.id_prefixes.clone(),
            types: manifest.types.clone(),
        }
    }
}

/// Target representation of a browsable catalog provided by an addon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddonCatalogTarget {
    /// Human-friendly display label for UI buttons/menus.
    pub label: String,
    /// Name of the providing addon.
    pub addon_name: String,
    /// Manifest URL of the providing addon.
    pub manifest_url: String,
    /// Content type of the catalog ("movie", "series").
    pub r#type: String,
    /// Catalog identifier within the addon.
    pub catalog_id: String,
}

/// Generates curated default catalog targets from active installed addons.
pub fn curated_catalog_presets(addons: &[InstalledAddon]) -> Vec<AddonCatalogTarget> {
    let mut targets = Vec::new();
    let enabled_catalogs: Vec<&InstalledAddon> = addons
        .iter()
        .filter(|a| a.enabled && a.provides_catalog)
        .collect();

    let multi = enabled_catalogs.len() > 1;

    for addon in &enabled_catalogs {
        let suffix = if multi {
            format!(" ({})", addon.name)
        } else {
            String::new()
        };

        let has_movies =
            addon.types.is_empty() || addon.types.iter().any(|t| t.eq_ignore_ascii_case("movie"));
        let has_series =
            addon.types.is_empty() || addon.types.iter().any(|t| t.eq_ignore_ascii_case("series"));

        if has_movies {
            targets.push(AddonCatalogTarget {
                label: format!("Top Movies{suffix}"),
                addon_name: addon.name.clone(),
                manifest_url: addon.manifest_url.clone(),
                r#type: "movie".to_string(),
                catalog_id: "top".to_string(),
            });
        }
        if has_series {
            targets.push(AddonCatalogTarget {
                label: format!("Top Series{suffix}"),
                addon_name: addon.name.clone(),
                manifest_url: addon.manifest_url.clone(),
                r#type: "series".to_string(),
                catalog_id: "top".to_string(),
            });
        }
        if has_movies && addon.is_core() {
            targets.push(AddonCatalogTarget {
                label: format!("Top Rated Movies{suffix}"),
                addon_name: addon.name.clone(),
                manifest_url: addon.manifest_url.clone(),
                r#type: "movie".to_string(),
                catalog_id: "imdbRating".to_string(),
            });
        }
        if has_series && addon.is_core() {
            targets.push(AddonCatalogTarget {
                label: format!("Top Rated Series{suffix}"),
                addon_name: addon.name.clone(),
                manifest_url: addon.manifest_url.clone(),
                r#type: "series".to_string(),
                catalog_id: "imdbRating".to_string(),
            });
        }
        if targets.len() >= 6 {
            break;
        }
    }

    if targets.is_empty() {
        let cinemeta = InstalledAddon::cinemeta_default();
        targets.push(AddonCatalogTarget {
            label: "Top Movies".to_string(),
            addon_name: cinemeta.name.clone(),
            manifest_url: cinemeta.manifest_url.clone(),
            r#type: "movie".to_string(),
            catalog_id: "top".to_string(),
        });
        targets.push(AddonCatalogTarget {
            label: "Top Series".to_string(),
            addon_name: cinemeta.name.clone(),
            manifest_url: cinemeta.manifest_url.clone(),
            r#type: "series".to_string(),
            catalog_id: "top".to_string(),
        });
        targets.push(AddonCatalogTarget {
            label: "Top Rated Movies".to_string(),
            addon_name: cinemeta.name.clone(),
            manifest_url: cinemeta.manifest_url.clone(),
            r#type: "movie".to_string(),
            catalog_id: "imdbRating".to_string(),
        });
        targets.push(AddonCatalogTarget {
            label: "Top Rated Series".to_string(),
            addon_name: cinemeta.name.clone(),
            manifest_url: cinemeta.manifest_url.clone(),
            r#type: "series".to_string(),
            catalog_id: "imdbRating".to_string(),
        });
    }

    targets.truncate(6);
    targets
}
