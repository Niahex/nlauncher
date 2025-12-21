use crate::state::ApplicationInfo;
use nucleo::{Config, Nucleo};

pub struct FuzzyMatcher {
    _matcher: Nucleo<usize>,
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
        Self { _matcher: matcher }
    }

    pub fn search(&mut self, query: &str, apps: &[ApplicationInfo]) -> Vec<usize> {
        if query.is_empty() {
            return (0..apps.len()).collect();
        }

        // Pour l'instant, utilisons une recherche simple mais efficace
        // TODO: Implémenter nucleo correctement quand on aura plus de temps
        let query_lower = query.to_lowercase();
        let mut results: Vec<(usize, i32)> = apps
            .iter()
            .enumerate()
            .filter_map(|(i, app)| {
                let name_lower = app.name.to_lowercase();
                if name_lower.contains(&query_lower) {
                    // Score simple : plus la correspondance est au début, meilleur c'est
                    let score = if name_lower.starts_with(&query_lower) {
                        1000
                    } else if let Some(pos) = name_lower.find(&query_lower) {
                        1000 - pos as i32
                    } else {
                        0
                    };
                    Some((i, score))
                } else {
                    None
                }
            })
            .collect();

        // Trier par score décroissant
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results.into_iter().map(|(i, _)| i).collect()
    }
}
