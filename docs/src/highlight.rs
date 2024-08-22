use comrak::{
    adapters::SyntaxHighlighterAdapter,
    plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder},
    Plugins,
};
use std::{collections::HashMap, io, rc::Rc};
use syntect::highlighting::{Color, ThemeSet};

pub struct Highlighter {
    adapter: Rc<StripHiddenCodeAdapter<SyntectAdapter>>,
}

impl Highlighter {
    pub fn get() -> Self {
        Self {
            adapter: ADAPTER.with(Rc::clone),
        }
    }

    pub fn as_plugins(&self) -> Plugins<'_> {
        let mut plugins = Plugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&*self.adapter);
        plugins
    }
}

thread_local! {
    static ADAPTER: Rc<StripHiddenCodeAdapter<SyntectAdapter>> = Rc::new(StripHiddenCodeAdapter {
        inner: create_syntect_adapter(),
    });
}

struct StripHiddenCodeAdapter<P> {
    inner: P,
}

impl<P: SyntaxHighlighterAdapter> SyntaxHighlighterAdapter for StripHiddenCodeAdapter<P> {
    fn write_highlighted(
        &self,
        output: &mut dyn io::Write,
        lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
        if lang == Some("rust") {
            let stripped_code = code
                .split('\n')
                .filter(|line| {
                    let line = line.trim();
                    line != "#" && !line.starts_with("# ")
                })
                .collect::<Vec<_>>()
                .join("\n");
            return self.inner.write_highlighted(output, lang, &stripped_code);
        }
        self.inner.write_highlighted(output, lang, code)
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn io::Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        self.inner.write_pre_tag(output, attributes)
    }

    fn write_code_tag(
        &self,
        output: &mut dyn io::Write,
        attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        if attributes["class"].ends_with("no_run") {
            panic!("{:?}", attributes)
        }
        self.inner.write_code_tag(output, attributes)
    }
}

fn create_syntect_adapter() -> SyntectAdapter {
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
}
