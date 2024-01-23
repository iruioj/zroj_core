//! markdown syntax tree: [mdast][].
//!
//! [mdast]: https://github.com/syntax-tree/mdast

use serde::{Deserialize, Serialize};
use serde_ts_typing::TsType;

/// Explicitness of a reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub enum ReferenceKind {
    /// The reference is implicit, its identifier inferred from its content.
    #[serde(rename = "shortcut")]
    Shortcut,
    /// The reference is explicit, its identifier inferred from its content.
    #[serde(rename = "collapsed")]
    Collapsed,
    /// The reference is explicit, its identifier explicitly set.
    #[serde(rename = "full")]
    Full,
}

/// GFM: alignment of phrasing content.
///
/// Used to align the contents of table cells within a table.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub enum AlignKind {
    /// Left alignment.
    ///
    /// See the `left` value of the `text-align` CSS property.
    ///
    /// ```markdown
    ///   | | aaa |
    /// > | | :-- |
    ///       ^^^
    /// ```
    #[serde(rename = "left")]
    Left,
    /// Right alignment.
    ///
    /// See the `right` value of the `text-align` CSS property.
    ///
    /// ```markdown
    ///   | | aaa |
    /// > | | --: |
    ///       ^^^
    /// ```
    #[serde(rename = "right")]
    Right,
    /// Center alignment.
    ///
    /// See the `center` value of the `text-align` CSS property.
    ///
    /// ```markdown
    ///   | | aaa |
    /// > | | :-: |
    ///       ^^^
    /// ```
    #[serde(rename = "center")]
    Center,
    /// No alignment.
    ///
    /// Phrasing content is aligned as defined by the host environment.
    ///
    /// ```markdown
    ///   | | aaa |
    /// > | | --- |
    ///       ^^^
    /// ```
    #[serde(rename = "none")]
    None,
}

/// Nodes.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, TsType)]
#[serde(tag = "type")]
pub enum Node {
    // Document:
    /// Root.
    #[serde(rename = "root")]
    Root(Root),

    // Container:
    /// Block quote.
    #[serde(rename = "blockquote")]
    BlockQuote(BlockQuote),
    /// Footnote definition.
    #[serde(rename = "footnoteDefinition")]
    FootnoteDefinition(FootnoteDefinition),
    /// List.
    #[serde(rename = "list")]
    List(List),

    // Frontmatter:
    /// Toml.
    #[serde(rename = "toml")]
    Toml(Toml),
    /// Yaml.
    #[serde(rename = "yaml")]
    Yaml(Yaml),

    // Phrasing:
    /// Break.
    #[serde(rename = "break")]
    Break(Break),
    /// Code (phrasing).
    #[serde(rename = "inlineCode")]
    InlineCode(InlineCode),
    /// Math (phrasing).
    #[serde(rename = "inlineMath")]
    InlineMath(InlineMath),
    /// Delete.
    #[serde(rename = "delete")]
    Delete(Delete),
    /// Emphasis.
    #[serde(rename = "emphasis")]
    Emphasis(Emphasis),
    /// Footnote reference.
    #[serde(rename = "footnoteReference")]
    FootnoteReference(FootnoteReference),
    /// Html (phrasing).
    #[serde(rename = "html")]
    Html(Html),
    /// Image.
    #[serde(rename = "image")]
    Image(Image),
    /// Image reference.
    #[serde(rename = "imageReference")]
    ImageReference(ImageReference),
    /// Link.
    #[serde(rename = "link")]
    Link(Link),
    /// Link reference.
    #[serde(rename = "linkReference")]
    LinkReference(LinkReference),
    /// Strong
    #[serde(rename = "strong")]
    Strong(Strong),
    /// Text.
    #[serde(rename = "text")]
    Text(Text),

    // Flow:
    /// Code (flow).
    #[serde(rename = "code")]
    Code(Code),
    /// Math (flow).
    #[serde(rename = "math")]
    Math(Math),
    /// Heading.
    #[serde(rename = "heading")]
    Heading(Heading),
    /// Html (flow).
    // Html(Html),
    /// Table.
    #[serde(rename = "table")]
    Table(Table),
    /// Thematic break.
    #[serde(rename = "thematicBreak")]
    ThematicBreak(ThematicBreak),

    // Table content.
    /// Table row.
    #[serde(rename = "tableRow")]
    TableRow(TableRow),

