use anyhow::{Context, Result};

use crate::routine::RoutineResult;

pub fn to_json(result: &RoutineResult) -> Result<String> {
    serde_json::to_string_pretty(result).context("Failed to serialize to JSON")
}

pub fn to_xml(result: &RoutineResult) -> Result<String> {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str("<routine>\n");
    xml.push_str(&format!("  <type>{}</type>\n", result.routine_type));
    xml.push_str(&format!("  <greeting><![CDATA[{}]]></greeting>\n", result.greeting));
    xml.push_str(&format!("  <timestamp>{}</timestamp>\n", result.timestamp));

    if let Some(ref yp) = result.year_progress {
        xml.push_str("  <year_progress>\n");
        xml.push_str(&format!("    <day_of_year>{}</day_of_year>\n", yp.day_of_year));
        xml.push_str(&format!("    <total_days>{}</total_days>\n", yp.total_days));
        xml.push_str(&format!("    <percentage>{:.1}</percentage>\n", yp.percentage));
        xml.push_str(&format!("    <bar><![CDATA[{}]]></bar>\n", yp.bar));
        xml.push_str("  </year_progress>\n");
    }

    if !result.problems.is_empty() {
        xml.push_str("  <problems>\n");
        for problem in &result.problems {
            xml.push_str("    <problem>\n");
            xml.push_str(&format!("      <platform>{}</platform>\n", problem.platform));
            xml.push_str(&format!("      <id>{}</id>\n", problem.problem.id));
            xml.push_str(&format!(
                "      <title><![CDATA[{}]]></title>\n",
                problem.problem.title
            ));
            xml.push_str(&format!("      <slug>{}</slug>\n", problem.problem.slug));
            xml.push_str(&format!(
                "      <difficulty>{}</difficulty>\n",
                problem.problem.difficulty
            ));
            xml.push_str(&format!("      <url>{}</url>\n", problem.url));
            xml.push_str(&format!(
                "      <is_daily_challenge>{}</is_daily_challenge>\n",
                problem.is_daily_challenge
            ));
            xml.push_str("    </problem>\n");
        }
        xml.push_str("  </problems>\n");
    }

    if let Some(ref stats) = result.running {
        xml.push_str("  <running>\n");
        xml.push_str(&format!("    <yesterday_km>{:.2}</yesterday_km>\n", stats.yesterday_km));
        xml.push_str(&format!(
            "    <yesterday_count>{}</yesterday_count>\n",
            stats.yesterday_count
        ));
        xml.push_str(&format!("    <month_km>{:.2}</month_km>\n", stats.month_km));
        xml.push_str(&format!(
            "    <month_count>{}</month_count>\n",
            stats.month_count
        ));
        xml.push_str(&format!("    <year_km>{:.2}</year_km>\n", stats.year_km));
        xml.push_str(&format!(
            "    <year_count>{}</year_count>\n",
            stats.year_count
        ));
        xml.push_str("  </running>\n");
    }

    if let Some(ref events) = result.history {
        xml.push_str("  <history>\n");
        for event in events {
            xml.push_str("    <event>\n");
            xml.push_str(&format!("      <year>{}</year>\n", event.year));
            xml.push_str(&format!(
                "      <text><![CDATA[{}]]></text>\n",
                event.text
            ));
            xml.push_str(&format!("      <url>{}</url>\n", event.url));
            xml.push_str(&format!(
                "      <age_context><![CDATA[{}]]></age_context>\n",
                event.age_context
            ));
            xml.push_str("    </event>\n");
        }
        xml.push_str("  </history>\n");
    }

    if let Some(ref q) = result.quote {
        xml.push_str("  <quote>\n");
        xml.push_str(&format!("    <text><![CDATA[{}]]></text>\n", q.text));
        xml.push_str(&format!(
            "    <author><![CDATA[{}]]></author>\n",
            q.author
        ));
        xml.push_str(&format!("    <source>{}</source>\n", q.source));
        xml.push_str("  </quote>\n");
    }

    xml.push_str(&format!(
        "  <formatted_message><![CDATA[{}]]></formatted_message>\n",
        result.formatted_message
    ));
    xml.push_str("</routine>");

    Ok(xml)
}
