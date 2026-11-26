use leptos::{prelude::*, task::spawn_local};

#[component]
pub fn AdminPortal() -> impl IntoView {
    let (checking, set_checking) = signal(true);
    let (authorized, set_authorized) = signal(false);
    let (session_notice, set_session_notice) = signal(None::<String>);
    let check_interval = set_interval_with_handle(move || {
        if authorized.get_untracked() {
            spawn_local(async move {
                match gloo_net::http::Request::get("/api/admin/session").send().await {
                    Ok(response) if response.status() == 204 => {},
                    _ => {
                        set_authorized.set(false);
                        set_session_notice.set(Some("Your session could not be verified. Sign in again to view feedback.".into()));
                    }
                }
            });
        }
    }, std::time::Duration::from_secs(30)).ok();
    on_cleanup(move || {
        if let Some(handle) = check_interval {
            handle.clear();
        }
    });
    spawn_local(async move {
        match gloo_net::http::Request::get("/api/admin/session")
            .send()
            .await
        {
            Ok(response) => {
                set_authorized.set(response.status() == 204);
            }
            Err(_) => set_session_notice.set(Some(
                "The service is unavailable. Please check your connection.".into(),
            )),
        }
        set_checking.set(false);
    });
    let logout = move |_| {
        set_authorized.set(false);
        spawn_local(async move {
            match gloo_net::http::Request::delete("/api/admin/session").send().await {
                Ok(response) if response.status() == 204 => set_session_notice.set(Some("You’ve signed out.".into())),
                _ => set_session_notice.set(Some("Your inbox has been cleared from this page, but sign-out could not be confirmed. Close the browser and contact your service administrator.".into())),
            }
        });
    };
    view! {
        <main id="main" class="admin-layout">
            <div class="page-heading"><div><p class="eyebrow">"ADMINISTRATOR WORKSPACE"</p><h1>"Listen. Understand. Improve."</h1><p class="lead">"A considered space to review the feedback that matters."</p></div>
                <Show when=move || authorized.get()><button class="button secondary" on:click=logout>"Sign out ↗"</button></Show>
            </div>
            <p class="session-notice" role="status">{move || session_notice.get()}</p>
            {move || if checking.get() { view! { <div class="state-panel" role="status">"Checking your session…"</div> }.into_any() }
                else if authorized.get() { view! { <super::inbox::Inbox on_expired=Callback::new(move |()| { set_authorized.set(false); set_session_notice.set(Some("Your session has ended. Sign in again to view feedback.".into())); })/> }.into_any() }
                else { view! { <SignIn on_signed_in=Callback::new(move |()| { set_session_notice.set(None); set_authorized.set(true); })/> }.into_any() }}
        </main>
    }
}

#[component]
fn SignIn(on_signed_in: Callback<()>) -> impl IntoView {
    let (credential, set_credential) = signal(String::new());
    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() {
            return;
        }
        let secret = credential.get_untracked();
        set_credential.set(String::new());
        set_busy.set(true);
        set_error.set(None);
        spawn_local(async move {
            let payload = serde_json::json!({"credential": secret});
            let request = gloo_net::http::Request::post("/api/admin/session")
                .header("content-type", "application/json")
                .body(payload.to_string());
            match request {
                Ok(request) => match request.send().await {
                    Ok(response) if response.status() == 204 => on_signed_in.run(()),
                    Ok(response) => set_error.set(Some(match response.status() {
                        401 => "That access credential wasn’t accepted. Please try again.",
                        403 => "This page’s origin is not authorized. Contact your service administrator.",
                        429 => "Too many sign-in attempts. Wait one minute before trying again.",
                        _ => "Administrator sign-in is unavailable. Please try again later.",
                    }.into())),
                    Err(_) => set_error.set(Some("Unable to reach the service. Please check your connection.".into())),
                },
                Err(_) => set_error.set(Some("Unable to prepare sign-in. Please refresh the page.".into())),
            }
            set_busy.set(false);
        });
    };
    view! {
        <section class="login-layout" aria-labelledby="signin-title">
            <aside class="review-principles"><span class="section-label">"RESPONSIBLE REVIEW"</span><h2>"Every message deserves a thoughtful read."</h2><p>"Feedback can contain sensitive experiences. Keep it confidential and use it only to improve the student experience."</p><ul><li>"Read with context and care."</li><li>"Keep personal details private."</li><li>"Sign out when you’re finished."</li></ul><span class="review-symbol" aria-hidden="true">"↗"</span></aside>
            <div class="signin-card"><span class="section-label">"RESTRICTED ACCESS"</span><h2 id="signin-title">"Welcome back."</h2><p class="muted">"Sign in to review submitted feedback."</p>
                <form on:submit=submit aria-busy=move || busy.get().to_string()>
                    <label for="admin-credential">"Administrator access credential"</label>
                    <input id="admin-credential" type="password" name="credential" autocomplete="current-password" required prop:value=move || credential.get() disabled=move || busy.get() on:input=move |event| set_credential.set(event_target_value(&event))/>
                    <small>"Use the credential provided by your service administrator."</small>
                    <button type="submit" class="button primary" disabled=move || busy.get()>{move || if busy.get() { "Signing in…" } else { "Sign in securely ↗" }}</button>
                    <p role="alert" class="error-message">{move || error.get()}</p>
                </form>
                <p class="signin-note">"Access is limited to authorized reviewers. Your session expires after 30 minutes."</p>
            </div>
        </section>
    }
}
