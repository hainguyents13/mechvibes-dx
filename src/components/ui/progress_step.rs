use dioxus::prelude::*;
use lucide_dioxus::{Check, X};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ImportStep {
    Idle = 0,
    FileSelecting = 1,
    Validating = 2,
    CheckingConflicts = 3,
    Installing = 4,
    Finalizing = 5,
    Refreshing = 6,
    Completed = 7,
}

#[derive(Props, Clone, PartialEq)]
pub struct ProgressStepProps {
    pub step_number: u8,
    pub title: String,
    pub current_step: ImportStep,
    pub error_message: String,
    pub success_message: String,
}

#[component]
pub fn ProgressStep(props: ProgressStepProps) -> Element {
    // Calculate is_active and is_completed based on current_step and step_number
    let current_step_num = props.current_step as u8;
    let is_completed = current_step_num > props.step_number
        || (current_step_num == props.step_number && current_step_num == 7); // Step 7 is completed when reached
    let is_active = current_step_num == props.step_number && !is_completed;
    rsx! {
      div { class: "space-y-2",
        div { class: "flex items-center",
          div {
            class: format!(
                "flex items-center mr-3 justify-center w-6 h-6 shrink-0 rounded-full text-xs font-medium {}",
                if !props.error_message.is_empty() {
                    "bg-error/70 text-error-content"
                } else if is_active {
                    "bg-base-300"
                } else if is_completed {
                    "bg-success/70 text-success-content"
                } else {
                    "bg-base-200 text-base-content/50"
                },
            ),
            if !props.error_message.is_empty() {
              X { class: "w-3 h-3" }
            } else if is_active {
              span { class: "loading loading-spinner loading-xs" }
            } else if is_completed {
              Check { class: "w-3 h-3" }
            } else {
              span { "{props.step_number}" }
            }
          }

          div {
            class: format!(
                "text-sm whitespace-nowrap {}",
                if !props.error_message.is_empty() {
                    "text-error"
                } else if is_active {
                    "text-base-content font-medium"
                } else if is_completed {
                    "text-base-content"
                } else {
                    "text-base-content/50"
                },
            ),
            if !props.success_message.is_empty() {
              "{props.title}:"
            } else {
              "{props.title}"
            }
          }
          if !props.success_message.is_empty() {
            div { class: "ml-1 text-sm text-base-content/50 truncate",
              "{props.success_message}"
            }
          }
        }

        // Success message under step (when completed and has success message)
        // if is_completed && !props.success_message.is_empty() {
        //   div { class: "ml-9 text-xs text-success bg-success/10 p-2 rounded border border-success/20",
        //     "{props.success_message}"
        //   }
        // }

        // Error message under step
        if !props.error_message.is_empty() {
          div { class: "ml-9 text-xs text-error bg-error/10 p-2 rounded border border-error/20",
            "{props.error_message}"
          }
        }
      }
    }
}
