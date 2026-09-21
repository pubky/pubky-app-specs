use crate::constants::social_path;
use crate::traits::{Root, ValidationCtx, ValidationError};
use crate::{
    common::{
        ascii_fold, check_extra, code_point_len, frozen_trim, timestamp, validate_safe_json_int,
    },
    limits::VALIDATION_LIMITS,
    models::tag::{sanitize_tag_label, validate_tag_label},
    traits::{HasIdPath, HashId, Validatable},
    PubkySocialPostKind,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::str::FromStr;

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Enum representing the reach of the feed.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[non_exhaustive]
pub enum PubkySocialFeedReach {
    Following,
    Followers,
    Friends,
    All,
    Wot,
    Me,
    #[serde(other)]
    Unknown,
}

impl PubkySocialFeedReach {
    /// `false` only for the `Unknown` catch-all a newer writer's value lands in.
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown)
    }

    /// The frozen wire spelling. One function, so the id input and every other text
    /// rendering of a value can never disagree.
    pub fn wire_name(&self) -> &'static str {
        match self {
            Self::Following => "following",
            Self::Followers => "followers",
            Self::Friends => "friends",
            Self::All => "all",
            Self::Wot => "wot",
            Self::Me => "me",
            Self::Unknown => "unknown",
        }
    }
}

/// Enum representing the layout of the feed.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[non_exhaustive]
pub enum PubkySocialFeedLayout {
    Columns,
    Wide,
    Visual,
    List,
    #[serde(other)]
    Unknown,
}

impl PubkySocialFeedLayout {
    /// `false` only for the `Unknown` catch-all a newer writer's value lands in.
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown)
    }

    /// The frozen wire spelling, see [`PubkySocialFeedReach::wire_name`].
    pub fn wire_name(&self) -> &'static str {
        match self {
            Self::Columns => "columns",
            Self::Wide => "wide",
            Self::Visual => "visual",
            Self::List => "list",
            Self::Unknown => "unknown",
        }
    }
}

/// Enum representing the sort order of the feed.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[non_exhaustive]
pub enum PubkySocialFeedSort {
    Recent,
    Popularity,
    #[serde(other)]
    Unknown,
}

impl PubkySocialFeedSort {
    /// `false` only for the `Unknown` catch-all a newer writer's value lands in.
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown)
    }

    /// The frozen wire spelling, see [`PubkySocialFeedReach::wire_name`].
    pub fn wire_name(&self) -> &'static str {
        match self {
            Self::Recent => "recent",
            Self::Popularity => "popularity",
            Self::Unknown => "unknown",
        }
    }
}

/// Configuration object for the feed. The whole of a feed's identity: two feeds with the
/// same config are the same feed, whatever they are named.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialFeedConfig {
    /// Canonical as stored: folded labels, deduplicated, sorted by code point, never empty.
    /// `None` is "no tag filter".
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub tags: Option<Vec<String>>,
    /// A domain filter, same rules as `tags`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub domain_tags: Option<Vec<String>>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub reach: PubkySocialFeedReach,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub layout: PubkySocialFeedLayout,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub sort: PubkySocialFeedSort,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub content: Option<PubkySocialPostKind>,
    /// Unknown members, preserved on rewrite; see the module contract in `models/mod.rs`.
    /// Outside the id input, which reads named fields only.
    #[serde(flatten)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl PubkySocialFeedConfig {
    /// The one builder. It canonicalizes both tag lists, so one filter has one spelling and
    /// one id; an empty result is stored as `None`, "no filter".
    pub fn new(
        tags: Option<Vec<String>>,
        domain_tags: Option<Vec<String>>,
        reach: PubkySocialFeedReach,
        layout: PubkySocialFeedLayout,
        sort: PubkySocialFeedSort,
        content: Option<PubkySocialPostKind>,
    ) -> Self {
        Self {
            tags: canonical_filter(tags),
            domain_tags: canonical_filter(domain_tags),
            reach,
            layout,
            sort,
            content,
            extra: Default::default(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialFeedConfig {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for `tags`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn tags(&self) -> Option<Vec<String>> {
        self.tags.clone()
    }

    /// Getter for `domain_tags`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn domain_tags(&self) -> Option<Vec<String>> {
        self.domain_tags.clone()
    }

    /// Getter for `name`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn reach(&self) -> PubkySocialFeedReach {
        self.reach.clone()
    }

    /// Getter for `layout`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn layout(&self) -> PubkySocialFeedLayout {
        self.layout.clone()
    }

    /// Getter for `sort`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn sort(&self) -> PubkySocialFeedSort {
        self.sort.clone()
    }

    /// Getter for `content`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn content(&self) -> Option<PubkySocialPostKind> {
        self.content.clone()
    }
}