    // Row content.
    /// Table cell.
    #[serde(rename = "tableCell")]
    TableCell(TableCell),

    // List content.
    /// List item.
    #[serde(rename = "listItem")]
    ListItem(ListItem),

    // Content.
    /// Definition.
    #[serde(rename = "definition")]
    Definition(Definition),
    /// Paragraph.
    #[serde(rename = "paragraph")]
    Paragraph(Paragraph),

    /// Two columns.
    #[serde(rename = "twoColumns")]
    TwoColumns(TwoColumns),
}

impl std::fmt::Debug for Node {
    // Debug the wrapped struct.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Root(x) => x.fmt(f),
            Node::BlockQuote(x) => x.fmt(f),
            Node::FootnoteDefinition(x) => x.fmt(f),
            Node::List(x) => x.fmt(f),
            Node::Toml(x) => x.fmt(f),
            Node::Yaml(x) => x.fmt(f),
            Node::Break(x) => x.fmt(f),
            Node::InlineCode(x) => x.fmt(f),
            Node::InlineMath(x) => x.fmt(f),
            Node::Delete(x) => x.fmt(f),
            Node::Emphasis(x) => x.fmt(f),
            Node::FootnoteReference(x) => x.fmt(f),
            Node::Html(x) => x.fmt(f),
            Node::Image(x) => x.fmt(f),
            Node::ImageReference(x) => x.fmt(f),
            Node::Link(x) => x.fmt(f),
            Node::LinkReference(x) => x.fmt(f),
            Node::Strong(x) => x.fmt(f),
            Node::Text(x) => x.fmt(f),
            Node::Code(x) => x.fmt(f),
            Node::Math(x) => x.fmt(f),
            Node::Heading(x) => x.fmt(f),
            Node::Table(x) => x.fmt(f),
            Node::ThematicBreak(x) => x.fmt(f),
            Node::TableRow(x) => x.fmt(f),
            Node::TableCell(x) => x.fmt(f),
            Node::ListItem(x) => x.fmt(f),
            Node::Definition(x) => x.fmt(f),
            Node::Paragraph(x) => x.fmt(f),
            Node::TwoColumns(x) => x.fmt(f),
        }
    }
}

fn children_to_string(children: &[Node]) -> String {
    children.iter().map(ToString::to_string).collect()
}

impl ToString for Node {
    fn to_string(&self) -> String {
        match self {
            // Parents.
            Node::Root(x) => children_to_string(&x.children),
            Node::BlockQuote(x) => children_to_string(&x.children),
            Node::FootnoteDefinition(x) => children_to_string(&x.children),
            Node::List(x) => children_to_string(&x.children),
            Node::Delete(x) => children_to_string(&x.children),
            Node::Emphasis(x) => children_to_string(&x.children),
            Node::Link(x) => children_to_string(&x.children),
            Node::LinkReference(x) => children_to_string(&x.children),
            Node::Strong(x) => children_to_string(&x.children),
            Node::Heading(x) => children_to_string(&x.children),
            Node::Table(x) => children_to_string(&x.children),
            Node::TableRow(x) => children_to_string(&x.children),
            Node::TableCell(x) => children_to_string(&x.children),
            Node::ListItem(x) => children_to_string(&x.children),
            Node::Paragraph(x) => children_to_string(&x.children),

            // Literals.
            Node::Toml(x) => x.value.clone(),
            Node::Yaml(x) => x.value.clone(),
            Node::InlineCode(x) => x.value.clone(),
            Node::InlineMath(x) => x.value.clone(),
            Node::Html(x) => x.value.clone(),
            Node::Text(x) => x.value.clone(),
            Node::Code(x) => x.value.clone(),
            Node::Math(x) => x.value.clone(),

            // Voids.
            Node::Break(_)
            | Node::FootnoteReference(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::ThematicBreak(_)
            | Node::Definition(_) => String::new(),

            // custom
            Node::TwoColumns(x) => x.left.to_string() + &x.right.to_string(),
        }
    }
}

/// Document.
///
/// ```markdown
/// > | a
///     ^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Root {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
}

/// Paragraph.
///
/// ```markdown
/// > | a
///     ^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Paragraph {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
}

/// Heading.
///
/// ```markdown
/// > | # a
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Heading {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    // Extra.
    /// Rank (between `1` and `6`, both including).
    pub depth: u8,
}

