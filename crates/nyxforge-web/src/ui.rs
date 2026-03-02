//! DOM helpers and page renderer.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlElement};

fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

fn get_element(id: &str) -> Option<Element> {
    document().get_element_by_id(id)
}

/// Initial render: inject the app shell if the host page has a `#nyxforge-root` element.
pub fn render_app() -> Result<(), JsValue> {
    let root = match get_element("nyxforge-root") {
        Some(el) => el,
        None => {
            web_sys::console::warn_1(&"#nyxforge-root not found in DOM".into());
            return Ok(());
        }
    };

    root.set_inner_html(APP_SHELL);
    Ok(())
}

const APP_SHELL: &str = r#"
<div class="nyx-app">
  <header>
    <h1>NyxForge</h1>
    <p class="tagline">Anonymous · Decentralised · Social Policy Bond Market</p>
  </header>

  <nav>
    <button onclick="nyxforge.showBonds()">Browse Bonds</button>
    <button onclick="nyxforge.showIssue()">Issue Bond</button>
    <button onclick="nyxforge.showWallet()">Wallet</button>
  </nav>

  <main id="nyx-main">
    <p>Loading bond market…</p>
  </main>

  <footer>
    <small>Running anonymously via WASM · No app store required</small>
  </footer>
</div>
"#;
