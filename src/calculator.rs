use numbat::{Context, InterpreterResult, module_importer::BuiltinModuleImporter};

pub struct Calculator {
    context: Context,
}

impl Calculator {
    pub fn new() -> Self {
        let importer = BuiltinModuleImporter::default();
        let context = Context::new(importer);
        
        Self { context }
    }

    pub fn evaluate(&mut self, expression: &str) -> Option<String> {
        // Vérifier si l'expression ressemble à un calcul
        if !self.looks_like_calculation(expression) {
            return None;
        }

        match self.context.interpret(expression, numbat::resolver::CodeSource::Text) {
            Ok((_, InterpreterResult::Value(value))) => {
                Some(format!("{}", value))
            }
            Ok((_, InterpreterResult::Continue)) => None,
            Err(_) => None,
        }
    }

    fn looks_like_calculation(&self, input: &str) -> bool {
        // Vérifier si l'input contient des caractères mathématiques
        let math_chars = ['+', '-', '*', '/', '(', ')', '^', '='];
        let has_math = input.chars().any(|c| math_chars.contains(&c));
        
        // Ou si c'est un nombre
        let is_number = input.trim().parse::<f64>().is_ok();
        
        // Ou si ça contient des unités communes
        let common_units = ["km", "m", "cm", "kg", "g", "°C", "°F", "USD", "EUR"];
        let has_units = common_units.iter().any(|unit| input.contains(unit));
        
        has_math || is_number || has_units
    }
}

pub fn is_calculator_query(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Vérifier les patterns de calcul
    let math_patterns = [
        r"\d+\s*[\+\-\*/\^]\s*\d+",  // 2+2, 5*3, etc.
        r"\d+\s*\w+",                 // 5km, 100USD, etc.
        r"^\d+\.?\d*$",               // nombres simples
    ];

    for pattern in &math_patterns {
        if regex::Regex::new(pattern).unwrap().is_match(trimmed) {
            return true;
        }
    }

    false
}