/// Folds every label, drops what folds to nothing, deduplicates and sorts by code point
/// (`str` order is UTF-8 byte order is code point order). Builders call this; a stored list
/// is already its own fixed point, so the id input joins it verbatim.
fn canonical_tag_list(tags: Vec<String>) -> Vec<String> {
    tags.into_iter()
        .map(|tag| sanitize_tag_label(&tag))
        .filter(|tag| !tag.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// A list that canonicalizes to nothing is not a filter, so it is stored as `None`.
fn canonical_filter(tags: Option<Vec<String>>) -> Option<Vec<String>> {
    tags.map(canonical_tag_list).filter(|list| !list.is_empty())
}

/// Builders fold an icon name the way they fold a tag label. The engine `trim`/`to_lowercase`
/// pair this replaces follows a Unicode table version and cannot be pinned across engines.
fn sanitize_feed_icon(icon: Option<String>) -> Option<String> {
    icon.map(|icon| ascii_fold(frozen_trim(&icon)))
}

/// Only the shape of the name is validated, not whether the icon exists: the icon set is
/// curated by the client.
///
/// `None` is accepted for feeds created before the field existed; new feeds always carry
/// one, since [`PubkySocialFeed::new`] requires it.
fn validate_feed_icon(icon: &Option<String>) -> Result<(), String> {
    let Some(icon) = icon else {
        return Ok(());
    };

    let icon_len = code_point_len(icon);
    if !(1..=VALIDATION_LIMITS.feed_icon_max_length).contains(&icon_len) {
        return Err(format!(
            "Validation Error: Feed icon '{}' must be 1 to {} characters",
            icon, VALIDATION_LIMITS.feed_icon_max_length
        ));
    }

    if let Some(c) = icon
        .chars()
        .find(|c| !c.is_ascii_lowercase() && !c.is_ascii_digit() && *c != '-')
    {
        return Err(format!(
            "Validation Error: Feed icon '{}' contains invalid character: {}",
            icon, c
        ));
    }

    Ok(())
}

/// A stored list must be exactly what the builder would have written: non-empty, within the
/// count cap, every label its own fold, and strictly increasing, which is deduplicated and
/// sorted in one check. Ingest rejects anything else instead of repairing it, so one filter
/// keeps one id and a reader never disagrees with the bytes.
fn validate_tag_list(tags: &Option<Vec<String>>, field_name: &str) -> Result<(), String> {
    let Some(tags) = tags else {
        return Ok(());
    };

    if tags.is_empty() {
        return Err(format!(
            "Validation Error: Feed config {field_name} cannot be an empty list, omit it for no filter"
        ));
    }

    if tags.len() > VALIDATION_LIMITS.feed_tags_max_count {
        return Err(format!(
            "Validation Error: Feed config cannot have more than {} {}",
            VALIDATION_LIMITS.feed_tags_max_count, field_name
        ));
    }

    for tag in tags {
        if *tag != sanitize_tag_label(tag) {
            return Err(format!(
                "Validation Error: Tag '{tag}' must be stored folded (trimmed, ASCII lowercase)"
            ));
        }
        validate_tag_label(tag)?;
    }

    if !tags.windows(2).all(|w| w[0] < w[1]) {
        return Err(format!(
            "Validation Error: Feed config {field_name} must be stored deduplicated and sorted by code point"
        ));
    }

    Ok(())
}

impl Validatable for PubkySocialFeedConfig {
    // No sanitize: a stored tag list is canonical as written. Folding it here would rewrite
    // the very bytes the id is derived from, and the reader would disagree with the writer.

    fn validate_fields(
        &self,
        _id: Option<&str>,
        _ctx: &ValidationCtx,
    ) -> Result<(), ValidationError> {
        // reach, layout and sort define the feed, so an unknown value rejects it.
        // An unknown content filter only degrades to "no filter", so it passes.
        if !self.reach.is_known() {
            return Err("Validation Error: feed reach is unknown".into());
        }
        if !self.layout.is_known() {
            return Err("Validation Error: feed layout is unknown".into());
        }
        if !self.sort.is_known() {
            return Err("Validation Error: feed sort is unknown".into());
        }
        check_extra(
            &self.extra,
            &["tags", "domain_tags", "reach", "layout", "sort", "content"],
        )?;
        validate_tag_list(&self.tags, "tags")?;
        validate_tag_list(&self.domain_tags, "domain_tags")?;

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkySocialFeedConfig {}

/// Represents a feed configuration.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkySocialFeed {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub feed: PubkySocialFeedConfig,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub name: String,
    /// Lucide icon name, e.g. `"bitcoin"`. Required on new feeds, but optional
    /// on the wire: feeds created before this field existed have none, and
    /// clients render their default icon for those. Not part of the `feed_id`,
    /// so the icon can change without recreating the feed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub icon: Option<String>,
    pub created_at: i64,
    /// Unknown members, preserved on rewrite; see the module contract in `models/mod.rs`.
    #[serde(flatten)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl PubkySocialFeed {
    /// Creates a new `PubkySocialFeed` instance and sanitizes it. Pass a config built by
    /// [`PubkySocialFeedConfig::new`], which is what canonicalizes the tag lists.
    pub fn new(feed: PubkySocialFeedConfig, name: String, icon: String) -> Self {
        let created_at = timestamp();
        Self {
            feed,
            name,
            icon: Some(icon),
            created_at,
            extra: Default::default(),
        }
        .sanitize()
    }

    /// "/{root}/social/v1/feeds/{id}.json". Feeds are private by default; the public
    /// spelling is the published copy.
    pub fn create_path_in(root: Root, id: &str) -> String {
        social_path(root, &format!("{}{id}.json", Self::PATH_SEGMENT))
    }
}

/// Both addresses of one feed.
///
/// A feed lives at `private`. To PUBLISH it, PUT the same bytes at `public`; to unpublish,
/// DELETE `public`. Nothing else is involved: a feed config carries no root-bearing URIs, so
/// publishing is a plain byte copy, and because the id is derived from the config alone the
/// two copies can never disagree about what the feed filters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedPaths {
    /// Where the builder writes.
    pub private: String,
    /// The published copy, absent until the user publishes.
    pub public: String,
}

/// The private and published addresses of the feed with this id, see [`FeedPaths`].
pub fn feed_paths(id: &str) -> FeedPaths {
    FeedPaths {
        private: PubkySocialFeed::create_path_in(Root::Priv, id),
        public: PubkySocialFeed::create_path_in(Root::Pub, id),
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkySocialFeed {
    /// Serialize to JSON for WASM.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for `feed`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn feed(&self) -> PubkySocialFeedConfig {
        self.feed.clone()
    }

    /// Getter for `name`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Getter for `icon`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn icon(&self) -> Option<String> {
        self.icon.clone()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkySocialFeed {}

impl HashId for PubkySocialFeed {
    /// "{reach}:{layout}:{sort}:{content or '-'}:{tags or '-'}:{domain_tags or '-'}", frozen
    /// wire names, each list joined with ',' exactly as stored (canonical: folded,
    /// deduplicated, sorted by code point). Injective because ':' and ',' are both in
    /// `tag_invalid_chars` and the segment count is fixed. A stored list is never empty, so
    /// '-' means only "no filter". `name`, `icon`, `created_at` and `extra` stay outside: a
    /// feed is what it filters, not how it looks.
    fn get_id_data(&self) -> String {
        let list = |l: &Option<Vec<String>>| {
            l.as_ref()
                .map_or_else(|| "-".to_string(), |list| list.join(","))
        };
        format!(
            "{}:{}:{}:{}:{}:{}",
            self.feed.reach.wire_name(),
            self.feed.layout.wire_name(),
            self.feed.sort.wire_name(),
            self.feed.content.as_ref().map_or("-", |c| c.wire_name()),
            list(&self.feed.tags),
            list(&self.feed.domain_tags),
        )
    }
}

impl HasIdPath for PubkySocialFeed {
    const ROOT: Root = Root::Priv;
    const PATH_SEGMENT: &'static str = "feeds/";

    fn create_path(id: &str) -> String {
        Self::create_path_in(Self::ROOT, id)
    }
}

impl Validatable for PubkySocialFeed {
    fn validate_fields(
        &self,
        id: Option<&str>,
        ctx: &ValidationCtx,
    ) -> Result<(), ValidationError> {
        // Config first, so an unrecognized value is reported as such and not as an id mismatch
        self.feed.validate(None, ctx)?;
        check_extra(&self.extra, &["feed", "name", "icon", "created_at"])?;

        if frozen_trim(&self.name).is_empty() {
            return Err("Validation Error: Feed name cannot be empty".into());
        }
        if code_point_len(&self.name) > VALIDATION_LIMITS.feed_name_max_length {
            return Err(format!(
                "Validation Error: Feed name exceeds maximum length of {} characters",
                VALIDATION_LIMITS.feed_name_max_length
            ));
        }

        validate_feed_icon(&self.icon)?;
        validate_safe_json_int(self.created_at)?;

        if let Some(id) = id {
            // The id is a write-side guarantee. A reader that does not know the content
            // filter cannot rebuild the writer's string around it, so it takes the id as
            // named; reach, layout and sort are rejected above, before ever reaching here.
            if self.feed.content.as_ref().is_none_or(|c| c.is_known()) {
                self.validate_id(id)?;
            }
        }

        Ok(())
    }

    fn sanitize(self) -> Self {
        PubkySocialFeed {
            name: frozen_trim(&self.name).to_string(),
            icon: sanitize_feed_icon(self.icon),
            ..self
        }
    }
}

impl FromStr for PubkySocialFeedReach {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "following" => Ok(PubkySocialFeedReach::Following),
            "followers" => Ok(PubkySocialFeedReach::Followers),
            "friends" => Ok(PubkySocialFeedReach::Friends),
            "all" => Ok(PubkySocialFeedReach::All),
            "wot" => Ok(PubkySocialFeedReach::Wot),
            "me" => Ok(PubkySocialFeedReach::Me),
            _ => Err(format!("Invalid feed reach: {}", s)),
        }
    }
}

impl FromStr for PubkySocialFeedLayout {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "columns" => Ok(PubkySocialFeedLayout::Columns),
            "wide" => Ok(PubkySocialFeedLayout::Wide),
            "visual" => Ok(PubkySocialFeedLayout::Visual),
            "list" => Ok(PubkySocialFeedLayout::List),
            _ => Err(format!("Invalid feed layout: {}", s)),
        }
    }
}

