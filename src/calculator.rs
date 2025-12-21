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
        // Vérifier si l'expression ressemble à un calcul
        if !self.looks_like_calculation(expression) {
            return None;
        }

        match self
            .context
            .interpret(expression, numbat::resolver::CodeSource::Text)
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
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Vérifier les patterns de calcul
    // Vérifier si c'est une expression mathématique simple
    let has_operator = trimmed.contains('+')
        || trimmed.contains('-')
        || trimmed.contains('*')
        || trimmed.contains('/')
        || trimmed.contains('^');

    let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());

    has_operator && has_digit
}
