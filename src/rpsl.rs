#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpslObject {
    pub fields: Vec<(String, String)>,
}

impl RpslObject {
    pub fn parse(content: &str) -> Self {
        let mut fields: Vec<(String, String)> = Vec::new();

        for line in content.lines() {
            if line.is_empty() || line.starts_with('%') {
                continue;
            }

            if line.starts_with(char::is_whitespace) || line.starts_with('+') {
                if let Some((_, value)) = fields.last_mut() {
                    value.push('\n');
                    value.push_str(
                        line.trim_start_matches(char::is_whitespace)
                            .trim_start_matches('+')
                            .trim_start(),
                    );
                }
                continue;
            }

            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            fields.push((key.to_string(), value.trim().to_string()));
        }

        Self { fields }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(field, _)| field.eq_ignore_ascii_case(key))
            .map(|(_, value)| value.as_str())
    }

    pub fn get_all(&self, key: &str) -> Vec<&str> {
        self.fields
            .iter()
            .filter(|(field, _)| field.eq_ignore_ascii_case(key))
            .map(|(_, value)| value.as_str())
            .collect()
    }
}