/// Thematic break.
///
/// ```markdown
/// > | ***
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct ThematicBreak {
    // Void.
}

/// Block quote.
///
/// ```markdown
/// > | > a
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct BlockQuote {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
}

/// List.
///
/// ```markdown
/// > | * a
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct List {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    // Extra.
    /// Ordered (`true`) or unordered (`false`).
    pub ordered: bool,
    /// Starting number of the list.
    /// `None` when unordered.
    pub start: Option<u32>,
    /// One or more of its children are separated with a blank line from its
    /// siblings (when `true`), or not (when `false`).
    pub spread: bool,
}

/// List item.
///
/// ```markdown
/// > | * a
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct ListItem {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    // Extra.
    /// The item contains two or more children separated by a blank line
    /// (when `true`), or not (when `false`).
    pub spread: bool,
    /// GFM: whether the item is done (when `true`), not done (when `false`),
    /// or indeterminate or not applicable (`None`).
    pub checked: Option<bool>,
}

/// Html (flow or phrasing).
///
/// ```markdown
/// > | <a>
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Html {
    // Text.
    /// Content model.
    pub value: String,
}

/// Code (flow).
///
/// ```markdown
/// > | ~~~
///     ^^^
/// > | a
///     ^
/// > | ~~~
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Code {
    // Text.
    /// Content model.
    pub value: String,
    // Extra.
    /// The language of computer code being marked up.
    pub lang: Option<String>,
    /// Custom info relating to the node.
    pub meta: Option<String>,
}

/// Math (flow).
///
/// ```markdown
/// > | $$
///     ^^
/// > | a
///     ^
/// > | $$
///     ^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Math {
    // Text.
    /// Content model.
    pub value: String,
    // Extra.
    /// Custom info relating to the node.
    pub meta: Option<String>,
}

/// Definition.
///
/// ```markdown
/// > | [a]: b
///     ^^^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Definition {
    // Void.
    // Resource.
    /// URL to the referenced resource.
    pub url: String,
    /// Advisory info for the resource, such as something that would be
    /// appropriate for a tooltip.
    pub title: Option<String>,
    // Association.
    /// Value that can match another node.
    /// `identifier` is a source value: character escapes and character references
    /// are *not* parsed.
    /// Its value must be normalized.
    pub identifier: String,
    /// `label` is a string value: it works just like `title` on a link or a
    /// `lang` on code: character escapes and character references are parsed.
    ///
    /// To normalize a value, collapse markdown whitespace (`[\t\n\r ]+`) to a
    /// space, trim the optional initial and/or final space, and perform
    /// case-folding.
    pub label: Option<String>,
}

/// Text.
///
/// ```markdown
/// > | a
///     ^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Text {
    // Text.
    /// Content model.
    pub value: String,
}

/// Emphasis.
///
/// ```markdown
/// > | *a*
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Emphasis {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
}

/// Strong.
///
/// ```markdown
/// > | **a**
///     ^^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Strong {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
}

/// Code (phrasing).
///
/// ```markdown
/// > | `a`
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct InlineCode {
    // Text.
    /// Content model.
    pub value: String,
}

/// Math (phrasing).
///
/// ```markdown
/// > | $a$
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct InlineMath {
    // Text.
    /// Content model.
    pub value: String,
}

/// Break.
///
/// ```markdown
/// > | a\
///      ^
///   | b
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Break {
    // Void.
}

/// Link.
///
/// ```markdown
/// > | [a](b)
///     ^^^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Link {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    // Resource.
    /// URL to the referenced resource.
    pub url: String,
    /// Advisory info for the resource, such as something that would be
    /// appropriate for a tooltip.
    pub title: Option<String>,
}

/// Image.
///
/// ```markdown
/// > | ![a](b)
///     ^^^^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Image {
    // Void.
    // Alternative.
    /// Equivalent content for environments that cannot represent the node as
    /// intended.
    pub alt: String,
    // Resource.
    /// URL to the referenced resource.
    pub url: String,
    /// Advisory info for the resource, such as something that would be
    /// appropriate for a tooltip.
    pub title: Option<String>,
}

