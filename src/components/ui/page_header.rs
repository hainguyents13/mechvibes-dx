use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct PageHeaderProps {
    pub title: String,
    #[props(optional)]
    pub subtitle: Option<String>,
    #[props(optional)]
    pub icon: Option<Element>,
}

#[component]
pub fn PageHeader(props: PageHeaderProps) -> Element {
    rsx! {
      div { class: "mb-8 flex items-center gap-4",
        if let Some(icon) = props.icon {
          div { class: "p-3 bg-base-300 rounded-full flex-shrink-0",
            {icon}
          }
        }
        div { class: "flex-1",
          h1 { class: "text-2xl leading-tight font-bold text-left text-base-content",
            "{props.title}"
          }
          if let Some(subtitle) = props.subtitle {
            p { class: "text-base-content/50 leading-tight text-md text-left", "{subtitle}" }
          }
        }
      }
    }
}
