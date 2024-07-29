use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug)]
enum Tag {
    Text(String),
    A,
    Aside,
    B,
    Blockquote,
    Br,
    Code,
    Em,
    FigCaption,
    Figure,
    H3,
    H4,
    Hr,
    I,
    IFrame,
    Img,
    Li,
    Ol,
    P,
    Pre,
    S,
    Strong,
    U,
    Ul,
    Video,
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match &self {
            Tag::Text(_) => "text",
            Tag::A => "a",
            Tag::Aside => "aside",
            Tag::B => "b",
            Tag::Blockquote => "blockquote",
            Tag::Br => "br",
            Tag::Code => "code",
            Tag::Em => "em",
            Tag::FigCaption => "figcaption",
            Tag::Figure => "figure",
            Tag::H3 => "h3",
            Tag::H4 => "h4",
            Tag::Hr => "hr",
            Tag::I => "i",
            Tag::IFrame => "iframe",
            Tag::Img => "img",
            Tag::Li => "li",
            Tag::Ol => "ok",
            Tag::P => "p",
            Tag::Pre => "pre",
            Tag::S => "s",
            Tag::Strong => "strong",
            Tag::U => "u",
            Tag::Ul => "ul",
            Tag::Video => "video",
        }
        .to_string();

        write!(f, "{}", str)
    }
}

impl Default for Tag {
    fn default() -> Self {
        Tag::Text(String::new())
    }
}

#[derive(Debug, Default)]
struct NodeElementAttribute {
    href: Option<String>,
    src: Option<String>,
}

impl NodeElementAttribute {
    fn new(href: Option<String>, src: Option<String>) -> Self {
        NodeElementAttribute { href, src }
    }

    fn to_hashmap(&self) -> HashMap<&str, String> {
        let mut attrs = HashMap::new();
        if let Some(value) = &self.href {
            attrs.insert("href", value.clone());
        }
        if let Some(value) = &self.src {
            attrs.insert("src", value.clone());
        }

        attrs
    }
}

#[derive(Debug, Default)]
pub struct NodeElement {
    tag: Tag,
    attributes: NodeElementAttribute,
    children: Vec<NodeElement>,
}

impl NodeElement {
    pub fn to_json(&self) -> Value {
        match &self.tag {
            Tag::Text(value) => json!(value),
            _ => {
                let tag = format!("{}", &self.tag);
                let attrs = self.attributes.to_hashmap();
                let children: Vec<Value> =
                    self.children.iter().map(|value| value.to_json()).collect();
                json!({
                    "tag": tag,
                    "attrs": attrs,
                    "children": children,
                })
            }
        }
    }

    pub fn a(href: &str, children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::A,
            attributes: NodeElementAttribute {
                href: Some(href.to_string()),
                src: None,
            },
            children,
        }
    }

    pub fn b(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::B,
            attributes: Default::default(),
            children,
        }
    }

    pub fn i(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::I,
            attributes: Default::default(),
            children,
        }
    }

    pub fn u(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::U,
            attributes: Default::default(),
            children,
        }
    }

    pub fn s(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::S,
            attributes: Default::default(),
            children,
        }
    }

    pub fn p(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::P,
            attributes: Default::default(),
            children,
        }
    }

    pub fn strong(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::Strong,
            attributes: Default::default(),
            children,
        }
    }

    pub fn h3(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::H3,
            attributes: Default::default(),
            children,
        }
    }

    pub fn h4(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::H4,
            attributes: Default::default(),
            children,
        }
    }

    pub fn pre(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::Pre,
            attributes: Default::default(),
            children,
        }
    }

    pub fn ol(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::Ol,
            attributes: Default::default(),
            children,
        }
    }

    pub fn li(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::Ol,
            attributes: Default::default(),
            children,
        }
    }

    pub fn ul(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::Ul,
            attributes: Default::default(),
            children,
        }
    }

    pub fn code(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::Code,
            attributes: Default::default(),
            children,
        }
    }

    pub fn em(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::Em,
            attributes: Default::default(),
            children,
        }
    }

    pub fn blockquote(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::Blockquote,
            attributes: Default::default(),
            children,
        }
    }

    pub fn aside(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::Aside,
            attributes: Default::default(),
            children,
        }
    }

    pub fn iframe(src: &str) -> Self {
        NodeElement {
            tag: Tag::IFrame,
            attributes: NodeElementAttribute::new(None, Some(src.to_string())),
            children: Default::default(),
        }
    }

    pub fn img(src: &str) -> Self {
        NodeElement {
            tag: Tag::Img,
            attributes: NodeElementAttribute::new(None, Some(src.to_string())),
            children: Default::default(),
        }
    }

    pub fn video(src: &str) -> Self {
        NodeElement {
            tag: Tag::Video,
            attributes: NodeElementAttribute::new(None, Some(src.to_string())),
            children: Default::default(),
        }
    }

    pub fn figure(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::Figure,
            attributes: Default::default(),
            children,
        }
    }

    pub fn figcaption(children: Vec<NodeElement>) -> Self {
        NodeElement {
            tag: Tag::FigCaption,
            attributes: Default::default(),
            children,
        }
    }

    pub fn br() -> Self {
        NodeElement {
            tag: Tag::Br,
            ..Default::default()
        }
    }

    pub fn hr() -> Self {
        NodeElement {
            tag: Tag::Br,
            ..Default::default()
        }
    }

    pub fn text(text: &str) -> Self {
        NodeElement {
            tag: Tag::Text(text.to_string()),
            ..Default::default()
        }
    }
}