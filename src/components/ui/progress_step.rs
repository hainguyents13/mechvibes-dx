use dioxus::prelude::*;
use lucide_dioxus::Check;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
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
    pub current_step: ImportStep,
    pub error_message: String,
}

#[component]
pub fn ProgressStep(props: ProgressStepProps) -> Element {
    // Calculate is_active and is_completed based on current_step and step_number
    let current_step_num = props.current_step as u8;
    let is_completed = current_step_num > props.step_number
        || (current_step_num == props.step_number && current_step_num == 7);

    // Step 7 is completed when reached
    let is_active = current_step_num == props.step_number && !is_completed;

    rsx! {
      div { class: "space-y-2",
        div { class: "flex items-center gap-3",
          span {
            class: format!(
                "flex items-center justify-center w-6 h-6 rounded-full text-xs font-medium {}",
                if !props.error_message.is_empty() {
                    "bg-error text-error-content"
                } else if is_active {
                    "bg-primary text-primary-content animate-pulse"
                } else if is_completed {
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
                if !props.error_message.is_empty() {
                    "text-error font-medium"
                } else if is_active {
                    "text-primary font-medium"
                } else if is_completed {
                    "text-success"
                } else {
                    "text-base-content/60"
                },
            ),
            "{props.title}"
          }
          if is_completed && props.error_message.is_empty() {
            Check { class: "w-4 h-4 ml-1 text-success" }
          } else if is_active && props.error_message.is_empty() {
            span { class: "loading loading-spinner loading-xs" }
          } else if !props.error_message.is_empty() {
            span { class: "text-error text-lg", "✗" }
          }
        }

        // Error message under step
        if !props.error_message.is_empty() {
          div { class: "ml-9 text-xs text-error bg-error/10 p-2 rounded border border-error/20",
            "{props.error_message}"
          }
        }
      }
    }
}
