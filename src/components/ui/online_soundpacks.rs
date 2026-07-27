use crate::{
    state::app::{ use_app_state, use_state_trigger },
    utils::soundpack_installer::download_and_install_soundpack_with_progress,
};
use dioxus::prelude::*;
use lucide_dioxus::{ Download, Search, Check, RefreshCw };

const ONLINE_SOUNDPACKS_JSON: &str = include_str!("../../../assets/online_soundpacks.json");

#[derive(serde::Deserialize, Clone, Debug, PartialEq)]
struct OnlineSoundpack {
    id: String,
    name: String,
    download_url: String,
    #[serde(rename = "type")]
    pack_type: String,
}

#[component]
pub fn OnlineSoundpacksTable() -> Element {
    let app_state = use_app_state();
    let state_trigger = use_state_trigger();

    // Parse static soundpacks list once
    let online_packs = use_signal(|| {
        let packs: Vec<OnlineSoundpack> = serde_json::from_str(ONLINE_SOUNDPACKS_JSON).unwrap_or_default();
        packs
    });

    // Search and filter state
    let mut search_query = use_signal(String::new);
    let mut selected_type = use_signal(|| "All".to_string());

    // Tracking active downloads (pack_id -> is_downloading)
    let mut downloading_pack_id = use_signal(|| None::<String>);
    let mut download_progress = use_signal(|| 0.0);
    let mut download_error = use_signal(|| None::<String>);
    let mut download_success = use_signal(|| None::<String>);

    // Get list of local installed soundpacks
    let local_soundpacks = app_state.get_soundpacks();

    // Filter packs
    let filtered_packs: Vec<OnlineSoundpack> = {
        let query = search_query().to_lowercase();
        let filter_type = selected_type();
        
        online_packs()
            .iter()
            .filter(|pack| {
                let matches_query = query.is_empty() || 
                    pack.name.to_lowercase().contains(&query) || 
                    pack.id.to_lowercase().contains(&query);
                
                let matches_type = filter_type == "All" || pack.pack_type == filter_type;
                
                matches_query && matches_type
            })
            .cloned()
            .collect()
    };

    rsx! {
        div { class: "flex flex-col h-full",
            // Controls section (Search & Filters)
            div { class: "flex flex-col sm:flex-row gap-3 px-4 pb-4 border-b border-base-300",
                // Search Input
                div { class: "relative flex-grow",
                    div { class: "absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-base-content/40",
                        Search { class: "w-4 h-4" }
                    }
                    input {
                        r#type: "text",
                        placeholder: "Search official sound packs...",
                        class: "input input-bordered w-full pl-9 bg-base-300/40 border-base-content/10",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }
                
                // Type Filter Select
                select {
                    class: "select select-bordered bg-base-300/40 border-base-content/10",
                    value: "{selected_type}",
                    onchange: move |e| selected_type.set(e.value()),
                    option { value: "All", "All Types" }
                    option { value: "Keyboard", "Keyboard only" }
                    option { value: "Mouse", "Mouse only" }
                }
            }

            // Status message (Toasts / Notification bar)
            if let Some(ref err) = download_error() {
                div { class: "bg-error/15 text-error text-center py-2 text-xs font-semibold px-4 border-b border-error/20 flex items-center justify-between",
                    span { "❌ {err}" }
                    button { 
                        class: "btn btn-ghost btn-xs text-error", 
                        onclick: move |_| download_error.set(None),
                        "✕" 
                    }
                }
            }
            if let Some(ref succ) = download_success() {
                div { class: "bg-success/15 text-success text-center py-2 text-xs font-semibold px-4 border-b border-success/20 flex items-center justify-between animate-pulse",
                    span { "✅ {succ}" }
                    button { 
                        class: "btn btn-ghost btn-xs text-success", 
                        onclick: move |_| download_success.set(None),
                        "✕" 
                    }
                }
            }

            // Main List / Table
            div { class: "flex-grow overflow-y-auto max-h-[460px] custom-scrollbar",
                if filtered_packs.is_empty() {
                    div { class: "flex flex-col items-center justify-center py-16 text-center px-4",
                        span { class: "text-4xl mb-2", "🏜️" }
                        p { class: "text-sm font-semibold text-base-content/60", "No matching soundpacks found" }
                    }
                } else {
                    table { class: "table w-full",
                        thead {
                            tr { class: "border-b border-base-300 bg-base-200/50 sticky top-0 z-10",
                                th { class: "text-left text-xs font-bold text-base-content/60 py-2.5 pl-4", "Name" }
                                th { class: "text-center text-xs font-bold text-base-content/60 py-2.5", "Type" }
                                th { class: "text-right text-xs font-bold text-base-content/60 py-2.5 pr-4", "Action" }
                            }
                        }
                        tbody {
                            for pack in filtered_packs {
                                {
                                    let target_id = format!("{}/{}", pack.pack_type.to_lowercase(), pack.id);
                                    let is_installed = local_soundpacks.iter().any(|p| p.id == target_id);
                                    let is_downloading = downloading_pack_id().as_ref() == Some(&pack.id);
                                    let pack_clone = pack.clone();
                                    
                                    let type_badge_class = if pack.pack_type == "Keyboard" {
                                        "badge-primary text-primary"
                                    } else {
                                        "badge-secondary text-secondary"
                                    };
                                    

                                    rsx! {
                                        tr { 
                                            key: "{pack.id}",
                                            class: "border-b border-base-300/40 hover:bg-base-200/20 transition-colors duration-150",
                                            
                                            // Name and Status
                                            td { class: "py-3 pl-4",
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "font-semibold text-sm text-base-content", "{pack.name}" }
                                                    if is_installed {
                                                        div { class: "flex items-center gap-1 text-[10px] text-success font-medium",
                                                            Check { class: "w-3 h-3" }
                                                            "Installed"
                                                        }
                                                    }
                                                }
                                            }

                                            // Type Badge
                                            td { class: "text-center py-3",
                                                span { 
                                                    class: "badge badge-sm badge-outline text-[10px] font-semibold border-base-content/10 {type_badge_class}",
                                                    "{pack.pack_type}"
                                                }
                                            }

                                            // Action Button
                                            td { class: "text-right py-3 pr-4",
                                                div { class: "relative inline-flex",
                                                    // Progress bar background (shown during download)
                                                    if is_downloading {
                                                        div {
                                                            class: "progress-fill rounded-btn bg-primary/30",
                                                            style: "width: {download_progress()}%"
                                                        }
                                                    }
                                                    button {
                                                        class: format!(
                                                            "btn btn-xs gap-1 font-semibold overflow-hidden relative min-w-[90px] {}",
                                                            if is_downloading {
                                                                "btn-ghost bg-base-300 text-base-content/75"
                                                            } else if is_installed {
                                                                "btn-outline btn-ghost opacity-60"
                                                            } else {
                                                                "btn-primary"
                                                            }
                                                        ),
                                                        disabled: is_downloading || is_installed,
                                                        onclick: move |_| {
                                                            let url = pack_clone.download_url.clone();
                                                            let pack_id = pack_clone.id.clone();
                                                            let pack_name = pack_clone.name.clone();
                                                            let p_type = if pack_clone.pack_type == "Keyboard" {
                                                                crate::state::soundpack::SoundpackType::Keyboard
                                                            } else {
                                                                crate::state::soundpack::SoundpackType::Mouse
                                                            };

                                                            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<f64>();

                                                            downloading_pack_id.set(Some(pack_id.clone()));
                                                            download_progress.set(0.0);
                                                            download_error.set(None);
                                                            download_success.set(None);

                                                            // Spawn progress update receiver task on UI thread
                                                            spawn(async move {
                                                                while let Some(percent) = rx.recv().await {
                                                                    download_progress.set(percent);
                                                                }
                                                            });

                                                            spawn(async move {
                                                                let progress_cb = move |percent: f64| {
                                                                    let _ = tx.send(percent);
                                                                };

                                                                match download_and_install_soundpack_with_progress(&url, Some(p_type), Some(progress_cb)).await {
                                                                    Ok(info) => {
                                                                        download_success.set(Some(format!("Installed: {}", info.name)));
                                                                        downloading_pack_id.set(None);
                                                                        download_progress.set(0.0);

                                                                        // Refresh local soundpacks list
                                                                        crate::state::app::refresh_global_cache();
                                                                        state_trigger(());

                                                                        // Update tray configuration checked states
                                                                        crate::libs::tray_service::request_tray_update();
                                                                    }
                                                                    Err(e) => {
                                                                        download_error.set(Some(format!("Failed to install '{}': {}", pack_name, e)));
                                                                        downloading_pack_id.set(None);
                                                                        download_progress.set(0.0);
                                                                    }
                                                                }
                                                            });
                                                        },

                                                        if is_downloading {
                                                            span { class: "z-10 flex items-center gap-1.5 text-[10px]",
                                                                RefreshCw { class: "w-3 h-3 animate-spin" }
                                                                "{download_progress() as u32}%"
                                                            }
                                                        } else if is_installed {
                                                            Check { class: "w-3 h-3 animate-check-pop" }
                                                            "Installed"
                                                        } else {
                                                            Download { class: "w-3 h-3" }
                                                            "Download"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
