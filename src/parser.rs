use crate::{Agenda, ErrorKind};
use scraper::{ElementRef, Selector};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct CssSelectors {
    pub agenda_item: Selector,
    pub title: Selector,
    pub url: Selector,
    pub description: Selector,
}

impl Display for CssSelectors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TivoliCssSelectors").finish()
    }
}

fn get_text_for_single(logical_selector_name: &str,  text_element: &ElementRef) -> Result<String, ErrorKind> {
    if text_element.text().count() == 0 {
        return Err(ErrorKind::CannotFindSelector {
            selector: logical_selector_name.to_string(),
        });
    }
    let text = text_element.text().next().unwrap();
    Ok(text.trim().to_string())
}

fn get_select_on_element<'a>(
    logical_selector_name: &str,
    search_in_element: &ElementRef<'a>,
    selector: &Selector,
) -> Result<ElementRef<'a>, ErrorKind> {
    let selected = search_in_element.select(&selector);
    if selected.count() != 1 {
        return Err(ErrorKind::CannotFindSelector {
            selector: logical_selector_name.to_string(),
        });
    }
    Ok(search_in_element.select(selector).next().unwrap())
}

pub fn get_text_from_element(
    logical_selector_name: &str,
    search_in: &ElementRef,
    selector: &Selector,
) -> Result<String, ErrorKind> {
    let selected = get_select_on_element(logical_selector_name,&search_in, &selector)?;
    get_text_for_single(logical_selector_name, &selected)
}

pub fn optional_text_from_element(
    logical_selector_name: &str,
    search_in: &ElementRef,
    selector: &Selector,
) -> Result<Option<String>, ErrorKind> {
    let selected_result = get_select_on_element(logical_selector_name,&search_in, &selector);
    match selected_result {
        Ok(selected) => match get_text_for_single(logical_selector_name,&selected) {
            Ok(text) => Ok(Some(text)),
            Err(_) => Ok(None),
        },
        Err(ErrorKind::CannotFindSelector { selector: _ }) => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn get_text_from_attr(
    logical_selector_name: &str,
    search_in: &ElementRef,
    selector: &Selector,
    attr_name: &str,
) -> Result<String, ErrorKind> {
    let selected_element = get_select_on_element(logical_selector_name, &search_in, &selector)?;
    let attr = selected_element.value().attr(&attr_name);
    match attr {
        Some(value) => Ok(value.to_string()),
        None => Err(ErrorKind::CannotFindAttribute {
            attribute_name: attr_name.to_string(),
        }),
    }
}

pub fn selector_for(selector: &str) -> Result<Selector, ErrorKind> {
    match Selector::parse(&selector) {
        Ok(selector) => Ok(selector),
        Err(parse_error) => Err(ErrorKind::CssSelectorError {
            message: format!(
                "At line {}, col {}",
                parse_error.location.line, parse_error.location.column
            ),
        }),
    }
}

pub fn agenda_from_element(
    search_in: &ElementRef,
    css_selectors: &CssSelectors,
) -> Result<Agenda, ErrorKind> {
    let url = get_text_from_attr("url", &search_in, &css_selectors.url, "href")?;
    let title = get_text_from_element("title", &search_in, &css_selectors.title)?;
    let description = optional_text_from_element("description:", &search_in, &css_selectors.description)?;

    Ok(Agenda {
        title,
        description,
        url: url.to_string(),
    })
}
