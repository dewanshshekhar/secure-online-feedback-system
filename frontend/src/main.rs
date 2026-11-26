#![forbid(unsafe_code)]

mod pages;

fn main() {
    leptos::mount::mount_to_body(pages::HomePage);
}
