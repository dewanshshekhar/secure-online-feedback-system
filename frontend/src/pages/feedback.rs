use leptos::{prelude::*, task::spawn_local};

#[component]
pub fn FeedbackForm() -> impl IntoView {
    let (category, set_category) = signal("teaching".to_string());
    let (message, set_message) = signal(String::new());
    let (status, set_status) = signal(None::<String>);
    let (submitting, set_submitting) = signal(false);
    let (receipt, set_receipt) = signal(None::<String>);
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if submitting.get_untracked() {
            return;
        }
        let text = message.get_untracked().trim().to_owned();
        if text.is_empty() || text.chars().count() > 2000 {
            set_status.set(Some(
                "Enter feedback between 1 and 2,000 characters.".into(),
            ));
            return;
        }
        set_submitting.set(true);
        set_status.set(None);
        spawn_local(async move {
            let payload =
                serde_json::json!({"category": category.get_untracked(), "message": text});
            let request = gloo_net::http::Request::post("/api/feedback")
                .header("content-type", "application/json")
                .body(payload.to_string());
            match request {
                Ok(request) => match request.send().await {
                    Ok(response) if response.status() == 201 => {
                        match response.json::<super::api::Receipt>().await {
                            Ok(saved) if !saved.id.is_empty() => {
                                set_receipt.set(Some(saved.id));
                                set_message.set(String::new());
                            }
                            _ => set_status.set(Some("Your submission was accepted, but its receipt couldn’t be read. Please don’t submit it again.".into())),
                        }
                    }
                    Ok(response) => set_status.set(Some(match response.status() {
                        413 => "Your feedback is too large. Please shorten it.",
                        400 | 422 => "Check your category and enter 1–2,000 characters.",
                        429 => "Too many requests. Please wait a moment before trying again.",
                        _ => "We couldn’t save your feedback. Your message is still here; please try again.",
                    }.into())),
                    Err(_) => set_status.set(Some("The connection was interrupted. Your submission may have reached us; retrying could send it twice. Your message is still here.".into())),
                },
                Err(_) => set_status.set(Some("We couldn’t prepare your submission. Please try again.".into())),
            }
            set_submitting.set(false);
        });
    };

    view! {
        <main id="main" class="public-layout">
            <div class="public-intro">
                <p class="eyebrow"><span class="tiny-dot"></span>"A BETTER EXPERIENCE STARTS WITH YOU"</p>
                <h1>"Your perspective."<br/><em>"Our next step."</em></h1>
                <p class="lead">"Tell us what’s working, what could be better, and what matters to you. Help shape a better learning experience."</p>
                <div class="how-it-works"><p class="section-label">"MAKE IT CONSTRUCTIVE"</p>
                    <div><span>"01"</span><p><strong>"Choose the right category"</strong><small>"Help the review team understand the context."</small></p></div>
                    <div><span>"02"</span><p><strong>"Share your experience"</strong><small>"Be specific, respectful, and describe what could improve."</small></p></div>
                    <div><span>"03"</span><p><strong>"Send it for review"</strong><small>"Your feedback goes to the administrator inbox."</small></p></div>
                </div>
                <aside class="privacy-note"><span aria-hidden="true">"↳"</span><p><strong>"Your words are enough."</strong>" We don’t ask for your name, email, or student number. Avoid including personal information in your message."</p></aside>
            </div>
            <section class="form-card" aria-labelledby="form-title">
                <div class="card-heading"><span class="section-label">"FEEDBACK FORM"</span><span class="badge">"Open for feedback"</span></div>
                <h2 id="form-title">"What’s on your mind?"</h2><p class="muted">"A little detail can make a real difference."</p>
                <Show when=move || receipt.get().is_none() fallback=move || view! {
                    <div class="success-panel" role="status">
                        <span class="success-icon" aria-hidden="true">"✓"</span><h3>"Thank you for speaking up."</h3>
                        <p>"Your feedback has been saved and is ready for the review team."</p>
                        <span class="section-label">"YOUR RECEIPT"</span><code>{move || receipt.get()}</code>
                        <small>"Keep this reference if you need to contact an administrator. It does not provide public access to your message."</small>
                        <button class="button secondary" on:click=move |_| { set_receipt.set(None); set_status.set(None); }>"Share more feedback"</button>
                    </div>
                }>
                <form on:submit=submit aria-busy=move || submitting.get().to_string()>
                    <fieldset disabled=move || submitting.get()>
                        <legend>"01 — Select a category"</legend>
                        <div class="category-grid">
                            {[("teaching", "Teaching", "Classes & instruction"), ("course-content", "Course content", "Materials & curriculum"), ("facilities", "Facilities", "Spaces & resources"), ("other", "Something else", "Anything on your mind")].into_iter().map(move |(value, label, description)| view! {
                                <label class="category-option" class:selected=move || category.get() == value>
                                    <input type="radio" name="category" value=value prop:checked=move || category.get() == value on:change=move |_| set_category.set(value.to_owned())/>
                                    <span><strong>{label}</strong><small>{description}</small></span>
                                </label>
                            }).collect_view()}
                        </div>
                        <label class="message-label" for="message">"02 — Your feedback"<span>"Required"</span></label>
                        <textarea id="message" name="message" required rows="7" prop:value=move || message.get() aria-describedby="message-help message-count" placeholder="What happened? What would you like to see improve?" on:input=move |event| set_message.set(event_target_value(&event))></textarea>
                        <div class="field-meta"><small id="message-help">"Please leave out personal details."</small><small id="message-count" class:over-limit={move || message.get().chars().count() > 2000}>{move || format!("{} / 2,000", message.get().chars().count())}</small></div>
                        <div class="form-bottom"><p>"Read it once more. Ready when you are."</p><button type="submit" class="button primary">{move || if submitting.get() { "Sending…" } else { "Send feedback ↗" }}</button></div>
                    </fieldset>
                    <p class="form-status" role="status">{move || status.get()}</p>
                </form>
                </Show>
                <div class="card-footnote"><span aria-hidden="true">"○"</span>"Reviewed by authorized administrators. No public posting."</div>
            </section>
        </main>
    }
}
