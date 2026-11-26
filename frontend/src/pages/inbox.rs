use super::api::{category_label, FeedbackPage};
use leptos::{prelude::*, task::spawn_local};

#[component]
pub fn Inbox(on_expired: Callback<()>) -> impl IntoView {
    let (page, set_page) = signal(None::<FeedbackPage>);
    let (offset, set_offset) = signal(0usize);
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (revision, set_revision) = signal(0u32);
    let (selected, set_selected) = signal(None::<String>);

    Effect::new(move |_| {
        let current_offset = offset.get();
        let generation = revision.get();
        set_page.set(None);
        set_selected.set(None);
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            let result = gloo_net::http::Request::get(&format!(
                "/api/admin/feedback?limit=20&offset={current_offset}"
            ))
            .send()
            .await;
            // Ignore an older response after a refresh or navigation.
            if offset.get_untracked() != current_offset || revision.get_untracked() != generation {
                return;
            }
            match result {
                Ok(response) if response.status() == 200 => {
                    match response.json::<FeedbackPage>().await {
                        Ok(data) => {
                            if offset.get_untracked() == current_offset
                                && revision.get_untracked() == generation
                            {
                                set_page.set(Some(data));
                            }
                        }
                        Err(_) => set_error.set(Some(
                            "The inbox response could not be read. Please refresh.".into(),
                        )),
                    }
                }
                Ok(response) if response.status() == 401 || response.status() == 404 => {
                    on_expired.run(())
                }
                _ => set_error.set(Some(
                    "We couldn’t load feedback. Please check your connection and try again.".into(),
                )),
            }
            set_busy.set(false);
        });
    });

    view! {
        <section class="inbox-card" aria-labelledby="inbox-title" aria-busy=move || busy.get().to_string()>
            <div class="inbox-toolbar"><div><span class="section-label">"SUBMITTED FEEDBACK"</span><h2 id="inbox-title">"The feedback inbox"</h2></div><button class="button secondary" disabled=move || busy.get() on:click=move |_| { set_offset.set(0); set_revision.update(|value| *value += 1); }>"↻ Refresh inbox"</button></div>
            <p class="inbox-caption">"Newest submissions first. Select a message to read the full feedback."</p>
            <Show when=move || busy.get()><div class="state-panel" role="status"><span class="loading-dot"></span>"Loading submitted feedback…"</div></Show>
            <Show when=move || error.get().is_some()><div class="state-panel error-message" role="alert"><h3>"The inbox is unavailable"</h3><p>{move || error.get()}</p><button class="button secondary" on:click=move |_| set_revision.update(|value| *value += 1)>"Try again"</button></div></Show>
            {move || page.get().map(|data| {
                if data.items.is_empty() {
                    view! { <div class="state-panel empty-state"><span class="empty-icon" aria-hidden="true">"↳"</span><h3>{if offset.get_untracked() == 0 { "A clear inbox. A place to listen." } else { "No more feedback on this page." }}</h3><p>"Submitted feedback will appear here. Refresh the inbox to check for new messages."</p></div> }.into_any()
                } else {
                    view! { <div class="feedback-list">{data.items.into_iter().map(move |record| {
                        let id = record.id.clone();
                        let toggle_id = id.clone();
                        let expanded_id = id.clone();
                        let panel_id = format!("feedback-{id}");
                        let time_label = record.created_at.replace('T', " ").trim_end_matches('Z').split('.').next().unwrap_or(&record.created_at).to_owned();
                        view! { <article class="feedback-item">
                            <button class="feedback-summary" aria-expanded=move || (selected.get().as_deref() == Some(&expanded_id)).to_string() aria-controls=panel_id.clone() on:click=move |_| {
                                set_selected.update(|value| *value = if value.as_ref() == Some(&toggle_id) { None } else { Some(toggle_id.clone()) });
                            }><span class="feedback-category">{category_label(&record.category)}</span><span class="feedback-preview">{record.message.clone()}</span><time datetime=record.created_at.clone()>{time_label}" UTC"</time><span aria-hidden="true">"↗"</span></button>
                            <div id=panel_id hidden=move || selected.get().as_deref() != Some(&id) class="feedback-detail"><p>{record.message}</p><div><span class="section-label">"RECEIPT"</span><code>{record.id}</code></div></div>
                        </article> }
                    }).collect_view()}</div> }.into_any()
                }
            })}
            <div class="inbox-pagination"><p>{move || page.get().map(|data| if data.items.is_empty() { "No messages on this page".to_owned() } else { format!("Showing {}–{} · 20 per page", offset.get() + 1, offset.get() + data.items.len()) })}</p><div>
                <button class="button secondary" disabled=move || busy.get() || offset.get() == 0 on:click=move |_| set_offset.update(|value| *value = value.saturating_sub(20))>"← Previous"</button>
                <button class="button secondary" disabled=move || busy.get() || !page.get().is_some_and(|data| data.has_more) on:click=move |_| { if let Some(next) = page.get_untracked().and_then(|data| data.next_offset) { set_offset.set(next); } }>"Next →"</button>
            </div></div>
        </section>
        <p class="confidential-note">"CONFIDENTIAL · Review feedback only for its intended purpose. Don’t copy personal information into other tools."</p>
    }
}
