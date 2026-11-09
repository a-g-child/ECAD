//! Validate bounded passive SVG without resolving files, network resources or executable content.
use cssparser::{Parser, ParserInput, Token};
use ecad_domain::{MAX_PICTURE_PIXELS, PictureError};
const SVG: &str = "http://www.w3.org/2000/svg";
const ELEMENTS: &[&str] = &[
    "svg",
    "g",
    "defs",
    "title",
    "desc",
    "path",
    "rect",
    "circle",
    "ellipse",
    "line",
    "polyline",
    "polygon",
    "text",
    "tspan",
    "textPath",
    "style",
    "use",
    "symbol",
    "switch",
    "linearGradient",
    "radialGradient",
    "stop",
    "pattern",
    "clipPath",
    "mask",
    "marker",
    "filter",
    "feBlend",
    "feColorMatrix",
    "feComponentTransfer",
    "feComposite",
    "feConvolveMatrix",
    "feDiffuseLighting",
    "feDisplacementMap",
    "feDistantLight",
    "feDropShadow",
    "feFlood",
    "feFuncA",
    "feFuncB",
    "feFuncG",
    "feFuncR",
    "feGaussianBlur",
    "feMerge",
    "feMergeNode",
    "feMorphology",
    "feOffset",
    "fePointLight",
    "feSpecularLighting",
    "feSpotLight",
    "feTile",
    "feTurbulence",
];
pub(crate) fn validate(bytes: &[u8]) -> Result<(u32, u32), PictureError> {
    let text = std::str::from_utf8(bytes).map_err(|_| PictureError)?;
    let doc = roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 10_000,
            ..Default::default()
        },
    )
    .map_err(|_| PictureError)?;
    let root = doc.root_element();
    if root.tag_name().name() != "svg" || root.tag_name().namespace() != Some(SVG) {
        return Err(PictureError);
    }
    for node in doc.descendants() {
        if node.is_pi() || node.ancestors().take(66).count() > 65 {
            return Err(PictureError);
        }
        if !node.is_element() {
            continue;
        }
        if node.tag_name().namespace() != Some(SVG) || !ELEMENTS.contains(&node.tag_name().name()) {
            return Err(PictureError);
        }
        for attr in node.attributes() {
            let name = attr.name();
            if name.to_ascii_lowercase().starts_with("on") || name == "base" {
                return Err(PictureError);
            }
            if let Some(ns) = attr.namespace()
                && ns != "http://www.w3.org/1999/xlink"
                && ns != "http://www.w3.org/XML/1998/namespace"
            {
                return Err(PictureError);
            }
            if name == "href" && !fragment(attr.value()) {
                return Err(PictureError);
            }
            // CSS escapes and comments must be interpreted, not searched as substrings.
            if matches!(
                name,
                "style"
                    | "fill"
                    | "stroke"
                    | "filter"
                    | "clip-path"
                    | "mask"
                    | "cursor"
                    | "marker"
                    | "marker-start"
                    | "marker-mid"
                    | "marker-end"
            ) {
                css(attr.value())?;
            }
        }
        if node.tag_name().name() == "style" {
            let body: String = node.children().filter_map(|n| n.text()).collect();
            css(&body)?;
        }
    }
    let view = root
        .attribute("viewBox")
        .map(|v| {
            let values = v
                .split(|c: char| c.is_ascii_whitespace() || c == ',')
                .filter(|v| !v.is_empty())
                .map(str::parse::<f64>)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| PictureError)?;
            if values.len() != 4
                || values.iter().any(|v| !v.is_finite())
                || values[2] <= 0.0
                || values[3] <= 0.0
            {
                return Err(PictureError);
            }
            Ok((values[2], values[3]))
        })
        .transpose()?;
    let width = length(root.attribute("width"), view.map(|v| v.0).unwrap_or(300.0))?;
    let height = length(root.attribute("height"), view.map(|v| v.1).unwrap_or(150.0))?;
    if u64::from(width) * u64::from(height) > MAX_PICTURE_PIXELS {
        return Err(PictureError);
    }
    Ok((width, height))
}
fn length(value: Option<&str>, fallback: f64) -> Result<u32, PictureError> {
    let number = if let Some(value) = value {
        let value = value.trim();
        if let Some(percent) = value.strip_suffix('%') {
            let percent = percent.parse::<f64>().map_err(|_| PictureError)?;
            if !percent.is_finite() || percent <= 0.0 {
                return Err(PictureError);
            }
            fallback
        } else {
            let units = [
                ("px", 1.0),
                ("mm", 96.0 / 25.4),
                ("cm", 96.0 / 2.54),
                ("in", 96.0),
                ("pt", 96.0 / 72.0),
                ("pc", 16.0),
            ];
            let (digits, scale) = units
                .iter()
                .find_map(|(unit, scale)| value.strip_suffix(unit).map(|v| (v, *scale)))
                .unwrap_or((value, 1.0));
            digits.trim().parse::<f64>().map_err(|_| PictureError)? * scale
        }
    } else {
        fallback
    };
    if !number.is_finite() || number <= 0.0 || number.ceil() > f64::from(u32::MAX) {
        return Err(PictureError);
    }
    Ok(number.ceil() as u32)
}
fn fragment(value: &str) -> bool {
    value
        .trim()
        .strip_prefix('#')
        .is_some_and(|s| !s.is_empty() && !s.contains(char::is_whitespace))
}
fn css(value: &str) -> Result<(), PictureError> {
    let mut input = ParserInput::new(value);
    tokens(&mut Parser::new(&mut input), 0)
}
fn tokens(parser: &mut Parser<'_, '_>, depth: u8) -> Result<(), PictureError> {
    if depth > 32 {
        return Err(PictureError);
    }
    while !parser.is_exhausted() {
        let token = parser
            .next_including_whitespace_and_comments()
            .map_err(|_| PictureError)?
            .clone();
        match token {
            Token::AtKeyword(_) | Token::BadUrl(_) | Token::BadString(_) => {
                return Err(PictureError);
            }
            Token::UnquotedUrl(url) if !fragment(&url) => return Err(PictureError),
            Token::Function(ref name) if name.eq_ignore_ascii_case("url") => {
                parser
                    .parse_nested_block(|p| {
                        let value = p.expect_string()?.to_string();
                        if !fragment(&value) {
                            return Err(p.new_custom_error::<(), ()>(()));
                        }
                        p.expect_exhausted()?;
                        Ok(())
                    })
                    .map_err(|_| PictureError)?;
            }
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                parser
                    .parse_nested_block(|p| {
                        tokens(p, depth + 1).map_err(|_| p.new_custom_error::<(), ()>(()))
                    })
                    .map_err(|_| PictureError)?;
            }
            _ => {}
        }
    }
    Ok(())
}
