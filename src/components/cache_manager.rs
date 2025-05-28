use crate::libs::audio::{cleanup_cache, get_cache_statistics};
use dioxus::prelude::*;
use lucide_dioxus::{HardDrive, RefreshCw, Trash2};

#[component]
pub fn CacheManager() -> Element {
    let cache_stats = use_signal(String::new);
    let mut is_loading = use_signal(|| false);
    let mut status_message = use_signal(String::new);

    // Get app state to access optimized cache
    use_effect(move || {
        load_cache_stats(cache_stats);
    });

    rsx! {
      div { class: "mt-4",
        // Cache Statistics
        div { class: "flex items-center gap-2 mb-3",
          HardDrive { class: "w-5 h-5 text-info" }
          h4 { class: "font-semibold", "Cache Statistics" }
        }

        if !cache_stats().is_empty() {
          pre { class: "text-xs text-base-content/70 whitespace-pre-wrap",
            "{cache_stats}"
          }
        } else {
          div { class: "text-base-content/50", "Loading cache statistics..." }
        }

        // Status Message
        if !status_message().is_empty() {
          div { class: "alert alert-info",
            span { "{status_message}" }
          }
        }

        // Action Buttons
        div { class: "flex gap-3 pt-4",
          button {
            class: "btn btn-soft btn-sm",
            disabled: is_loading(),
            onclick: move |_| {
                load_cache_stats(cache_stats);
                status_message.set("Cache statistics refreshed".to_string());
            },
            RefreshCw { class: "w-4 h-4 mr-2" }
            "Refresh Stats"
          }

          button {
            class: "btn btn-soft btn-sm",
            disabled: is_loading(),
            onclick: move |_| {
                is_loading.set(true);
                status_message.set("Clearing all cache files...".to_string());
                match cleanup_cache(0) {
                    Ok(_) => {
                        status_message.set("All cache files cleared successfully".to_string());
                        load_cache_stats(cache_stats);
                    }
                    Err(e) => {
                        status_message.set(format!("Failed to clear cache: {}", e));
                    }
                }
                is_loading.set(false);
            },
            Trash2 { class: "w-4 h-4 mr-2" }
            "Purge cache"
          }
        }

        // Help Text
        div { class: "text-xs text-base-content/60 pt-4 mt-4 border-t border-base-200",
          p { "• Cache files are automatically managed and cleaned up every 5 minutes" }
          p { "• The most recent 10 soundpacks are kept in cache by default" }
          p { "• Cached soundpacks load much faster than loading from files" }
        }
      }
    }
}

fn load_cache_stats(mut cache_stats: Signal<String>) {
    match get_cache_statistics() {
        Ok(stats) => cache_stats.set(stats),
        Err(e) => cache_stats.set(format!("Error loading cache stats: {}", e)),
    }
}
