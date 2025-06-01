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
      div { class: "mb-8 text-center",
        if let Some(icon) = props.icon {
          div { class: "mx-auto p-2 bg-base-300 inline-block rounded-full mb-2",
            {icon}
          }
        }
        div {
          h1 { class: "text-2xl font-bold text-center text-base-content ",
            "{props.title}"
          }
          if let Some(subtitle) = props.subtitle {
            p { class: "text-base-content/50 text-md text-center", "{subtitle}" }
          }
        }
      }
    }
}