/// Link reference.
///
/// ```markdown
/// > | [a]
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct LinkReference {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    // Reference.
    /// Explicitness of a reference.
    #[cfg_attr(feature = "serde", serde(rename = "referenceType"))]
    pub reference_kind: ReferenceKind,
    // Association.
    /// Value that can match another node.
    /// `identifier` is a source value: character escapes and character references
    /// are *not* parsed.
    /// Its value must be normalized.
    pub identifier: String,
    /// `label` is a string value: it works just like `title` on a link or a
    /// `lang` on code: character escapes and character references are parsed.
    ///
    /// To normalize a value, collapse markdown whitespace (`[\t\n\r ]+`) to a
    /// space, trim the optional initial and/or final space, and perform
    /// case-folding.
    pub label: Option<String>,
}

/// Image reference.
///
/// ```markdown
/// > | ![a]
///     ^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct ImageReference {
    // Void.
    // Alternative.
    /// Equivalent content for environments that cannot represent the node as
    /// intended.
    pub alt: String,
    // Reference.
    /// Explicitness of a reference.
    #[cfg_attr(feature = "serde", serde(rename = "referenceType"))]
    pub reference_kind: ReferenceKind,
    // Association.
    /// Value that can match another node.
    /// `identifier` is a source value: character escapes and character references
    /// are *not* parsed.
    /// Its value must be normalized.
    pub identifier: String,
    /// `label` is a string value: it works just like `title` on a link or a
    /// `lang` on code: character escapes and character references are parsed.
    ///
    /// To normalize a value, collapse markdown whitespace (`[\t\n\r ]+`) to a
    /// space, trim the optional initial and/or final space, and perform
    /// case-folding.
    pub label: Option<String>,
}

/// GFM: footnote definition.
///
/// ```markdown
/// > | [^a]: b
///     ^^^^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct FootnoteDefinition {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    // Association.
    /// Value that can match another node.
    /// `identifier` is a source value: character escapes and character references
    /// are *not* parsed.
    /// Its value must be normalized.
    pub identifier: String,
    /// `label` is a string value: it works just like `title` on a link or a
    /// `lang` on code: character escapes and character references are parsed.
    ///
    /// To normalize a value, collapse markdown whitespace (`[\t\n\r ]+`) to a
    /// space, trim the optional initial and/or final space, and perform
    /// case-folding.
    pub label: Option<String>,
}

/// GFM: footnote reference.
///
/// ```markdown
/// > | [^a]
///     ^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct FootnoteReference {
    // Void.
    // Association.
    /// Value that can match another node.
    /// `identifier` is a source value: character escapes and character references
    /// are *not* parsed.
    /// Its value must be normalized.
    pub identifier: String,
    /// `label` is a string value: it works just like `title` on a link or a
    /// `lang` on code: character escapes and character references are parsed.
    ///
    /// To normalize a value, collapse markdown whitespace (`[\t\n\r ]+`) to a
    /// space, trim the optional initial and/or final space, and perform
    /// case-folding.
    pub label: Option<String>,
}

/// GFM: table.
///
/// ```markdown
/// > | | a |
///     ^^^^^
/// > | | - |
///     ^^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Table {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    // Extra.
    /// Represents how cells in columns are aligned.
    pub align: Vec<AlignKind>,
}

/// GFM: table row.
///
/// ```markdown
/// > | | a |
///     ^^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct TableRow {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
}

/// GFM: table cell.
///
/// ```markdown
/// > | | a |
///     ^^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct TableCell {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
}

/// GFM: delete.
///
/// ```markdown
/// > | ~~a~~
///     ^^^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Delete {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
}

/// Frontmatter: yaml.
///
/// ```markdown
/// > | ---
///     ^^^
/// > | a: b
///     ^^^^
/// > | ---
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Yaml {
    // Void.
    /// Content model.
    pub value: String,
}

/// Frontmatter: toml.
///
/// ```markdown
/// > | +++
///     ^^^
/// > | a: b
///     ^^^^
/// > | +++
///     ^^^
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct Toml {
    // Void.
    /// Content model.
    pub value: String,
}

/// 拓展语法：两栏布局
/// 主要用于样例的显示
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, TsType)]
pub struct TwoColumns {
    pub left: Box<Node>,
    pub right: Box<Node>,
}

