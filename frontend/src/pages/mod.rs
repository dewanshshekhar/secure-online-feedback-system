mod admin;
mod api;
mod feedback;
mod inbox;

use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let admin = web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .is_some_and(|path| path.starts_with("/admin"));
    view! {
        <a class="skip-link" href="#main">"Skip to content"</a>
        <header class="site-header">
            <a class="brand" href="/"><span class="brand-mark" aria-hidden="true">"sf"</span><span>"Secure Feedback"<small>"LISTEN. UNDERSTAND. IMPROVE."</small></span></a>
            <nav aria-label="Main navigation">
                <a class:active=!admin aria-current=(!admin).then_some("page") href="/">"Give feedback"</a>
                <a class:active=admin aria-current=admin.then_some("page") href="/admin">"Admin portal"<span aria-hidden="true">" ↗"</span></a>
            </nav>
        </header>
        {if admin {
            view! { <admin::AdminPortal/> }.into_any()
        } else { view! { <feedback::FeedbackForm/> }.into_any() }}
        <footer class="site-footer"><a href="/">"Secure Feedback"</a><p>"Thoughtful feedback. Meaningful change."</p><a href="/admin">"Administrator access"</a></footer>
    }
}
