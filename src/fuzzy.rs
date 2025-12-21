use crate::state::ApplicationInfo;
use nucleo::{Config, Nucleo, Utf32String};

pub struct FuzzyMatcher {
    matcher: Nucleo<usize>,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        let config = Config::DEFAULT;
        let matcher = Nucleo::new(config, std::sync::Arc::new(|| {}), None, 1);
        Self { matcher }
    }

    pub fn search(&mut self, query: &str, apps: &[ApplicationInfo]) -> Vec<usize> {
        if query.is_empty() {
            return (0..apps.len()).collect();
        }

        // Réinitialiser le matcher
        self.matcher.restart(false);

        // Injecter les applications
        let injector = self.matcher.injector();
        for (i, app) in apps.iter().enumerate() {
            let name = Utf32String::from(app.name.as_str());
            let _ = injector.push(i, |cols| cols[0] = name.clone());
        }

        // Parser le pattern et lancer la recherche
        let _pattern = nucleo::pattern::Pattern::parse(
            query,
            nucleo::pattern::CaseMatching::Ignore,
            nucleo::pattern::Normalization::Smart,
        );
        self.matcher.pattern.reparse(
            0,
            query,
            nucleo::pattern::CaseMatching::Ignore,
            nucleo::pattern::Normalization::Smart,
            false,
        );

        // Attendre que le matching soit terminé
        self.matcher.tick(10);

        // Récupérer les résultats
        let snapshot = self.matcher.snapshot();
        let mut results: Vec<usize> = Vec::new();

        for i in 0..snapshot.matched_item_count() {
            if let Some(item) = snapshot.get_matched_item(i) {
                results.push(*item.data);
            }
        }

        results
    }
}
