use horrorshow::helper::*;
use horrorshow::prelude::*;

use std::fmt;

pub(crate) fn render_document<T: fmt::Display>(s: T) -> String {
    (html! {
        : doctype::HTML;
        html {
            body {
                : Raw(s.to_string())
            }
        }
    })
    .to_string()
}

pub(crate) fn render_type<T: fmt::Display>(s: T) -> String {
    (html! { span(class="type"): s.to_string() }).to_string()
}

pub(crate) fn render_variant<T: fmt::Display>(s: T) -> String {
    (html! { span(class="variant"): s.to_string() }).to_string()
}

pub(crate) fn render_tag<T: fmt::Display>(s: T) -> String {
    (html! { span(class="tag"): s.to_string() }).to_string()
}

pub(crate) fn render_module<T: fmt::Display>(s: T) -> String {
    (html! { span(class="module"): s.to_string() }).to_string()
}

pub(crate) fn render_function_name<T: fmt::Display>(s: T) -> String {
    (html! { span(class="function_name"): s.to_string() }).to_string()
}

pub(crate) fn render_param<T: fmt::Display>(s: T) -> String {
    (html! { span(class="param"): s.to_string() }).to_string()
}

pub(crate) fn render_output<T: fmt::Display>(s: T) -> String {
    (html! { span(class="output"): s.to_string() }).to_string()
}

pub(crate) fn render_function<T: fmt::Display>(
    header: T,
    raw_function_params: T,
    raw_function_results: T,
) -> String {
    (html! { section(class="function") {
        h3: Raw(header.to_string());
        : Raw(raw_function_params.to_string());
        : Raw(raw_function_results.to_string());
    }})
    .to_string()
}

pub(crate) fn render_function_params<T: fmt::Display>(raws: &[T]) -> String {
    (html! { section(class="function_params") {
        ul {
        @for raw in raws {
            li: Raw(raw.to_string())
        }
    }
    }})
    .to_string()
}

pub(crate) fn render_function_results<T: fmt::Display>(raws: &[T]) -> String {
    (html! { section(class="function_results") {
        ul {
        @for raw in raws {
            li: Raw(raw.to_string())
        }
    }
    }})
    .to_string()
}
