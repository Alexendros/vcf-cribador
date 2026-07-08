//! Reglas de clasificación basadas en regex.

use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct ClassificationRule {
    pub pattern: Regex,
    pub n1: &'static str,
    pub n2: &'static str,
}

pub static CLASSIFICATION_RULES: LazyLock<Vec<ClassificationRule>> = LazyLock::new(|| {
    vec![
        rule(
            r"(?i)juzgado|instrucci[óo]n|tsj|audiencia.*penal",
            "PROF",
            "PROF-JUD",
        ),
        rule(r"(?i)fiscal[íi]a", "PROF", "PROF-FIS"),
        rule(r"(?i)icav|icab|colegio\s+de\s+abogados", "PROF", "PROF-COL"),
        rule(r"(?i)procurador", "PROF", "PROF-PROC"),
        rule(r"(?i)notar[ií]a|notario", "PROF", "PROF-NOT"),
        rule(r"(?i)@gva\.es|@xij\.gencat|generalitat", "INST", "INST-AUT"),
        rule(r"(?i)ayuntamiento|@valencia\.es", "INST", "INST-LOC"),
        rule(
            r"(?i)@seg-social\.es|@policia\.es|@mineco\.es|tgss",
            "INST",
            "INST-EST",
        ),
        rule(r"(?i)revolut|cofidis|talenom|evobanco", "FIN", "FIN-FINTEC"),
        rule(r"(?i)bybit|bit2me|crypto\.com", "FIN", "FIN-CRYPTO"),
        rule(r"(?i)seguros|caser|agencia\.caser", "FIN", "FIN-SEG"),
        rule(r"(?i)axesor|procobro|gescobro", "FIN", "FIN-REC"),
        rule(r"(?i)@itti\.es|itti", "FORM", "FORM-FP"),
        rule(r"(?i)unie|universidad", "FORM", "FORM-UNIV"),
        rule(
            r"(?i)energy.?control|parc.?salut|defensor.?paciente",
            "SALUD",
            "SALUD-SOC",
        ),
        rule(r"(?i)gigabyte|sony|mitsubishi|ecodan", "TEC", "TEC-HW"),
        rule(
            r"(?i)cursor|notion|perplexity|clickup|paragraph|neuralnine|bio\.link|mt5|appvillis",
            "TEC",
            "TEC-SW",
        ),
        rule(r"(?i)orange|digi|one\.com|@orange\.com", "TEC", "TEC-COM"),
        rule(r"(?i)tidal", "TEC", "TEC-ENT"),
    ]
});

fn rule(pattern: &str, n1: &'static str, n2: &'static str) -> ClassificationRule {
    ClassificationRule {
        pattern: Regex::new(pattern).expect("regex de clasificación inválida"),
        n1,
        n2,
    }
}