impl FromStr for PubkySocialFeedSort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "recent" => Ok(PubkySocialFeedSort::Recent),
            "popularity" => Ok(PubkySocialFeedSort::Popularity),
            _ => Err(format!("Invalid feed sort: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::PUB_CTX;
    use crate::{limits::VALIDATION_LIMITS, traits::Validatable};

    const PRIV_CTX: ValidationCtx = ValidationCtx { root: Root::Priv };
    const PK: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    use PubkySocialFeedLayout as L;
    use PubkySocialFeedReach as R;
    use PubkySocialFeedSort as S;

    /// A config exactly as stored, bypassing the canonicalizing builder.
    fn stored(
        tags: Option<Vec<&str>>,
        domain_tags: Option<Vec<&str>>,
        reach: R,
        layout: L,
        sort: S,
        content: Option<PubkySocialPostKind>,
    ) -> PubkySocialFeedConfig {
        let own = |l: Option<Vec<&str>>| l.map(|l| l.into_iter().map(String::from).collect());
        PubkySocialFeedConfig {
            tags: own(tags),
            domain_tags: own(domain_tags),
            reach,
            layout,
            sort,
            content,
            extra: Default::default(),
        }
    }

    fn feed(config: PubkySocialFeedConfig) -> PubkySocialFeed {
        PubkySocialFeed::new(config, "Test Feed".into(), "rss".into())
    }

    fn validate(f: &PubkySocialFeed) -> Result<(), String> {
        f.validate(Some(&f.create_id()), &PRIV_CTX)
    }

    #[test]
    fn test_id_input_is_the_pinned_string() {
        // Six fixed segments, wire names, the stored lists joined verbatim
        let legacy = feed(stored(None, None, R::All, L::List, S::Popularity, None));
        assert_eq!(legacy.get_id_data(), "all:list:popularity:-:-:-");
        // blake3("all:list:popularity:-:-:-")[..16] in Crockford
        assert_eq!(legacy.create_id(), "X73G7QREDQ81D7K49GCZ89SEHC");

        let fixture = feed(stored(
            Some(vec!["rust"]),
            Some(vec!["dev"]),
            R::Wot,
            L::Columns,
            S::Recent,
            Some(PubkySocialPostKind::Note),
        ));
        assert_eq!(fixture.get_id_data(), "wot:columns:recent:note:rust:dev");
        // blake3("wot:columns:recent:note:rust:dev")[..16] in Crockford
        assert_eq!(fixture.create_id(), "2CPRX2C4D6FNNS9ZRM50X99288");

        // The builder sorts, so the two spellings of one filter are one feed
        let two = feed(PubkySocialFeedConfig::new(
            Some(vec!["b".into(), "a".into()]),
            None,
            R::All,
            L::Columns,
            S::Recent,
            None,
        ));
        assert_eq!(two.get_id_data(), "all:columns:recent:-:a,b:-");
        // blake3("all:columns:recent:-:a,b:-")[..16] in Crockford
        assert_eq!(two.create_id(), "H0GZXBEPAQAA65145FNQP7H50R");
    }

    #[test]
    fn test_id_covers_the_config_and_nothing_else() {
        let config = || {
            PubkySocialFeedConfig::new(
                Some(vec!["rust".into()]),
                None,
                R::All,
                L::Columns,
                S::Recent,
                None,
            )
        };
        let plain = feed(config());
        let mut dressed = PubkySocialFeed::new(config(), "Another Name".into(), "bitcoin".into());
        dressed.created_at = 1_700_000_000_000_000;
        dressed.extra.insert("ext".into(), 1.into());
        assert_eq!(plain.create_id(), dressed.create_id());

        // every segment moves the id
        let base = plain.create_id();
        for edited in [
            PubkySocialFeedConfig::new(
                Some(vec!["rust".into()]),
                None,
                R::Following,
                L::Columns,
                S::Recent,
                None,
            ),
            PubkySocialFeedConfig::new(
                Some(vec!["rust".into()]),
                None,
                R::All,
                L::List,
                S::Recent,
                None,
            ),
            PubkySocialFeedConfig::new(
                Some(vec!["rust".into()]),
                None,
                R::All,
                L::Columns,
                S::Popularity,
                None,
            ),
            PubkySocialFeedConfig::new(
                Some(vec!["rust".into()]),
                None,
                R::All,
                L::Columns,
                S::Recent,
                Some(PubkySocialPostKind::Note),
            ),
            PubkySocialFeedConfig::new(
                Some(vec!["rust".into(), "bitcoin".into()]),
                None,
                R::All,
                L::Columns,
                S::Recent,
                None,
            ),
            PubkySocialFeedConfig::new(
                Some(vec!["rust".into()]),
                Some(vec!["dev".into()]),
                R::All,
                L::Columns,
                S::Recent,
                None,
            ),
        ] {
            assert_ne!(feed(edited).create_id(), base);
        }
    }

    #[test]
    fn test_builder_sorts_by_code_point_not_utf16_unit() {
        // U+1D51E sorts after U+FB00 by code point; a JS default sort(), which compares
        // UTF-16 units, would put the surrogate pair first and fork the id.
        let config = PubkySocialFeedConfig::new(
            Some(vec!["\u{1D51E}".into(), "\u{FB00}".into()]),
            None,
            R::All,
            L::Columns,
            S::Recent,
            None,
        );
        assert_eq!(
            config.tags,
            Some(vec!["\u{FB00}".to_string(), "\u{1D51E}".to_string()])
        );
        assert!(validate(&feed(config)).is_ok());
    }

    #[test]
    fn test_builder_folds_dedups_and_drops_empties() {
        let config = PubkySocialFeedConfig::new(
            Some(vec![
                "  RUST ".into(),
                "rust".into(),
                "  ".into(),
                "Bitcoin".into(),
            ]),
            Some(vec!["  ".into()]),
            R::All,
            L::Columns,
            S::Recent,
            None,
        );
        assert_eq!(
            config.tags,
            Some(vec!["bitcoin".to_string(), "rust".to_string()])
        );
        // a list that canonicalizes to nothing is "no filter", never Some([])
        assert_eq!(config.domain_tags, None);
        assert!(validate(&feed(config)).is_ok());
    }

    #[test]
    fn test_a_stored_list_is_canonical_or_it_is_rejected() {
        for (list, expected) in [
            (Some(vec![]), "cannot be an empty list"),
            (Some(vec!["Rust"]), "stored folded"),
            (Some(vec![" rust"]), "stored folded"),
            (Some(vec!["b", "a"]), "sorted by code point"),
            (Some(vec!["a", "a"]), "sorted by code point"),
            (Some(vec!["a:b"]), "invalid character"),
            (Some(vec!["a,b"]), "invalid character"),
            (Some(vec!["a b"]), "whitespace"),
            (
                Some(vec!["t1", "t2", "t3", "t4", "t5", "t6"]),
                "more than 5",
            ),
        ] {
            for field in ["tags", "domain_tags"] {
                let (tags, domain_tags) = match field {
                    "tags" => (list.clone(), None),
                    _ => (None, list.clone()),
                };
                let f = feed(stored(
                    tags,
                    domain_tags,
                    R::All,
                    L::Columns,
                    S::Recent,
                    None,
                ));
                let e = f.validate(Some(&f.create_id()), &PRIV_CTX).unwrap_err();
                assert!(e.contains(expected), "{field} {list:?}: {e}");
            }
        }
        // the cap itself accepts
        let f = feed(stored(
            Some(vec!["t1", "t2", "t3", "t4", "t5"]),
            None,
            R::All,
            L::Columns,
            S::Recent,
            None,
        ));
        assert!(validate(&f).is_ok());
        assert_eq!(VALIDATION_LIMITS.feed_tags_max_count, 5);
    }

    #[test]
    fn test_unknown_enums_and_the_write_side_id() {
        for (config, field) in [
            (
                stored(None, None, R::Unknown, L::List, S::Recent, None),
                "reach",
            ),
            (
                stored(None, None, R::All, L::Unknown, S::Recent, None),
                "layout",
            ),
            (
                stored(None, None, R::All, L::List, S::Unknown, None),
                "sort",
            ),
        ] {
            let e = feed(config)
                .validate(Some("8Z8CWH8NVYQY39ZEBFGKQWWEKG"), &PRIV_CTX)
                .unwrap_err();
            assert!(e.contains(field) && e.contains("unknown"), "{e}");
        }
        // An unknown content filter leaves the id unrebuildable, so the id is taken as named
        let f = feed(stored(
            None,
            None,
            R::All,
            L::List,
            S::Recent,
            Some(PubkySocialPostKind::Unknown),
        ));
        assert!(f
            .validate(Some("8Z8CWH8NVYQY39ZEBFGKQWWEKG"), &PRIV_CTX)
            .is_ok());
        // a known content filter is still checked
        let f = feed(stored(
            None,
            None,
            R::All,
            L::List,
            S::Recent,
            Some(PubkySocialPostKind::Note),
        ));
        let e = f
            .validate(Some("8Z8CWH8NVYQY39ZEBFGKQWWEKG"), &PRIV_CTX)
            .unwrap_err();
        assert!(e.contains("Invalid ID"), "{e}");
    }

    #[test]
    fn test_name_rules() {
        let max = VALIDATION_LIMITS.feed_name_max_length;
        assert_eq!(max, 100);
        for (name, ok) in [("x", true), ("  Rust Bitcoiners", true), ("   ", false)] {
            let config =
                PubkySocialFeedConfig::new(None, None, R::All, L::Columns, S::Recent, None);
            let f = PubkySocialFeed::new(config, name.into(), "rss".into());
            assert_eq!(validate(&f).is_ok(), ok, "{name:?}");
        }
        // the builder trims, so the cap counts code points of the trimmed name
        let config = PubkySocialFeedConfig::new(None, None, R::All, L::Columns, S::Recent, None);
        let f = PubkySocialFeed::new(config.clone(), "🦀".repeat(max), "rss".into());
        assert_eq!(f.name.chars().count(), max);
        assert!(validate(&f).is_ok());
        let f = PubkySocialFeed::new(config, "🦀".repeat(max + 1), "rss".into());
        let e = validate(&f).unwrap_err();
        assert!(e.contains("exceeds maximum length"), "{e}");
    }

    #[test]
    fn test_icon_rules() {
        let config = || PubkySocialFeedConfig::new(None, None, R::All, L::Columns, S::Recent, None);
        // the builder folds with the frozen ops
        let f = PubkySocialFeed::new(config(), "Mixed".into(), "\u{3000}Code-2 ".into());
        assert_eq!(f.icon, Some("code-2".into()));
        assert!(validate(&f).is_ok());
        // a stored icon that is not its own fold has a character outside [a-z0-9-]
        let mut stored_icon = f.clone();
        stored_icon.icon = Some("Code".into());
        let e = validate(&stored_icon).unwrap_err();
        assert!(e.contains("invalid character: C"), "{e}");

        for bad in ["bad icon", "bad_icon", "", &"a".repeat(51)] {
            let mut f = f.clone();
            f.icon = Some(bad.into());
            assert!(validate(&f).is_err(), "{bad:?}");
        }
        // shape only: an icon no client knows is still a valid name
        let mut f = f.clone();
        f.icon = Some("no-such-icon-42".into());
        assert!(validate(&f).is_ok());
        // and a feed written before the field existed has none
        f.icon = None;
        assert!(validate(&f).is_ok());
        assert_eq!(VALIDATION_LIMITS.feed_icon_max_length, 50);
    }

    #[test]
    fn test_created_at_is_json_safe() {
        let mut f = feed(stored(None, None, R::All, L::List, S::Recent, None));
        f.created_at = i64::MAX;
        assert!(validate(&f).unwrap_err().contains("JSON-safe"));
    }

    #[test]
    fn test_feeds_are_private_and_publishing_is_a_byte_copy() {
        let f = feed(stored(None, None, R::All, L::List, S::Recent, None));
        let id = f.create_id();
        assert_eq!(
            PubkySocialFeed::create_path(&id),
            format!("/priv/social/v1/feeds/{id}.json")
        );
        let paths = feed_paths(&id);
        assert_eq!(paths.private, PubkySocialFeed::create_path(&id));
        assert_eq!(paths.public, format!("/pub/social/v1/feeds/{id}.json"));

        // the builder's URI is the private one, and both roots parse back to the same feed
        let uri = crate::feed_uri_builder(PK.into(), id.clone());
        assert_eq!(uri, format!("pubky://{PK}{}", paths.private));
        for (path, visibility) in [
            (&paths.private, crate::Visibility::Private),
            (&paths.public, crate::Visibility::Public),
        ] {
            let uri = format!("pubky://{PK}{path}");
            let parsed = crate::ParsedUri::try_from(uri.as_str()).unwrap();
            assert_eq!(parsed.visibility, visibility);
            assert_eq!(parsed.resource, crate::Resource::Feed(id.clone()));
            assert_eq!(parsed.try_to_uri_str().unwrap(), uri);
        }
    }

    #[test]
    fn test_try_from_validates_and_preserves() {
        let blob = br#"{"feed":{"tags":["rust"],"reach":"all","layout":"columns","sort":"recent","content":null,"ext":{"pinned":true}},"name":"Rust","icon":"code","created_at":1700000000,"ext":{"badge":1}}"#;
        let id = feed(stored(
            Some(vec!["rust"]),
            None,
            R::All,
            L::Columns,
            S::Recent,
            None,
        ))
        .create_id();
        let f = <PubkySocialFeed as Validatable>::try_from(blob, &id, &PRIV_CTX).unwrap();
        assert_eq!(f.extra["ext"]["badge"], 1);
        assert_eq!(f.feed.extra["ext"]["pinned"], true);
        let back = serde_json::to_string(&f).unwrap();
        assert!(back.contains(r#""ext":{"badge":1}"#), "{back}");
        assert!(back.contains(r#""ext":{"pinned":true}"#), "{back}");

        // an unknown member never shadows a known field, on either object
        let mut shadow = f.clone();
        shadow.extra.insert("name".into(), "x".into());
        assert!(validate(&shadow).unwrap_err().contains("shadow"));
        let mut shadow = f.clone();
        shadow.feed.extra.insert("reach".into(), "x".into());
        assert!(validate(&shadow).unwrap_err().contains("shadow"));
    }

    #[test]
    fn test_ingest_by_uri_under_both_roots() {
        let blob = br#"{"feed":{"tags":null,"reach":"all","layout":"list","sort":"popularity","content":null},"name":"All","created_at":1700000000}"#;
        let id = feed(stored(None, None, R::All, L::List, S::Popularity, None)).create_id();
        for path in [feed_paths(&id).private, feed_paths(&id).public] {
            let uri = format!("pubky://{PK}{path}");
            assert!(
                crate::PubkySocialObject::from_uri(&uri, blob).is_ok(),
                "{uri}"
            );
        }
    }

    #[test]
    fn test_in_memory_size_cap() {
        let mut f = feed(stored(None, None, R::All, L::List, S::Recent, None));
        f.extra
            .insert("ext".into(), "a".repeat(PubkySocialFeed::MAX_BYTES).into());
        assert!(f.validate_fields(None, &PRIV_CTX).is_ok());
        assert!(f.validate(None, &PRIV_CTX).unwrap_err().contains("exceeds"));
    }

    #[test]
    fn test_wire_names_are_the_serde_names() {
        fn serde_name<T: serde::Serialize>(v: &T) -> String {
            serde_json::to_value(v)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        }
        for r in [
            R::Following,
            R::Followers,
            R::Friends,
            R::All,
            R::Wot,
            R::Me,
            R::Unknown,
        ] {
            assert_eq!(r.wire_name(), serde_name(&r));
        }
        for l in [L::Columns, L::Wide, L::Visual, L::List, L::Unknown] {
            assert_eq!(l.wire_name(), serde_name(&l));
        }
        for s in [S::Recent, S::Popularity, S::Unknown] {
            assert_eq!(s.wire_name(), serde_name(&s));
        }
        for k in [
            PubkySocialPostKind::Note,
            PubkySocialPostKind::Article,
            PubkySocialPostKind::Image,
            PubkySocialPostKind::Video,
            PubkySocialPostKind::Link,
            PubkySocialPostKind::File,
            PubkySocialPostKind::Collection,
            PubkySocialPostKind::Unknown,
        ] {
            assert_eq!(k.wire_name(), serde_name(&k));
        }
    }

    #[test]
    fn test_validate_with_the_public_ctx_too() {
        // A feed carries no reference-tier field, so the destination root changes nothing
        let f = feed(stored(None, None, R::All, L::List, S::Recent, None));
        assert_eq!(
            f.validate(Some(&f.create_id()), &PUB_CTX),
            f.validate(Some(&f.create_id()), &PRIV_CTX)
        );
    }

    #[test]
    fn test_feed_reach_from_str() {
        // Valid cases
        assert_eq!(
            "following".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::Following
        );
        assert_eq!(
            "followers".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::Followers
        );
        assert_eq!(
            "friends".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::Friends
        );
        assert_eq!(
            "all".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::All
        );
        assert_eq!(
            "wot".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::Wot
        );
        assert_eq!(
            "me".parse::<PubkySocialFeedReach>().unwrap(),
            PubkySocialFeedReach::Me
        );

        // Invalid case
        assert!("invalid".parse::<PubkySocialFeedReach>().is_err());
    }

    #[test]
    fn test_feed_layout_from_str() {
        // Valid cases
        assert_eq!(
            "columns".parse::<PubkySocialFeedLayout>().unwrap(),
            PubkySocialFeedLayout::Columns
        );
        assert_eq!(
            "wide".parse::<PubkySocialFeedLayout>().unwrap(),
            PubkySocialFeedLayout::Wide
        );
        assert_eq!(
            "visual".parse::<PubkySocialFeedLayout>().unwrap(),
            PubkySocialFeedLayout::Visual
        );
        assert_eq!(
            "list".parse::<PubkySocialFeedLayout>().unwrap(),
            PubkySocialFeedLayout::List
        );

        // Invalid case
        assert!("invalid".parse::<PubkySocialFeedLayout>().is_err());
    }

    #[test]
    fn test_feed_sort_from_str() {
        // Valid cases
        assert_eq!(
            "recent".parse::<PubkySocialFeedSort>().unwrap(),
            PubkySocialFeedSort::Recent
        );
        assert_eq!(
            "popularity".parse::<PubkySocialFeedSort>().unwrap(),
            PubkySocialFeedSort::Popularity
        );

        // Invalid case
        assert!("invalid".parse::<PubkySocialFeedSort>().is_err());
    }

    #[test]
    fn test_new_keeps_the_config_and_times_the_feed() {
        let config = PubkySocialFeedConfig::new(
            Some(vec!["bitcoin".into(), "rust".into()]),
            None,
            R::Following,
            L::Columns,
            S::Recent,
            Some(PubkySocialPostKind::Image),
        );
        let f = PubkySocialFeed::new(config.clone(), "Rust Bitcoiners".into(), "bitcoin".into());
        assert_eq!(f.feed, config);
        assert_eq!(f.name, "Rust Bitcoiners");
        assert_eq!(f.icon, Some("bitcoin".to_string()));
        let now = timestamp();
        assert!(f.created_at <= now && f.created_at >= now - 1_000_000);
    }
}
