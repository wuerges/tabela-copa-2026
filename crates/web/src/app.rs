use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::pages::{bracket, guaranteed_thirds};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <header>
            <h1>Classificados Copa do Mundo 2026</h1>
            <nav>
                <A href="">Bracket</A>
                <A href="/guaranteed-thirds">Probabilidades</A>
            </nav>
        </header>
        <main>
            <Router>
                <Routes fallback=|| "Not found">
                    <Route path=path!("") view=bracket::BracketPage/>
                    <Route path=path!("/guaranteed-thirds") view=guaranteed_thirds::GuaranteedThirdsPage/>
                </Routes>
            </Router>
        </main>
    }
}
