use dioxus::prelude::*;
use lucide_dioxus::Check;

#[derive(Clone, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ImportStep {
    Idle = 0,
    OpeningDialog = 1,
    FileSelected = 2,
    Validating = 3,
    CheckingConflicts = 4,
    Installing = 5,
    Finalizing = 6,
    Completed = 7,
}

#[derive(Props, Clone, PartialEq)]
pub struct ProgressStepProps {
    pub step_number: u8,
    pub title: String,
    pub is_active: bool,
    pub is_completed: bool,
}

#[component]
pub fn ProgressStep(props: ProgressStepProps) -> Element {
    rsx! {
      div { class: "flex items-center gap-3",
        span {
          class: format!(
              "flex items-center justify-center w-6 h-6 rounded-full text-xs font-medium {}",
              if props.is_active {
                  "bg-primary text-primary-content animate-pulse"
              } else if props.is_completed {
                  "bg-success text-success-content"
              } else {
                  "bg-base-300 text-base-content/60"
              },
          ),
          "{props.step_number}"
        }
        span {
          class: format!(
              "text-sm {}",
              if props.is_active {
                  "text-primary font-medium"
              } else if props.is_completed {
                  "text-success"
              } else {
                  "text-base-content/60"
              },
          ),
          "{props.title}"
        }
        if props.is_completed {
          Check { class: "w-4 h-4 ml-1 text-success" }
        } else if props.is_active {
          span { class: "loading loading-spinner loading-xs" }
        }
      }
    }
}
