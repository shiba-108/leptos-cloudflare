use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    ParamSegment, SsrMode, StaticSegment, WildcardSegment,
};

use crate::components::about_page::AboutPage;
use crate::components::app_layout::AppLayout;
use crate::components::architecture_page::ArchitecturePage;
use crate::components::auth_page::LoginPage;
use crate::components::community_detail_page::CommunityDetailPage;
use crate::components::community_index_page::CommunityIndexPage;
use crate::components::contact_page::ContactPage;
use crate::components::creator_dashboard_page::CreatorDashboardPage;
use crate::components::home_page::HomePage;
use crate::components::patterns_page::PatternsPage;
use crate::components::product_detail_page::ProductDetailPage;
use crate::components::resource_explorer::{ResourceDetailPage, ResourceExplorerPage};
use crate::components::search_page::SearchPage;
use crate::components::site_chrome::provide_admin_signal;
use crate::components::start_page::StartPage;
use crate::components::static_pages::{AboutViPage, ContactViPage, PrivacyViPage, TermsViPage};
use crate::components::todo_detail_page::TodoDetailPage;
use crate::components::todo_page::TodoPage;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    leptos::prelude::provide_context(RwSignal::new(crate::auth_client::stored_session()));
    provide_admin_signal();

    view! {
        <Title text="Leptos CF"/>
        <Meta
            name="description"
            content="A source-derived field guide and full-stack Leptos starter for Cloudflare Workers."
        />

        <Router>
            // Keeping it inside Router gives the shared nav links their routing context.
            <AppLayout>
                <Routes fallback=|| view! { <NotFoundPage/> }.into_view()>
                    <Route path=StaticSegment("") view=HomePage ssr=SsrMode::OutOfOrder/>
                    <Route path=StaticSegment("vi") view=HomePage ssr=SsrMode::OutOfOrder/>
                    <Route path=StaticSegment("start") view=StartPage ssr=SsrMode::OutOfOrder/>
                    <Route path=StaticSegment("architecture") view=ArchitecturePage ssr=SsrMode::OutOfOrder/>
                    <Route path=StaticSegment("patterns") view=PatternsPage ssr=SsrMode::OutOfOrder/>
                    // Bounded data is included in the HTML before hydration.
                    <Route path=(StaticSegment("patterns"), StaticSegment("resource-explorer")) view=ResourceExplorerPage ssr=SsrMode::Async/>
                    <Route path=(StaticSegment("patterns"), StaticSegment("resource-explorer"), ParamSegment("id")) view=ResourceDetailPage ssr=SsrMode::Async/>
                    <Route path=StaticSegment("lab") view=TodoPage ssr=SsrMode::OutOfOrder/>
                    <Route path=StaticSegment("about") view=AboutPage ssr=SsrMode::OutOfOrder/>
                    <Route path=StaticSegment("contact") view=ContactPage ssr=SsrMode::OutOfOrder/>
                    <Route path=(StaticSegment("vi"), StaticSegment("about")) view=AboutViPage ssr=SsrMode::OutOfOrder/>
                    <Route path=(StaticSegment("vi"), StaticSegment("terms")) view=TermsViPage ssr=SsrMode::OutOfOrder/>
                    <Route path=(StaticSegment("vi"), StaticSegment("privacy")) view=PrivacyViPage ssr=SsrMode::OutOfOrder/>
                    <Route path=(StaticSegment("vi"), StaticSegment("contact")) view=ContactViPage ssr=SsrMode::OutOfOrder/>
                    <Route path=(StaticSegment("vi"), StaticSegment("search")) view=SearchPage ssr=SsrMode::Async/>
                    <Route path=StaticSegment("dashboard") view=CreatorDashboardPage ssr=SsrMode::Async/>
                    <Route path=(StaticSegment("vi"), StaticSegment("dashboard")) view=CreatorDashboardPage ssr=SsrMode::Async/>
                    <Route path=(StaticSegment("vi"), StaticSegment("login")) view=LoginPage ssr=SsrMode::OutOfOrder/>
                    <Route path=(StaticSegment("vi"), StaticSegment("product"), ParamSegment("slug")) view=ProductDetailPage ssr=SsrMode::Async/>
                    <Route path=(StaticSegment("product"), ParamSegment("slug")) view=ProductDetailPage ssr=SsrMode::Async/>
                    <Route path=StaticSegment("community") view=CommunityIndexPage ssr=SsrMode::OutOfOrder/>
                    <Route path=(StaticSegment("vi"), StaticSegment("community")) view=CommunityIndexPage ssr=SsrMode::OutOfOrder/>
                    <Route path=(StaticSegment("vi"), StaticSegment("community"), ParamSegment("id")) view=CommunityDetailPage ssr=SsrMode::OutOfOrder/>

                    <Route
                        path=(StaticSegment("lab"), ParamSegment("id"))
                        view=TodoDetailPage
                        ssr=SsrMode::OutOfOrder
                    />
                    // Preserve old deep links while consumers move to `/lab/:id`.
                    <Route
                        path=(StaticSegment("todo"), ParamSegment("id"))
                        view=TodoDetailPage
                        ssr=SsrMode::OutOfOrder
                    />

                    // Critical for Cloudflare + Leptos on the edge:
                    // This must be last. It ensures deep links and pre-hydration
                    // requests get a full SSR HTML shell (in cooperation with
                    // the Worker entry in `src/edge/`).
                    <Route path=WildcardSegment("any") view=NotFoundPage ssr=SsrMode::OutOfOrder/>
                </Routes>
            </AppLayout>
        </Router>
    }
}

#[component]
fn NotFoundPage() -> impl IntoView {
    // The route that actually renders recovery owns its HTTP status. Keeping
    // another path registry in the Worker would reject newly added app routes.
    #[cfg(feature = "ssr")]
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.set_status(axum::http::StatusCode::NOT_FOUND);
    }

    view! {
        <Title text="Page not found — Leptos CF"/>
        <div class="page-shell page-shell--compact">
            <section class="route-miss section-stack section-stack--related">
                <p class="eyebrow">"Unknown folio"</p>
                <h1>"This route is outside the field guide."</h1>
                <p>"Return to the request map or open the checked-in start path."</p>
                <div class="action-cluster">
                    <a class="control-frame control-frame--standard control-frame--primary" href="/">"Field guide"</a>
                    <a class="control-frame control-frame--standard control-frame--secondary" href="/start">"Use the starter"</a>
                </div>
            </section>
        </div>
    }
}
