# Security

As part of it's theorem explorer and unicode preview mmt1 renders user input HTML, which obviously always comes with obvious security concerns. In this document I want to describe how mmt1 deals with those.

## HTML whitelist

When mmt1 parses user input html, it checks it using an internal whitelist of tags, attributes and inline styles (see below). When you open a database which contains HTML not found on the whitelist, you will receive a warning and be asked to verify the HTML yourself. I would highly highly advise you not to ignore such warning, and actually verify each shown HTML snippet by hand. Sometimes mmt1 will render html found in the description of an `.mmp` file. If any invalid HTML is found, mmt1 will refuse to render it. This cannot be turned off.

As you can see, as long as you carefully verify any HTML not found on the whitelist, using mmt1 is perfectly safe. If you for whatever reason are not sure whether HTML found in a metamath database you are trying to open is safe, feel free to create an issue on [Github](https://github.com/marloBruder/mmt1), so that I can help you out.

As you will see below, the whitelist for what is considered safe html is by no means exhaustive and was put together based on the HTML found in `set.mm`. If you have a tag, attribute or inline style you want added to the list, please create an issue on [Github](https://github.com/marloBruder/mmt1), so I can add it to the list.

## The actual whitelist

Allowed tags and their allowed attributes:

- "span": Allowed Attributes: "class" "style"
- "u"
- "b"
- "font": Allowed Attributes: "size", "face"
- "sup"-
- "sub"
- "small"
- "i"
- "ol": Allowed Attributes: "type"
- "li"
- "code"
- "pre"
- "ul"
- "table": Allowed Attributes: "border", "align", "cellspacing", "width"
- "tbody"
- "tr"
- "th": Allowed Attributes: "nowrap", "width"
- "td": Allowed Attributes: "nowrap", "style", "width", "rowspan"
- "p"
- "br"
- "a": Allowed Attributes: "href"
- "tt"
- "em"
- "h1"
- "h2"
- "h3"
- "it"
- "div": Allowed Attributes: "style"

Allowed inline styles:

- "color",
- "border-bottom",
- "text-decoration",
- "text-align",
- "overflow",
- "width",
- "height",
- "display",
- "font-size",
- "font-weight",
- "position",
- "top",
- "left",
- "line-height",