impl From<markdown::mdast::Root> for Root {
    fn from(value: markdown::mdast::Root) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
        }
    }
}
impl From<markdown::mdast::BlockQuote> for BlockQuote {
    fn from(value: markdown::mdast::BlockQuote) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
        }
    }
}
impl From<markdown::mdast::FootnoteDefinition> for FootnoteDefinition {
    fn from(value: markdown::mdast::FootnoteDefinition) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
            identifier: value.identifier,
            label: value.label,
        }
    }
}
impl From<markdown::mdast::List> for List {
    fn from(value: markdown::mdast::List) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
            ordered: value.ordered,
            start: value.start,
            spread: value.spread,
        }
    }
}
impl From<markdown::mdast::Toml> for Toml {
    fn from(value: markdown::mdast::Toml) -> Self {
        Self { value: value.value }
    }
}
impl From<markdown::mdast::Yaml> for Yaml {
    fn from(value: markdown::mdast::Yaml) -> Self {
        Self { value: value.value }
    }
}
impl From<markdown::mdast::InlineCode> for InlineCode {
    fn from(value: markdown::mdast::InlineCode) -> Self {
        Self { value: value.value }
    }
}
impl From<markdown::mdast::InlineMath> for InlineMath {
    fn from(value: markdown::mdast::InlineMath) -> Self {
        Self { value: value.value }
    }
}
impl From<markdown::mdast::Delete> for Delete {
    fn from(value: markdown::mdast::Delete) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
        }
    }
}
impl From<markdown::mdast::Emphasis> for Emphasis {
    fn from(value: markdown::mdast::Emphasis) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
        }
    }
}
impl From<markdown::mdast::FootnoteReference> for FootnoteReference {
    fn from(value: markdown::mdast::FootnoteReference) -> Self {
        Self {
            identifier: value.identifier,
            label: value.label,
        }
    }
}
impl From<markdown::mdast::Html> for Html {
    fn from(value: markdown::mdast::Html) -> Self {
        Self { value: value.value }
    }
}
impl From<markdown::mdast::Image> for Image {
    fn from(value: markdown::mdast::Image) -> Self {
        Self {
            alt: value.alt,
            url: value.url,
            title: value.title,
        }
    }
}
impl From<markdown::mdast::ImageReference> for ImageReference {
    fn from(value: markdown::mdast::ImageReference) -> Self {
        Self {
            alt: value.alt,
            reference_kind: value.reference_kind.into(),
            identifier: value.identifier,
            label: value.label,
        }
    }
}
impl From<markdown::mdast::ReferenceKind> for ReferenceKind {
    fn from(value: markdown::mdast::ReferenceKind) -> Self {
        match value {
            markdown::mdast::ReferenceKind::Shortcut => Self::Shortcut,
            markdown::mdast::ReferenceKind::Collapsed => Self::Collapsed,
            markdown::mdast::ReferenceKind::Full => Self::Full,
        }
    }
}
impl From<markdown::mdast::Link> for Link {
    fn from(value: markdown::mdast::Link) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
            url: value.url,
            title: value.title,
        }
    }
}
impl From<markdown::mdast::LinkReference> for LinkReference {
    fn from(value: markdown::mdast::LinkReference) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
            reference_kind: value.reference_kind.into(),
            identifier: value.identifier,
            label: value.label,
        }
    }
}
impl From<markdown::mdast::Strong> for Strong {
    fn from(value: markdown::mdast::Strong) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
        }
    }
}
impl From<markdown::mdast::Text> for Text {
    fn from(value: markdown::mdast::Text) -> Self {
        Self { value: value.value }
    }
}
impl From<markdown::mdast::Code> for Code {
    fn from(value: markdown::mdast::Code) -> Self {
        Self {
            value: value.value,
            lang: value.lang,
            meta: value.meta,
        }
    }
}
impl From<markdown::mdast::Math> for Math {
    fn from(value: markdown::mdast::Math) -> Self {
        Self {
            value: value.value,
            meta: value.meta,
        }
    }
}
impl From<markdown::mdast::Heading> for Heading {
    fn from(value: markdown::mdast::Heading) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
            depth: value.depth,
        }
    }
}
impl From<markdown::mdast::Table> for Table {
    fn from(value: markdown::mdast::Table) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
            align: value.align.into_iter().map(|x| x.into()).collect(),
        }
    }
}
impl From<markdown::mdast::AlignKind> for AlignKind {
    fn from(value: markdown::mdast::AlignKind) -> Self {
        match value {
            markdown::mdast::AlignKind::Left => Self::Left,
            markdown::mdast::AlignKind::Right => Self::Right,
            markdown::mdast::AlignKind::Center => Self::Center,
            markdown::mdast::AlignKind::None => Self::None,
        }
    }
}
impl From<markdown::mdast::TableRow> for TableRow {
    fn from(value: markdown::mdast::TableRow) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
        }
    }
}
impl From<markdown::mdast::TableCell> for TableCell {
    fn from(value: markdown::mdast::TableCell) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
        }
    }
}
impl From<markdown::mdast::ListItem> for ListItem {
    fn from(value: markdown::mdast::ListItem) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
            spread: value.spread,
            checked: value.checked,
        }
    }
}
impl From<markdown::mdast::Definition> for Definition {
    fn from(value: markdown::mdast::Definition) -> Self {
        Self {
            url: value.url,
            title: value.title,
            identifier: value.identifier,
            label: value.label,
        }
    }
}
impl From<markdown::mdast::Paragraph> for Paragraph {
    fn from(value: markdown::mdast::Paragraph) -> Self {
        Self {
            children: value.children.into_iter().map(|x| x.into()).collect(),
        }
    }
}
impl From<markdown::mdast::Node> for Node {
    fn from(value: markdown::mdast::Node) -> Self {
        match value {
            markdown::mdast::Node::Root(r) => Node::Root(r.into()),
            markdown::mdast::Node::BlockQuote(r) => Node::BlockQuote(r.into()),
            markdown::mdast::Node::FootnoteDefinition(r) => Node::FootnoteDefinition(r.into()),
            // markdown::mdast::Node::MdxJsxFlowElement(_) => todo!(),
            markdown::mdast::Node::List(r) => Node::List(r.into()),
            // markdown::mdast::Node::MdxjsEsm(_) => todo!(),
            markdown::mdast::Node::Toml(r) => Node::Toml(r.into()),
            markdown::mdast::Node::Yaml(r) => Node::Yaml(r.into()),
            markdown::mdast::Node::Break(_) => Node::Break(Break {}),
            markdown::mdast::Node::InlineCode(r) => Node::InlineCode(r.into()),
            markdown::mdast::Node::InlineMath(r) => Node::InlineMath(r.into()),
            markdown::mdast::Node::Delete(r) => Node::Delete(r.into()),
            markdown::mdast::Node::Emphasis(r) => Node::Emphasis(r.into()),
            // markdown::mdast::Node::MdxTextExpression(_) => todo!(),
            markdown::mdast::Node::FootnoteReference(r) => Node::FootnoteReference(r.into()),
            markdown::mdast::Node::Html(r) => Node::Html(r.into()),
            markdown::mdast::Node::Image(r) => Node::Image(r.into()),
            markdown::mdast::Node::ImageReference(r) => Node::ImageReference(r.into()),
            // markdown::mdast::Node::MdxJsxTextElement(_) => todo!(),
            markdown::mdast::Node::Link(r) => Node::Link(r.into()),
            markdown::mdast::Node::LinkReference(r) => Node::LinkReference(r.into()),
            markdown::mdast::Node::Strong(r) => Node::Strong(r.into()),
            markdown::mdast::Node::Text(r) => Node::Text(r.into()),
            markdown::mdast::Node::Code(r) => Node::Code(r.into()),
            markdown::mdast::Node::Math(r) => Node::Math(r.into()),
            // markdown::mdast::Node::MdxFlowExpression(_) => todo!(),
            markdown::mdast::Node::Heading(r) => Node::Heading(r.into()),
            markdown::mdast::Node::Table(r) => Node::Table(r.into()),
            markdown::mdast::Node::ThematicBreak(_) => Node::ThematicBreak(ThematicBreak {}),
            markdown::mdast::Node::TableRow(r) => Node::TableRow(r.into()),
            markdown::mdast::Node::TableCell(r) => Node::TableCell(r.into()),
            markdown::mdast::Node::ListItem(r) => Node::ListItem(r.into()),
            markdown::mdast::Node::Definition(r) => Node::Definition(r.into()),
            markdown::mdast::Node::Paragraph(r) => Node::Paragraph(r.into()),
            _ => panic!("invalid node"),
        }
    }
}

#[test]
fn test_ts() {
    eprintln!("{}", Node::type_context().render_code(4));
    eprintln!("{}", Node::type_def());
}
