use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::download::Download;

pub mod download;
pub mod file;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/beacon-panel.css"/>

        <Title text="Beacon"/>

        <Router fallback=|| {
            // When on the server, return a 404 error.
            // On the client this would not do anything.
            #[cfg(feature = "ssr")]
            {
                use http::StatusCode;
                use leptos_axum::ResponseOptions;

                let response = use_context::<ResponseOptions>();
                if let Some(response) = response {
                    response.set_status(StatusCode::NOT_FOUND);
                }
            }
            view! { <p>Page not found</p> }.into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/files/:file_id/:name" view=Download/>
                </Routes>
            </main>
        </Router>
    }
}

/// The home page of the panel.
#[component]
fn HomePage() -> impl IntoView {
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1 class="text-2xl">"Beacon"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
