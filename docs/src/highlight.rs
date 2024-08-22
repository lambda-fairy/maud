use comrak::{
    plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder},
    Plugins,
};
use std::rc::Rc;
use syntect::highlighting::{Color, ThemeSet};

pub struct Highlighter {
    adapter: Rc<SyntectAdapter>,
}

impl Highlighter {
    pub fn get() -> Self {
        Self {
            adapter: SYNTECT_ADAPTER.with(Rc::clone),
        }
    }

    pub fn as_plugins(&self) -> Plugins<'_> {
        let mut plugins = Plugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&*self.adapter);
        plugins
    }
}

thread_local! {
    static SYNTECT_ADAPTER: Rc<SyntectAdapter> = Rc::new({
        SyntectAdapterBuilder::new()
            .theme_set({
                let mut ts = ThemeSet::load_defaults();
                let mut theme = ts.themes["InspiredGitHub"].clone();
                theme.settings.background = Some(Color {
                    r: 0xff,
                    g: 0xee,
                    b: 0xff,
                    a: 0xff,
                });
                ts.themes.insert("InspiredGitHub2".to_string(), theme);
                ts
            })
            .theme("InspiredGitHub2")
            .build()
    });
}
