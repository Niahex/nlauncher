use numbat::{module_importer::BuiltinModuleImporter, Context, InterpreterResult};

pub struct Calculator {
    context: Context,
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    pub fn new() -> Self {
        let importer = BuiltinModuleImporter::default();
        let context = Context::new(importer);

        Self { context }
    }

    pub fn evaluate(&mut self, expression: &str) -> Option<String> {
        // Enlever le "=" du début
        let expr = expression.strip_prefix('=').unwrap_or(expression).trim();
        
        if expr.is_empty() {
            return None;
        }

        match self
            .context
            .interpret(expr, numbat::resolver::CodeSource::Text)
        {
            Ok((_, InterpreterResult::Value(value))) => Some(format!("{value}")),
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
    query.trim().starts_with('=')
}
