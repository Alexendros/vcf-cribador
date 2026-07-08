# Política de Seguridad

## Reportar vulnerabilidades

Si descubres una vulnerabilidad de seguridad, por favor **no abras un issue público**.

Envía un correo a los mantenedores del proyecto con los detalles. Responderemos en un plazo máximo de 48 horas.

## Versiones soportadas

| Versión | Soportada          |
|---------|--------------------|
| 0.1.x   | ✅ Soporte completo |

## Consideraciones de seguridad

- **Archivos VCF**: vcf-cribador procesa archivos de contactos. No ejecutes la herramienta sobre archivos de fuentes no confiables sin revisarlos previamente.
- **Datos personales**: El archivo de salida VCF y la auditoría TSV contienen datos de contacto personales. Trátalos con el mismo nivel de seguridad que el archivo original.
- **Dependencias**: Usamos `cargo audit` semanalmente para detectar vulnerabilidades en dependencias.
