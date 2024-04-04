mod download;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::app::download::Download;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/beacon-panel.css"/>

        <Title text="Beacon"/>
        <Body class="bg-gray-800"/>

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
            view! { <p class="text-white">Page not found</p> }.into_view()
        }>
            <main class="bg-gray-800">
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
    view! {
        <h1 class="text-white text-2xl">"Beacon"</h1>
    }
}
