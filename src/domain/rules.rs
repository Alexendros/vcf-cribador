//! Reglas de clasificación basadas en regex.

use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct ClassificationRule {
    pub pattern: Regex,
    pub n1: String,
    pub n2: String,
    pub n3: String,
}

pub static CLASSIFICATION_RULES: LazyLock<Vec<ClassificationRule>> = LazyLock::new(|| {
    vec![
        // ── Profesionales jurídicos ──
        rule(
            r"(?i)juzgado|instrucci[óo]n|tsj|audiencia.*penal",
            "PROF",
            "PROF-JUD",
            "JUD-JUZ",
        ),
        rule(r"(?i)fiscal[íi]a", "PROF", "PROF-FIS", "JUD-FIS"),
        rule(
            r"(?i)icav|icab|colegio\s+de\s+abogados",
            "PROF",
            "PROF-COL",
            "COL-ABG",
        ),
        rule(r"(?i)procurador", "PROF", "PROF-PROC", "COL-PRO"),
        rule(r"(?i)notar[ií]a|notario", "PROF", "PROF-NOT", "COL-NOT"),
        rule(
            r"(?i)despacho\s+jur[ií]dico|bufete\s+jur[ií]dico|consultor.*legal",
            "PROF",
            "PROF-CON",
            "JUR-CON",
        ),
        // ── Instituciones públicas ──
        rule(
            r"(?i)@gva\.es|@xij\.gencat|generalitat",
            "INST",
            "INST-AUT",
            "AUT-VAL",
        ),
        rule(
            r"(?i)ayuntamiento|@valencia\.es",
            "INST",
            "INST-LOC",
            "LOC-VAL",
        ),
        rule(
            r"(?i)@seg-social\.es|@policia\.es|@mineco\.es|tgss",
            "INST",
            "INST-EST",
            "EST-SS",
        ),
        rule(
            r"(?i)ministerio|delegaci[oó]n\s+del\s+gobierno|gobierno\s+de",
            "INST",
            "INST-EST",
            "EST-GOB",
        ),
        rule(
            r"(?i)hospital|cl[ií]nica|salud|sanidad|centro\s+de\s+salud",
            "SALUD",
            "SALUD-SAN",
            "SAN-HOSP",
        ),
        // ── Finanzas ──
        rule(
            r"(?i)revolut|cofidis|talenom|evobanco",
            "FIN",
            "FIN-FINTEC",
            "FINTEC",
        ),
        rule(
            r"(?i)bybit|bit2me|crypto\.com",
            "FIN",
            "FIN-CRYPTO",
            "CRYPTO",
        ),
        rule(
            r"(?i)seguros|caser|agencia\.caser",
            "FIN",
            "FIN-SEG",
            "SEGUROS",
        ),
        rule(
            r"(?i)axesor|procobro|gescobro",
            "FIN",
            "FIN-REC",
            "RECOBROS",
        ),
        rule(
            r"(?i)banco|caixa|bbva|santander|sabadell|bankinter|ing\s|openbank",
            "FIN",
            "FIN-BAN",
            "BANCA",
        ),
        rule(
            r"(?i)broker|inversi[oó]n|gestor[ií]a|asesor[ií]a\s+financiera",
            "FIN",
            "FIN-INV",
            "INV",
        ),
        // ── Formación ──
        rule(r"(?i)@itti\.es|itti", "FORM", "FORM-FP", "FP"),
        rule(r"(?i)unie|universidad", "FORM", "FORM-UNIV", "UNIV"),
        rule(
            r"(?i)academia|escuela|instituto|colegio\s+\w+|formaci[oó]n",
            "FORM",
            "FORM-EDU",
            "EDU",
        ),
        // ── Tecnología ──
        rule(
            r"(?i)gigabyte|sony|mitsubishi|ecodan",
            "TEC",
            "TEC-HW",
            "HW",
        ),
        rule(
            r"(?i)cursor|notion|perplexity|clickup|paragraph|neuralnine|bio\.link|mt5|appvillis",
            "TEC",
            "TEC-SW",
            "SW",
        ),
        rule(
            r"(?i)orange|digi|one\.com|@orange\.com",
            "TEC",
            "TEC-COM",
            "COM",
        ),
        rule(r"(?i)tidal|spotify|netflix", "TEC", "TEC-ENT", "ENT"),
        rule(
            r"(?i)hosting|cloud|vps|servidor|dominio",
            "TEC",
            "TEC-SVC",
            "SVC",
        ),
        // ── Comercio y servicios ──
        rule(
            r"(?i)hotel|restaurante|bar\s|caf[eé]|hostal",
            "HOST",
            "HOST-REST",
            "REST",
        ),
        rule(
            r"(?i)taxi|transporte|log[ií]stica|mensajer[ií]a",
            "TRAN",
            "TRAN-VEH",
            "VEH",
        ),
        rule(
            r"(?i)inmobiliaria|constructora|promotora|inmobiliari",
            "INMO",
            "INMO-INM",
            "INM",
        ),
        rule(
            r"(?i)limpieza|mantenimiento|seguridad|vigilancia",
            "SERV",
            "SERV-EXT",
            "EXT",
        ),
        rule(
            r"(?i)asociaci[oó]n|vecinos|comunidad",
            "ASOC",
            "ASOC-VEC",
            "VEC",
        ),
    ]
});

fn rule(pattern: &str, n1: &str, n2: &str, n3: &str) -> ClassificationRule {
    ClassificationRule {
        pattern: Regex::new(pattern).expect("regex de clasificación inválida"),
        n1: n1.to_string(),
        n2: n2.to_string(),
        n3: n3.to_string(),
    }
}
