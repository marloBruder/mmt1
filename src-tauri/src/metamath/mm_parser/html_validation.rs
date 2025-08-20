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
) -> bool {
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

    while let Some(node) = nodes.pop() {
        nodes.extend(node.children());

        if let Some(element_data) = node.as_element() {
            let element_name = element_data.name.local.as_ref();
            // println!("{}", element_name);

            if let Some(allowed_attributes) = allowed_tags_and_attributes.get(element_name) {
                for (attr_name, attr_value) in &element_data.attributes.borrow().map {
                    if !allowed_attributes.contains(attr_name.local.as_ref()) {
                        return false;
                    }
                    if attr_name.local.as_ref() == "style"
                        && !verify_inline_css(&attr_value.value, allowed_css_properties)
                    {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
    }

    true
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
