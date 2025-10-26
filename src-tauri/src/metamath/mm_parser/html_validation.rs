use std::collections::{HashMap, HashSet};

use kuchiki::{parse_fragment, traits::TendrilSink, NodeRef};
use lightningcss::{
    printer::PrinterOptions,
    rules::CssRule,
    stylesheet::{ParserOptions, StyleSheet},
};
use markup5ever::{LocalName, Namespace, Prefix, QualName};

pub fn verify_html(
    html: &str,
    allowed_tags_and_attributes: &HashMap<String, HashSet<String>>,
    allowed_css_properties: &HashSet<String>,
) -> (bool, Option<String>) {
    let dom = parse_fragment(
        QualName::new(
            Some(Prefix::from("html")),
            Namespace::from("http://www.w3.org/1999/xhtml"),
            LocalName::from(""),
        ),
        Vec::new(),
    )
    .from_utf8()
    .one(html.as_bytes());

    let mut nodes: Vec<NodeRef> = dom
        .children()
        .next()
        .map(|node| node.children().collect())
        .unwrap_or(Vec::new());

    let mut id_sanitized = false;
    let mut valid_html = true;

    while let Some(node) = nodes.pop() {
        nodes.extend(node.children());

        if let Some(element_data) = node.as_element() {
            let element_name = element_data.name.local.as_ref();
            // println!("{}", element_name);

            if let Some(allowed_attributes) = allowed_tags_and_attributes.get(element_name) {
                let mut id_in_attributes = false;
                for (attr_name, attr_value) in &element_data.attributes.borrow().map {
                    if attr_name.local.as_ref() == "id" {
                        id_sanitized = true;
                        id_in_attributes = true;
                    } else {
                        if !allowed_attributes.contains(attr_name.local.as_ref()) {
                            // println!("Non allowed attibute: {}", attr_name.local.as_ref());
                            // println!("{}\n", html);
                            valid_html = false;
                        }
                        if attr_name.local.as_ref() == "style"
                            && !verify_inline_css(&attr_value.value, allowed_css_properties)
                        {
                            // println!("Non allowed inline css:");
                            // println!("{}\n", html);
                            valid_html = false;
                        }
                    }
                }
                if id_in_attributes {
                    element_data.attributes.borrow_mut().remove("id");
                }
            } else {
                // println!("Non allowed tag: {}", element_name);
                // println!("{}\n", html);
                valid_html = false;
            }
        }
    }

    if id_sanitized {
        (
            valid_html,
            Some(
                dom.children()
                    .next()
                    .map(|node| node.to_string())
                    .unwrap_or(String::new()),
            ),
        )
    } else {
        (valid_html, None)
    }
}

fn verify_inline_css(css: &str, allowed_css_properties: &HashSet<String>) -> bool {
    let wrapped_css = format!("div {{ {} }}", css);

    match StyleSheet::parse(&wrapped_css, ParserOptions::default()) {
        Ok(stylesheet) => {
            // Iterate through rules (expecting one rule for our dummy div)
            for rule in stylesheet.rules.0 {
                if let CssRule::Style(style) = rule {
                    for prop in style.declarations.declarations {
                        let prop_name = prop
                            .to_css_string(false, PrinterOptions::default())
                            .unwrap_or_default();

                        let parts: Vec<&str> = prop_name.split(':').collect();
                        if parts.len() != 2 {
                            return false;
                        }
                        let actual_prop_name = parts[0].trim().to_lowercase();
                        let actual_prop_value = parts[1].trim();

                        if !allowed_css_properties.contains(&*actual_prop_name)
                            || !verify_css_value(actual_prop_value)
                        {
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            }
        }
        Err(_) => {
            return false;
        }
    }

    true
}

fn verify_css_value(value: &str) -> bool {
    let lowercase_value = value.to_lowercase();

    !lowercase_value.contains("url(")
        && !lowercase_value.contains("expression(")
        && !lowercase_value.contains("javascript:")
        && !lowercase_value.contains("calc(")
}

pub fn create_rule_structs() -> (HashMap<String, HashSet<String>>, HashSet<String>) {
    let mut html_allowed_tags_and_attributes: HashMap<String, HashSet<String>> = HashMap::new();
    html_allowed_tags_and_attributes.insert(
        String::from("span"),
        HashSet::from([String::from("class"), String::from("style")]),
    );
    html_allowed_tags_and_attributes.insert(String::from("u"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("b"), HashSet::new());
    html_allowed_tags_and_attributes.insert(
        String::from("font"),
        HashSet::from([String::from("size"), String::from("face")]),
    );
    html_allowed_tags_and_attributes.insert(String::from("sup"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("sub"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("small"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("i"), HashSet::new());
    html_allowed_tags_and_attributes
        .insert(String::from("ol"), HashSet::from([String::from("type")]));
    html_allowed_tags_and_attributes.insert(String::from("li"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("code"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("pre"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("ul"), HashSet::new());
    html_allowed_tags_and_attributes.insert(
        String::from("table"),
        HashSet::from([
            String::from("border"),
            String::from("align"),
            String::from("cellspacing"),
            String::from("width"),
        ]),
    );
    html_allowed_tags_and_attributes.insert(String::from("tbody"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("tr"), HashSet::new());
    html_allowed_tags_and_attributes.insert(
        String::from("th"),
        HashSet::from([String::from("nowrap"), String::from("width")]),
    );
    html_allowed_tags_and_attributes.insert(
        String::from("td"),
        HashSet::from([
            String::from("nowrap"),
            String::from("style"),
            String::from("width"),
            String::from("rowspan"),
        ]),
    );
    html_allowed_tags_and_attributes.insert(String::from("p"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("br"), HashSet::new());
    html_allowed_tags_and_attributes
        .insert(String::from("a"), HashSet::from([String::from("href")]));
    html_allowed_tags_and_attributes.insert(String::from("tt"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("em"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("h1"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("h2"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("h3"), HashSet::new());
    html_allowed_tags_and_attributes.insert(String::from("it"), HashSet::new()); // todo: remove
    html_allowed_tags_and_attributes
        .insert(String::from("div"), HashSet::from([String::from("style")]));

    let css_allowed_properties: HashSet<String> = HashSet::from(
        [
            "color",
            "border-bottom",
            "text-decoration",
            "text-align",
            "overflow",
            "width",
            "height",
            "display",
            "font-size",
            "font-weight",
            "position",
            "top",
            "left",
            "line-height",
        ]
        .map(|s| s.to_string()),
    );

    (html_allowed_tags_and_attributes, css_allowed_properties)
}
